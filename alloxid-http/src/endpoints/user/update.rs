use axum::body::Body;
use axum::extract::{Json, Path};
use axum_macros::debug_handler;
use http::Response;
use tracing::{debug, debug_span, info, Instrument};
use uuid::Uuid;

use crate::error::ServiceError;
use crate::model::user::{UserData, UserUpdateRaw};
use crate::{JsonBody, StateExtension};

#[debug_handler]
pub(crate) async fn update(
    state: StateExtension,
    Path(user_id): Path<Uuid>,
    Json(UserUpdateRaw { username }): Json<UserUpdateRaw>,
) -> Result<Response<Body>, ServiceError> {
    let pool = state.db_pool.clone();
    let settings = state.settings.clone();

    debug!(
        "update called, port={} db_name={} user_id={}",
        settings.app.port, settings.database.name, user_id,
    );

    let query_span = debug_span!("query_span");
    let updated_user = sqlx::query_as!(
        UserData,
        r#"
            update users
            set username = $2
            where id = $1
            returning id, username
        "#,
        user_id,
        username,
    )
    .fetch_one(&pool)
    .instrument(query_span)
    .await?;

    let json = serde_json::to_vec(&JsonBody::new(updated_user))?;

    info!("Successfully updated user_id={}", user_id);
    Ok(Response::new(Body::from(json)))
}
