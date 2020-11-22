use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
//use std::future::Future;
//use std::pin::Pin;
use tide::http::headers::HeaderValue;
use tide::http::StatusCode;
use tide::security::{CorsMiddleware, Origin};
use tide::{Request, Response};

mod settings;
use settings::Settings;

mod endpoints;
use endpoints::*;

#[cfg(test)]
mod tests;

//mod log_middleware;
//use log_middleware::logger;

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

#[derive(Clone, Debug)]
pub struct State {
    db_pool: PgPool,
}

async fn configure_app() -> Result<tide::Server<State>, std::io::Error> {
    let Settings { database, .. } = Settings::new().expect("Failed to load configuration.");

    let db_pool = PgPool::new(&database.url)
        .await
        .expect("Failed to create db pool.");

    let state = State {
        db_pool: db_pool.clone(),
    };

    let mut app = tide::with_state(state);

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, PUT, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);
    //app.with(logger);

    //app.at("/").get(|_| async {
    //    Ok(format!(
    //        "
    //Try one of these routes:\n
    //GET   /all      - get all todos
    //POST  /todo     - create a new todo
    //GET   /todo/:id - get a single todo
    //        "
    //    ))
    //});
    app.at("/").serve_dir("dist/")?;
    app.at("/health-check")
        .get(|_req: Request<State>| async move { Ok(Response::new(StatusCode::Ok)) });
    app.at("/api/all").get(get_all);
    app.at("/api/todo").post(new_todo);
    app.at("/api/todo/:id").get(get_todo);

    #[cfg(not(test))]
    tide::log::start();

    Ok(app)
}

#[async_std::main]
async fn main() -> Result<(), sqlx::Error> {
    //tide::log::start();
    let Settings { app, .. } = Settings::new().expect("Failed to load configuration.");
    let address = format!("{}:{}", app.host, app.port);

    let app = configure_app().await.expect("Failed to configure app.");

    println!("Server listening on {}", address);
    app.listen(address).await?;

    Ok(())
}
