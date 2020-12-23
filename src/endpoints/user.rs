use argonautica::{Hasher, Verifier};
use chrono::prelude::*;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sqlx::PgPool;
use std::env;
use tide::{http::StatusCode, Request, Response};
use uuid::Uuid;

use crate::{RawUserData, State, User, ValidUserData};

pub async fn create_user(mut req: Request<State>) -> tide::Result {
    // Only cloning an Arc here so no real costs involved.
    let pool = &req.state().db_pool.clone();

    let raw: RawUserData = req.body_json().await?;
    let valid_user_data = ValidUserData::parse(raw).expect("Failed to parse valid user.");

    let new_user = insert_new_user(pool, valid_user_data).await?;
    let json = serde_json::to_string(&new_user)?;

    let _token = insert_auth_token(pool, &new_user.id).await?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(json);
    Ok(res)
}

async fn insert_new_user(pool: &PgPool, user_data: ValidUserData) -> Result<User, sqlx::Error> {
    let id = Uuid::new_v4();
    let date = Utc::now();

    let ValidUserData(RawUserData { username, password }) = user_data;

    let secret = env::var("SECRET").expect("Failed to read env var.");
    let mut hasher = Hasher::default();
    let hash = hasher
        .with_password(&password)
        .with_secret_key(secret)
        .hash()
        .expect("Failed to hash password.");

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
        hash,
        date,
        date,
    )
    .fetch_one(pool)
    .await
}

async fn insert_auth_token(pool: &PgPool, user_id: &Uuid) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4();
    let token = rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>();

    let record = sqlx::query!(
        r#"
        INSERT INTO auth_tokens ( id, user_id, token) VALUES ( $1, $2, $3)
        RETURNING token
        "#,
        id,
        user_id,
        token,
    )
    .fetch_one(pool)
    .await?;

    Ok(record.token)
}

pub async fn login(mut req: Request<State>) -> tide::Result {
    let pool = &req.state().db_pool.clone();

    let RawUserData { username, password } = req.body_json().await?;

    let row = sqlx::query!(
        r#"
        select id as user_id, hashed_password from users
        where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .await?;

    let is_valid = verify_password(&row.hashed_password, &password);

    if !is_valid {
        let res = Response::new(StatusCode::Unauthorized);
        return Ok(res);
    }

    let row = sqlx::query!(
        r#"
        select token from auth_tokens
        where user_id = $1
        "#,
        row.user_id,
    )
    .fetch_one(pool)
    .await?;

    let token = serde_json::to_string(&row.token)?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(token);
    Ok(res)
}

fn verify_password(hash: &str, password: &str) -> bool {
    let secret = env::var("SECRET").expect("Failed to read env var.");
    let mut verifier = Verifier::default();
    verifier
        .with_hash(&hash)
        .with_password(password)
        .with_secret_key(secret)
        .verify()
        .expect("Failed to verify hash.")
}
