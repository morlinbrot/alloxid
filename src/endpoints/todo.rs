use sqlx::PgPool;
use tide::{http::StatusCode, Request, Response};

use crate::{RawTodo, State, Todo, ValidRawTodo};

pub async fn get_all(req: Request<State>) -> tide::Result {
    let pool = &req.state().db_pool;

    let todos = sqlx::query_as!(Todo, "SELECT * from todos")
        .fetch_all(pool)
        .await?;

    let body = serde_json::to_string(&todos)?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(body);
    Ok(res)
}

pub async fn get_todo(req: Request<State>) -> tide::Result {
    let pool = &req.state().db_pool;

    let id: i32 = req.param("id")?;
    let todo = sqlx::query_as!(Todo, "SELECT * from todos WHERE id = $1", id)
        .fetch_one(pool)
        .await?;

    let body = serde_json::to_string(&todo)?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(body);
    Ok(res)
}

pub async fn create_todo(mut req: Request<State>) -> tide::Result {
    let pool = &req.state().db_pool.clone();

    let raw: RawTodo = req.body_json().await?;
    let valid_raw_todo = ValidRawTodo::parse(raw).expect("Failed to parse valid todo.");

    let todo = insert_new_todo(pool, valid_raw_todo).await?;
    let json = serde_json::to_string(&todo)?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(json);
    Ok(res)
}

async fn insert_new_todo(pool: &PgPool, todo: ValidRawTodo) -> Result<Todo, sqlx::Error> {
    let ValidRawTodo(RawTodo { text, completed }) = todo;

    sqlx::query_as!(
        Todo,
        r#"
        INSERT INTO todos (text, completed) VALUES ($1, $2) RETURNING *
        "#,
        text,
        completed,
    )
    .fetch_one(pool)
    .await
}
