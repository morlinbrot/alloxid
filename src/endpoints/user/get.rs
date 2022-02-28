use axum::body::Body;
use axum_macros::debug_handler;
use http::Response;
use tracing::{debug, debug_span, error, info, Instrument};

use crate::auth::AuthUser;
use crate::error::ServiceError;
use crate::model::user::{UserData, UserEntry};
use crate::JsonBody;
use crate::StateExtension;

#[debug_handler]
pub(crate) async fn get(
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
        UserEntry,
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

    info!("Successfully got user_id={:?}", user_id);
    Ok(Response::new(Body::from(json)))
}
