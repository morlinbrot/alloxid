use axum::response::IntoResponse;
use axum_macros::debug_handler;
use tracing::debug;

use crate::{auth::AuthUser, error::ServiceError, StateExtension};

#[debug_handler]
pub(crate) async fn delete(
    state: StateExtension,
    AuthUser { user_id }: AuthUser,
    // Json(raw_user_data): Json<UserCreateRaw>,
) -> Result<impl IntoResponse, ServiceError> {
    let pool = state.db_pool.clone();
    let settings = state.settings.clone();

    let user_id = user_id.take();

    debug!(
        "delete called, port={} db_name={}",
        settings.app.port, settings.database.name,
    );

    sqlx::query!(r#" delete from auth_tokens where user_id = $1; "#, &user_id)
        .execute(&pool)
        .await?;

    sqlx::query!(r#" delete from users where id = $1; "#, &user_id)
        .execute(&pool)
        .await?;

    debug!("Successfully deleted user_id={:?}", user_id);
    Ok(())
}
