use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use tide::http::headers::HeaderValue;
use tide::http::StatusCode;
use tide::security::{CorsMiddleware, Origin};
use tide::{Request, Response};
use uuid::Uuid;

mod error;
pub use error::*;

mod settings;
use settings::Settings;

mod endpoints;
use endpoints::user;

#[cfg(test)]
mod tests;

pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

#[derive(Debug, Deserialize, Serialize)]
struct JsonBody<T> {
    data: T,
}

impl<T> JsonBody<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

pub type Token = String;

#[derive(Debug, Deserialize, Serialize)]
struct RawUserData {
    username: String,
    password: String,
}

struct ValidUserData(RawUserData);

impl ValidUserData {
    pub fn parse(create_user: RawUserData) -> Result<Self> {
        // TODO: Add some validation logic.
        let RawUserData { username, password } = create_user;

        Ok(Self(RawUserData { username, password }))
    }
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
struct User {
    id: Uuid,
    username: String,
    hashed_password: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserData {
    id: Uuid,
    username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserCreationData {
    id: Uuid,
    token: String,
}

#[derive(Clone, Debug)]
pub struct State {
    db_pool: PgPool,
    settings: Settings,
}

async fn configure_app(db_pool: PgPool, settings: Settings) -> Result<tide::Server<State>> {
    let state = State { db_pool, settings };

    let mut app = tide::with_state(state);

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, PUT, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);

    app.with(tide::log::LogMiddleware::new());
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    pretty_env_logger::try_init().ok();

    app.at("/").serve_dir("dist/")?;
    app.at("/health-check")
        .get(|_req: Request<State>| async move { Ok(Response::new(StatusCode::Ok)) });
    app.at("/user").post(user::create_user);
    app.at("/user/:id").get(user::get_user);
    app.at("/user/login").post(user::login);

    Ok(app)
}

#[async_std::main]
async fn main() -> Result<()> {
    //tide::log::start();
    let settings = Settings::new()?;
    let address = format!("{}:{}", settings.app.host, settings.app.port);

    let db_pool = PgPool::connect(&settings.database.url()).await?;

    let app = configure_app(db_pool, settings).await?;

    println!("Server listening on {}", address);
    app.listen(address).await?;

    Ok(())
}
