use axum::body::Body;
use axum::extract::Json;
use axum_macros::debug_handler;
use http::{Response, StatusCode};
use tracing::{debug, debug_span, error, info, Instrument};

use crate::database::{insert_auth_token, insert_new_user};
use crate::error::ServiceError;
use crate::{helpers, StateExtension};
// use crate::auth::UserId;
use crate::model::user::{RawUserData, UserCreationData, ValidUserData};
use crate::JsonBody;

// https://github.com/launchbadge/realworld-axum-sqlx/blob/main/src/http/extractor.rs
//
// use crate::auth::AUTHORIZATION;
// use axum::async_trait;
// use axum::extract::{Extension, FromRequest, Json, RequestParts};
// #[async_trait]
// impl<B> FromRequest<B> for RawUserData
// where
//     B: Send, // required by `async_trait`
// {
//     type Rejection = ServiceError;
//
//     async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
//         let ctx: Extension<ApiContext> = Extension::from_request(req)
//             .await
//             .expect("BUG: ApiContext was not added as an extension");
//
//         // Get the value of the `Authorization` header, if it was sent at all.
//         let auth_header = req
//             .headers()
//             .ok_or(ServiceError::WithStatusCode(StatusCode::UNAUTHORIZED))?
//             .get(AUTHORIZATION)
//             .ok_or(ServiceError::WithStatusCode(StatusCode::UNAUTHORIZED))?;
//
//         Self::from_authorization(&ctx, auth_header)
//     }
// }

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
        "Handler called, port={} db_name={}",
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
        "Handler called, port={} db_name={}",
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

    let (user_id, hashed_password) = match row {
        Ok(row) => {
            debug!("Found matching user_id {}", &row.user_id);
            (row.user_id, row.hashed_password)
        }
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                error!("Err: {:?}", err);
                let res = Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .expect("Failed to create response.");
                return Ok(res);
            }
            _ => {
                error!("Err: {:?}", err);
                // return Ok(Response::new(StatusCode::INTERNAL_SERVER_ERROR));
                let res = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .expect("Failed to create response.");
                return Ok(res);
            }
        },
    };

    let is_valid = helpers::verify_password(&hashed_password, &password, secret)?;
    debug!(is_valid);

    if !is_valid {
        let res = Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .expect("Failed to create response.");
        return Ok(res);
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
