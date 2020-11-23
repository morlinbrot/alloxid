use tide::{http::StatusCode, Request, Response};
use uuid::Uuid;

use crate::{CreateUser, State, Todo, User};

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
    match sqlx::query_as!(Todo, "SELECT * from todos WHERE id = $1", id)
        .fetch_one(pool)
        .await
    {
        Ok(todo) => {
            let body = serde_json::to_string(&todo)?;
            let mut res = Response::new(StatusCode::Ok);
            res.set_body(body);
            Ok(res)
        }
        Err(err) => {
            println!("Error: {}", err);
            Ok(Response::new(StatusCode::NotFound))
        }
    }
}

pub async fn new_todo(mut req: Request<State>) -> tide::Result {
    let todo: Todo = req.body_json().await?;
    let body = serde_json::to_string(&todo)?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(body);
    Ok(res)
}

pub async fn new_user(mut req: Request<State>) -> tide::Result {
    // Only cloning an Arc here so no real costs involved.
    let pool = &req.state().db_pool.clone();

    let CreateUser { username } = req.body_json().await?;
    let id = Uuid::new_v4();

    // TODO: Make this return a User directly.
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id, username) VALUES ( $1, $2 )
        RETURNING *
        "#,
        id,
        username,
    )
    .fetch_one(pool)
    .await?;

    let body = serde_json::to_string(&user)?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(body);
    Ok(res)
}
