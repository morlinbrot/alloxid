use std::sync::Arc;

use axum::body::Body;
use axum::extract::Extension;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{AddExtensionLayer, Router};
use http::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
// use tower::layer::layer_fn;
use tower::ServiceBuilder;
// use tower_http::auth::AsyncRequireAuthorizationLayer;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

pub mod error;
pub mod model;
pub mod settings;
pub mod telemetry;

mod auth;
mod database;
mod endpoints;
mod helpers;

// use auth::middleware_ax::AuthMiddleware;
use endpoints::user_ax;
use error::*;
use settings::Settings;

pub type Result<T, E = ServiceError> = std::result::Result<T, E>;
pub type ServiceResult<T = tide::Response, E = ServiceError> = std::result::Result<T, E>;
pub type StateExtension = Extension<Arc<State>>;

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonBody<T> {
    pub data: T,
}

impl<T> JsonBody<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub db_pool: PgPool,
    pub settings: Settings,
}

async fn health_check() -> &'static str {
    "Hello, world!"
}

async fn handle_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

pub async fn configure_app(db_pool: PgPool, settings: Settings) -> Result<axum::Router> {
    let state = Arc::new(State { db_pool, settings });

    // let headers: Arc<[_]> = Arc::new([header::AUTHORIZATION]);

    let service = ServiceBuilder::new()
        // .layer(layer_fn(|inner| AuthMiddleware { inner }))
        .layer(AddExtensionLayer::new(state))
        // .layer(RequireAuthorizationLayer::custom(AuthMiddleware))
        .layer(TraceLayer::new_for_http().make_span_with(
            |_req: &Request<Body>| tracing::debug_span!( "http-request", req_id = %Uuid::new_v4()),
        ));

    let app = Router::new()
        // .route("/", get(root))
        .route("/health-check", get(health_check))
        .route("/user", post(user_ax::create))
        .route("/user/login", post(user_ax::login))
        .route("/user/login/x", post(user_ax::login))
        .layer(service);

    let app = app.fallback(handle_404.into_service());

    Ok(app)

    // let state = State { db_pool, settings };
    // let mut app = tide::with_state(state);
    //
    // let cors = CorsMiddleware::new()
    //     .allow_methods("GET, POST, PUT, OPTIONS".parse::<HeaderValue>().unwrap())
    //     .allow_origin(Origin::from("*"))
    //     .allow_credentials(false);
    //
    // app.with(cors);
    //
    // app.at("/").serve_dir("dist/")?;
    // app.at("/health-check")
    //     .get(|_req: Request<State>| async move { Ok(Response::new(StatusCode::Ok)) });
    // app.at("/user").post(user::create_user);
    // app.at("/user/login").post(user::login);
    //
    // app.at("/user/:id")
    //     .with(auth::authorize)
    //     .get(user::get_user)
    //     .put(user::update_user)
    //     .delete(user::delete_user);
}
