use axum::body::Body;
use axum::extract::Json;
use http::Response;
use tracing::{debug, debug_span, error, info, Instrument};

use crate::error::ServiceError;
use crate::model::user::UserCreateRaw;
use crate::JsonBody;
use crate::{helpers, StateExtension};

pub async fn login(
    state: StateExtension,
    Json(UserCreateRaw { username, password }): Json<UserCreateRaw>,
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

    info!("Successfully logged in user_id={}", user_id);
    Ok(Response::new(Body::from(json)))
}
