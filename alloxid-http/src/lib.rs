use std::sync::Arc;

use axum::body::Body;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Router};
use http::{Method, Request, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Origin};
use tower_http::trace::TraceLayer;
use tracing::debug;
use uuid::Uuid;

pub mod error;
pub mod model;
pub mod settings;
pub mod telemetry;

mod auth;
mod database;
mod endpoints;
mod helpers;

use endpoints::grpc;
use endpoints::user;
use error::*;
use settings::Settings;

pub type Result<T, E = ServiceError> = std::result::Result<T, E>;
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
    // debug!("Health check called");
    "Hello, healthy world!"
}

async fn handle_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        r#"Nothing to see here.

Navigate to /health-check to receive a 200 response.

Run tests to see the app in action.
"#,
    )
}

pub async fn configure_app(db_pool: PgPool, settings: Settings) -> Result<axum::Router> {
    let cors = CorsLayer::new()
        .allow_origin(Origin::exact(
            settings
                .app
                .cors_url
                .parse()
                .expect("Failed to parse frontend url"),
        ))
        .allow_methods(vec![Method::GET, Method::POST]);

    let state = Arc::new(State { db_pool, settings });

    let service = ServiceBuilder::new()
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http().make_span_with(
            |_req: &Request<Body>| tracing::debug_span!("request", req_id = %Uuid::new_v4()),
        ))
        .layer(cors);

    let grpc_routes = Router::new().route("/hello", get(grpc::hello));

    let app = Router::new()
        // .route("/", get(root))
        .route("/health-check", get(health_check))
        .route("/user", post(user::create))
        .route("/user/login", post(user::login))
        .route(
            "/user/:id",
            get(user::get).put(user::update).delete(user::delete),
        )
        .nest("/grpc", grpc_routes)
        .layer(service);

    let app = app.fallback(handle_404.into_service());

    Ok(app)
}
