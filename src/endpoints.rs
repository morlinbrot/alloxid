use tide::{http::StatusCode, Request, Response};

use crate::{State, Todo};

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
