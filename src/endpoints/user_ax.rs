use axum::body::Body;
use axum::extract::Json;
use axum_macros::debug_handler;
use http::{Response, StatusCode};
use tracing::{debug, debug_span, error, info, Instrument};

use crate::auth::AuthUser;
use crate::database::{insert_auth_token, insert_new_user};
use crate::error::ServiceError;
use crate::model::user::{RawUserData, User, UserCreationData, UserData, ValidUserData};
use crate::JsonBody;
use crate::{helpers, StateExtension};

#[debug_handler]
pub(crate) async fn create(
    state: StateExtension,
    Json(raw_user_data): Json<RawUserData>,
    // req: Request<Body>,
) -> Result<Response<Body>, ServiceError> {
    let pool = state.db_pool.clone();
    let settings = state.settings.clone();
    let secret = settings.app.secret.as_ref();

    debug!(
        "create called, port={} db_name={}",
        settings.app.port, settings.database.name,
    );

    let valid_user_data: ValidUserData = raw_user_data.try_into().map_err(|err| {
        error!("Err: {:?}", err);
        err
    })?;

    let user = insert_new_user(&pool, valid_user_data, secret)
        .instrument(debug_span!("insert_new_user"))
        .await
        .map_err(|err| {
            error!("Err: {:?}", err);
            err
        })?;

    let token = insert_auth_token(&pool, &user.id)
        .instrument(debug_span!("insert_auth_token"))
        .await
        .map_err(|err| {
            error!("Err: {:?}", err);
            err
        })?;

    let data = UserCreationData { token, id: user.id };
    let json = serde_json::to_vec(&JsonBody::new(data))?;

    let location = format!(
        "{}:{}/user/{}",
        settings.app.host, settings.app.port, user.id
    );

    let res = Response::builder()
        .header("Location", location)
        .status(StatusCode::CREATED)
        .body(Body::from(json))
        .expect("Failed to create response.");

    Ok(res)
}

pub async fn login(
    state: StateExtension,
    Json(RawUserData { username, password }): Json<RawUserData>,
) -> Result<Response<Body>, ServiceError> {
    let pool = state.db_pool.clone();
    let settings = state.settings.clone();
    let secret = settings.app.secret.as_ref();

    debug!(
        "login called, port={} db_name={}",
        settings.app.port, settings.database.name,
    );

    let query_user_span = debug_span!("query_user_span");
    let row = sqlx::query!(
        r#"
            select id as user_id, hashed_password from users
            where username = $1
        "#,
        username,
    )
    .fetch_one(&pool)
    .instrument(query_user_span)
    .await;

    debug!("User row found: {:?}", &row);

    let (user_id, hashed_password) = match row {
        Ok(row) => {
            debug!("Found matching user_id {}", &row.user_id);
            (row.user_id, row.hashed_password)
        }
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                error!("Err: {:?}", err);
                return Err(ServiceError::Unauthorized);
            }
            _ => {
                error!("Err: {:?}", err);
                return Err(err.into());
            }
        },
    };

    let is_valid = helpers::verify_password(&hashed_password, &password, secret)?;

    if !is_valid {
        return Err(ServiceError::Unauthorized);
    }

    let query_token_span = debug_span!("query_token_span");
    let row = sqlx::query!(
        r#"
            select token from auth_tokens
            where user_id = $1
        "#,
        user_id,
    )
    .fetch_one(&pool)
    .instrument(query_token_span)
    .await?;

    let json = serde_json::to_string(&JsonBody::new(row.token))?;

    let res = Response::new(Body::from(json));

    info!("Successfully logged in user_id={}", user_id);
    Ok(res)
}

#[debug_handler]
pub(crate) async fn get_user(
    state: StateExtension,
    AuthUser { user_id }: AuthUser,
) -> Result<Response<Body>, ServiceError> {
    let pool = state.db_pool.clone();
    let settings = state.settings.clone();

    debug!(
        "get_user called, port={} db_name={} user_id={:?}",
        settings.app.port, settings.database.name, user_id,
    );

    let user_id = user_id;

    let query_span = debug_span!("query_span");
    let user = sqlx::query_as!(
        User,
        r#"
            select * from users where id = $1;
        "#,
        user_id.take(),
    )
    .fetch_one(&pool)
    .instrument(query_span)
    .await;

    let user_data = match user {
        Ok(user) => {
            debug!("Found user id={} username={}", user.id, user.username);
            UserData {
                id: user.id,
                username: user.username,
            }
        }
        Err(err) => match err {
            // Requested user doesn't exist, e.g. token must be illegal.
            sqlx::Error::RowNotFound => {
                error!("Err: {:?}", err);
                return Err(ServiceError::Forbidden);
            }
            _ => {
                error!("Err: {:?}", err);
                return Err(err.into());
            }
        },
    };

    let json = serde_json::to_vec(&JsonBody::new(user_data))?;

    let res = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(json))
        .expect("Failed to create response.");

    info!("Successfully got user_id={:?}", user_id);
    Ok(res)
}
