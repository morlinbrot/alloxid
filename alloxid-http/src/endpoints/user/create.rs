use axum::body::Body;
use axum::extract::Json;
use axum::response::IntoResponse;
use axum_macros::debug_handler;
use http::{Response, StatusCode};
use tracing::{debug, debug_span, error, Instrument};

use crate::database::{insert_auth_token, insert_new_user};
use crate::error::ServiceError;
use crate::model::user::{UserAuthData, UserCreateRaw, ValidUserData};
use crate::JsonBody;
use crate::StateExtension;

#[debug_handler]
pub(crate) async fn create(
    state: StateExtension,
    Json(raw_user_data): Json<UserCreateRaw>,
) -> Result<impl IntoResponse, ServiceError> {
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

    let data = UserAuthData { token, id: user.id };
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

    // Ok((StatusCode::CREATED, headers, Json(json)))
    Ok(res)
}
