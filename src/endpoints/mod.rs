use chrono::prelude::*;
use sqlx::PgPool;
use tide::{http::StatusCode, Request, Response};
use uuid::Uuid;

use crate::{RawUserData, State, Todo, User, ValidUserData};

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

pub async fn create_user(mut req: Request<State>) -> tide::Result {
    // Only cloning an Arc here so no real costs involved.
    let pool = &req.state().db_pool.clone();

    let create_user: RawUserData = req.body_json().await?;

    let valid_user_data = ValidUserData::parse(create_user).expect("Failed to parse valid user.");

    let new_user = insert_new_user(pool, valid_user_data).await?;
    let json = serde_json::to_string(&new_user)?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(json);
    Ok(res)
}

async fn insert_new_user(pool: &PgPool, user_data: ValidUserData) -> Result<User, sqlx::Error> {
    let ValidUserData(RawUserData {
        username,
        password: hashed_password,
    }) = user_data;

    let id = Uuid::new_v4();
    let date = Utc::now();

    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (
            id,
            username,
            hashed_password,
            created_at,
            updated_at
        ) VALUES ( $1, $2, $3, $4, $5)
        RETURNING *
        "#,
        id,
        username,
        hashed_password,
        date,
        date,
    )
    .fetch_one(pool)
    .await
}
