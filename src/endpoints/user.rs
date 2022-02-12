use argonautica::{Hasher, Verifier};
use async_std::task;
use chrono::prelude::*;
use sqlx::PgPool;
use std::convert::TryInto;
use tide::{http::StatusCode, Request, Response};
use tracing::{debug, debug_span, error, info, instrument, Instrument};
use uuid::Uuid;

use crate::auth::UserId;
use crate::model::user::{RawUserData, User, UserCreationData, UserData, ValidUserData};
use crate::{JsonBody, ServiceError, State};

#[instrument(level = "info", skip(req), fields(
    app_port = req.state().settings.app.port,
    db_name = %req.state().settings.database.name.clone(),
    req_id = %Uuid::new_v4(),
))]
pub async fn create_user(mut req: Request<State>) -> tide::Result {
    debug!("Request received");

    // Only cloning an Arc here so no real costs involved.
    let pool = &req.state().db_pool.clone();

    let settings = &req.state().settings.clone();
    let secret = settings.app.secret.as_ref();

    let raw: RawUserData = req.body_json().await?;
    let valid_user_data: ValidUserData = raw.try_into().map_err(|err| {
        error!("Err: {:?}", err);
        err
    })?;

    let user = insert_new_user(pool, valid_user_data, secret)
        .instrument(debug_span!("insert_new_user"))
        .await?;
    let token = insert_auth_token(pool, &user.id)
        .instrument(debug_span!("insert_auth_token"))
        .await?;

    let data = UserCreationData { token, id: user.id };
    let json = serde_json::to_string(&JsonBody::new(data))?;

    let location = format!(
        "{}:{}/user/{}",
        settings.app.host, settings.app.port, user.id
    );

    let res = Response::builder(StatusCode::Created)
        .body(json)
        .header("Location", location)
        .build();

    info!("Successfully created user with id {}", &user.id);
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

    let user_span = debug_span!("user_span");
    let res = sqlx::query_as!(
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
    .instrument(user_span)
    .await
    .map_err(|err| {
        error!("Err: {:?}", err);
        err
    });

    debug!("Inserted user into DB for user_id={}.", id);
    res
}

async fn insert_auth_token(pool: &PgPool, user_id: &Uuid) -> Result<String, sqlx::Error> {
    let token_id = Uuid::new_v4();

    let token = crate::auth::create(UserId::new(*user_id), "User").unwrap();

    let token_span = debug_span!("token_span");
    let record = sqlx::query!(
        r#"
            INSERT INTO auth_tokens ( id, user_id, token) VALUES ( $1, $2, $3)
            RETURNING token
        "#,
        token_id,
        user_id,
        token,
    )
    .fetch_one(pool)
    .instrument(token_span)
    .await
    .map_err(|err| {
        error!("Err: {:?}", err);
        err
    })?;

    debug!("Inserted token into DB for user_id={}", user_id);
    Ok(record.token)
}

#[instrument(level = "info", skip(req), fields(
    app_port = req.state().settings.app.port,
    db_name = %req.state().settings.database.name.clone(),
    req_id = %Uuid::new_v4(),
))]
pub async fn login(mut req: Request<State>) -> tide::Result {
    debug!("Request received");

    let pool = &req.state().db_pool.clone();
    let secret = &req.state().settings.clone().app.secret;

    let RawUserData { username, password } = req.body_json().await?;

    let query_user_span = debug_span!("query_user_span");
    let row = sqlx::query!(
        r#"
            select id as user_id, hashed_password from users
            where username = $1
        "#,
        username,
    )
    .fetch_one(pool)
    .instrument(query_user_span)
    .await;

    let (user_id, hashed_password) = match row {
        Ok(row) => {
            debug!("Found matching user_id {}", &row.user_id);
            (row.user_id, row.hashed_password)
        }
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                error!("Err: {:?}", err);
                return Ok(Response::new(StatusCode::Unauthorized));
            }
            _ => {
                error!("Err: {:?}", err);
                return Ok(Response::new(StatusCode::InternalServerError));
            }
        },
    };

    let is_valid = verify_password(&hashed_password, &password, secret)?;
    debug!(is_valid);

    if !is_valid {
        let res = Response::new(StatusCode::Unauthorized);
        return Ok(res);
    }

    let query_token_span = debug_span!("query_token_span");
    let row = sqlx::query!(
        r#"
            select token from auth_tokens
            where user_id = $1
        "#,
        user_id,
    )
    .fetch_one(pool)
    .instrument(query_token_span)
    .await?;

    let json = serde_json::to_string(&JsonBody::new(row.token))?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(json);

    info!("Successfully logged in user_id={}", user_id);
    Ok(res)
}

fn verify_password(hash: &str, password: &str, secret: &str) -> crate::Result<bool> {
    let mut verifier = Verifier::default();
    Ok(verifier
        .with_hash(&hash)
        .with_password(password)
        .with_secret_key(secret)
        .verify()
        .map_err(|err| ServiceError::from(err))?)
}

#[instrument(level = "info", skip(req), fields(
    app_port = req.state().settings.app.port,
    db_name = %req.state().settings.database.name.clone(),
    req_id = %Uuid::new_v4(),
))]
pub async fn get_user(req: Request<State>) -> tide::Result {
    debug!("Request received");

    let pool = &req.state().db_pool.clone();
    let user_id: &UserId = req.ext().expect("Failed to extract token from request.");

    let query_span = debug_span!("query_span");
    let user = sqlx::query_as!(
        User,
        r#"
            select * from users where id = $1;
        "#,
        user_id.take(),
    )
    .fetch_one(pool)
    .instrument(query_span)
    .await;

    let user_data = match user {
        Err(err) => match err {
            // Requested user doesn't exist, e.g. token must be illegal.
            sqlx::Error::RowNotFound => {
                error!("Err: {:?}", err);
                return Ok(Response::new(StatusCode::Forbidden));
            }
            // Any other sqlx error.
            _ => {
                error!("Err: {:?}", err);
                return Ok(Response::new(StatusCode::InternalServerError));
            }
        },
        Ok(user) => {
            debug!("Found user id={} username={}", user.id, user.username);
            UserData {
                id: user.id,
                username: user.username,
            }
        }
    };

    let json = serde_json::to_string(&JsonBody::new(user_data))?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(json);

    info!("Successfully got user_id={:?}", user_id);
    Ok(res)
}

#[instrument(level = "info", skip(req), fields(
    app_port = req.state().settings.app.port,
    db_name = %req.state().settings.database.name.clone(),
    req_id = %Uuid::new_v4(),
))]
pub async fn update_user(mut req: Request<State>) -> tide::Result {
    debug!("Request received");

    // TODO: In middleware, check if token ok && id == token.user_id
    let pool = &req.state().db_pool.clone();

    let user_id = Uuid::parse_str(req.param("id")?)?;

    let mut patch: serde_json::Value = req.body_json().await?;
    let username = patch.get_mut("username").expect("Failed to parse json.");
    let username: String = serde_json::from_value(username.take())?;

    let query_span = debug_span!("query_span");
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
    .instrument(query_span)
    .await?;

    info!("Successfully updated user_id={}", user_id);
    Ok(Response::builder(200)
        .body(serde_json::to_string(&JsonBody::new(updated_user))?)
        .build())
}

#[instrument(level = "info", skip(req), fields(
    app_port = req.state().settings.app.port,
    db_name = %req.state().settings.database.name.clone(),
    req_id = %Uuid::new_v4(),
))]
pub async fn delete_user(req: Request<State>) -> tide::Result {
    debug!("Request received");

    // TODO: In middleware, check if token ok && id == token.user_id
    let user_id = Uuid::parse_str(req.param("id")?)?;

    let pool = &req.state().db_pool.clone();

    sqlx::query!(r#" delete from auth_tokens where user_id = $1; "#, user_id)
        .execute(pool)
        .await?;

    sqlx::query!(r#" delete from users where id = $1; "#, user_id)
        .execute(pool)
        .await?;

    info!("Successfully deleted user_id={}", user_id);
    Ok(Response::builder(200).build())
}
