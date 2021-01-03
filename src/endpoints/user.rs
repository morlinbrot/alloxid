use argonautica::{Hasher, Verifier};
use async_std::task;
use chrono::prelude::*;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sqlx::PgPool;
use tide::{http::StatusCode, Request, Response};
use uuid::Uuid;

use crate::{JsonBody, RawUserData, State, User, UserCreationData, UserData, ValidUserData};

pub async fn create_user(mut req: Request<State>) -> tide::Result {
    // Only cloning an Arc here so no real costs involved.
    let pool = &req.state().db_pool.clone();
    let secret = &req.state().settings.clone().app.secret;

    let raw: RawUserData = req.body_json().await?;
    let valid_user_data = ValidUserData::parse(raw)?;

    let user = insert_new_user(pool, valid_user_data, secret).await?;
    let token = insert_auth_token(pool, &user.id).await?;
    let data = UserCreationData { token, id: user.id };

    let json = serde_json::to_string(&JsonBody::new(data))?;

    let mut res = Response::new(StatusCode::Created);
    res.set_body(json);
    Ok(res)
}

async fn insert_new_user(
    pool: &PgPool,
    user_data: ValidUserData,
    secret: &str,
) -> Result<User, sqlx::Error> {
    let id = Uuid::new_v4();
    let date = Utc::now();
    let secret = secret.to_string();

    let ValidUserData(RawUserData { username, password }) = user_data;

    // Since the hashing actually takes some time, we're offloading it onto a dedicated thread pool for blocking tasks.
    let hash = task::spawn_blocking(move || {
        let mut hasher = Hasher::default();
        hasher.configure_iterations(192);

        #[cfg(test)]
        hasher.configure_iterations(10);

        hasher
            .with_password(&password)
            .with_secret_key(secret)
            .hash()
            .expect("Failed to hash password.")
    })
    .await;

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
    let secret = &req.state().settings.clone().app.secret;

    let RawUserData { username, password } = req.body_json().await?;

    let row = sqlx::query!(
        r#"
        select id as user_id, hashed_password from users
        where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .await;

    let (user_id, hashed_password) = match row {
        Ok(row) => (row.user_id, row.hashed_password),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                return Ok(Response::new(StatusCode::Unauthorized));
            }
            _ => {
                return Ok(Response::new(StatusCode::InternalServerError));
            }
        },
    };

    let is_valid = verify_password(&hashed_password, &password, secret)?;

    if !is_valid {
        let res = Response::new(StatusCode::Unauthorized);
        return Ok(res);
    }

    let row = sqlx::query!(
        r#"
        select token from auth_tokens
        where user_id = $1
        "#,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    let json = serde_json::to_string(&JsonBody::new(row.token))?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(json);
    Ok(res)
}

fn verify_password(hash: &str, password: &str, secret: &str) -> crate::Result<bool> {
    let mut verifier = Verifier::default();
    Ok(verifier
        .with_hash(&hash)
        .with_password(password)
        .with_secret_key(secret)
        .verify()
        .map_err(|err| crate::Error::from(err))?)
}

pub async fn get_user(req: Request<State>) -> tide::Result {
    // TODO: Create middleware to do this.
    let token = match req.header("Authentication") {
        Some(token) => token.as_str().to_string(),
        None => {
            return Ok(Response::new(StatusCode::Unauthorized));
        }
    };

    // TODO: In middleware, check if token ok && id == token.user_id
    let user_id = Uuid::parse_str(req.param("id")?)?;

    let pool = &req.state().db_pool.clone();

    let user = sqlx::query_as!(
        User,
        r#"
        select u.* from users u
        join auth_tokens a on a.user_id = u.id
        where a.token = $1
        "#,
        token,
    )
    .fetch_one(pool)
    .await;

    let user_data = match user {
        Err(err) => match err {
            // Requested user doesn't exist, e.g. token must be illegal.
            sqlx::Error::RowNotFound => return Ok(Response::new(StatusCode::Forbidden)),
            // Any othe sqlx error.
            _ => return Ok(Response::new(StatusCode::InternalServerError)),
        },
        // We found a user matching the token but requested id doesn't match, e.g. illegal token.
        Ok(user) if user.id != user_id => return Ok(Response::new(StatusCode::Forbidden)),
        Ok(user) => UserData {
            id: user.id,
            username: user.username,
        },
    };

    let json = serde_json::to_string(&JsonBody::new(user_data))?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(json);
    Ok(res)
}

pub async fn update_user(mut req: Request<State>) -> tide::Result {
    // TODO: Create middleware to do this.
    let _token = match req.header("Authentication") {
        Some(token) => token.as_str().to_string(),
        None => {
            return Ok(Response::new(StatusCode::Unauthorized));
        }
    };

    let pool = &req.state().db_pool.clone();
    // TODO: In middleware, check if token ok && id == token.user_id
    let user_id = Uuid::parse_str(req.param("id")?)?;

    let mut patch: serde_json::Value = req.body_json().await?;
    let username = patch.get_mut("username").expect("Failed to parse json.");
    let username: String = serde_json::from_value(username.take())?;

    let updated_user = sqlx::query_as!(
        UserData,
        r#"
            update users
            set username = $2
            where id = $1
            returning id, username
            "#,
        user_id,
        username,
    )
    .fetch_one(pool)
    .await?;

    Ok(Response::builder(200)
        .body(serde_json::to_string(&JsonBody::new(updated_user))?)
        .build())
}
