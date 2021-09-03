use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::convert::TryFrom;
use tide::http::headers::HeaderValue;
use tide::http::StatusCode;
use tide::security::{CorsMiddleware, Origin};
use tide::{Request, Response};
use uuid::Uuid;

pub mod error;
pub mod settings;

mod auth;
mod endpoints;

use endpoints::user;
use error::*;
use settings::Settings;

pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;
pub type ServiceResult<T = tide::Response, E = anyhow::Error> = std::result::Result<T, E>;

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonBody<T> {
    pub data: T,
}

impl<T> JsonBody<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RawUserData {
    pub username: String,
    pub password: String,
}

pub struct ValidUserData(RawUserData);

impl TryFrom<RawUserData> for ValidUserData {
    type Error = anyhow::Error;

    fn try_from(value: RawUserData) -> Result<Self, Self::Error> {
        // TODO: Add some validation logic.
        let RawUserData { username, password } = value;

        Ok(Self(RawUserData { username, password }))
    }
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub hashed_password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct UserData {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserCreationData {
    pub id: Uuid,
    pub token: String,
}

#[derive(Clone, Debug)]
pub struct State {
    pub db_pool: PgPool,
    pub settings: Settings,
}

pub async fn configure_app(db_pool: PgPool, settings: Settings) -> Result<tide::Server<State>> {
    let state = State { db_pool, settings };

    let mut app = tide::with_state(state);

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, PUT, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);

    app.with(tide::log::LogMiddleware::new());
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "WARN");
    }
    pretty_env_logger::try_init().ok();

    app.at("/").serve_dir("dist/")?;
    app.at("/health-check")
        .get(|_req: Request<State>| async move { Ok(Response::new(StatusCode::Ok)) });
    app.at("/user").post(user::create_user);
    app.at("/user/login").post(user::login);

    app.at("/user/:id")
        .with(auth::authorize)
        .get(user::get_user)
        .put(user::update_user)
        .delete(user::delete_user);

    Ok(app)
}
