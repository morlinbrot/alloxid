use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use uuid::Uuid;
//use std::future::Future;
//use std::pin::Pin;
use tide::http::headers::HeaderValue;
use tide::http::StatusCode;
use tide::security::{CorsMiddleware, Origin};
use tide::{Request, Response};

mod settings;
use settings::Settings;

mod endpoints;
//use endpoints::todo;
use endpoints::user;

#[cfg(test)]
mod tests;

// mod log_middleware;
//use log_middleware::logger;

#[derive(Debug, Deserialize, Serialize)]
struct RawTodo {
    text: String,
    completed: bool,
}

struct ValidRawTodo(RawTodo);

impl ValidRawTodo {
    // TODO: Errors.
    pub fn parse(raw: RawTodo) -> Result<Self, String> {
        let RawTodo { text, completed } = raw;
        // TODO: Add some validation logic.
        Ok(Self(RawTodo { text, completed }))
    }
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

#[allow(dead_code)]
impl Todo {
    pub(crate) fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct RawUserData {
    username: String,
    password: String,
}

struct ValidUserData(RawUserData);

impl ValidUserData {
    // TODO: Errors.
    pub fn parse(create_user: RawUserData) -> Result<Self, String> {
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

#[derive(Clone, Debug)]
pub struct State {
    db_pool: PgPool,
}

async fn configure_app(db_pool: PgPool) -> Result<tide::Server<State>, std::io::Error> {
    let state = State { db_pool };

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

    //app.at("/").get(|_| async {
    //    Ok(format!(
    //        "
    //Try one of these routes:\n
    //GET   /all      - get all todos
    //POST  /todo     - create a new todo
    //GET   /todo/:id - get a single todo
    //        "
    //    ))
    //})
    app.at("/").serve_dir("dist/")?;
    app.at("/health-check")
        .get(|_req: Request<State>| async move { Ok(Response::new(StatusCode::Ok)) });
    // app.at("/todo").post(todo::create_todo);
    // app.at("/todo/all").get(todo::get_all);
    // app.at("/todo/:id").get(todo::get_todo);
    app.at("/user").post(user::create_user);

    Ok(app)
}

#[async_std::main]
async fn main() -> Result<(), sqlx::Error> {
    //tide::log::start();
    let Settings { app, database } = Settings::new().expect("Failed to load configuration.");
    let address = format!("{}:{}", app.host, app.port);

    let db_pool = PgPool::connect(&database.url())
        .await
        .expect("Failed to create db pool.");

    let app = configure_app(db_pool)
        .await
        .expect("Failed to configure app.");

    println!("Server listening on {}", address);
    app.listen(address).await?;

    Ok(())
}
