use argonautica::{Hasher, Verifier};
use async_std::task;
use chrono::prelude::*;
use sqlx::PgPool;
use std::convert::TryInto;
use tide::{http::StatusCode, Request, Response};
use tracing::{debug, debug_span, error, info, Instrument, instrument};
use uuid::Uuid;

use crate::{
    JsonBody, RawUserData, ServiceError, State, User, UserCreationData, UserData, ValidUserData, telemetry::LogInfo
};

use crate::auth::UserId;

#[instrument(level = "debug", skip(req))]
pub async fn create_user(mut req: Request<State>) -> tide::Result {
    let log_info = LogInfo::from_req(&req);
    debug!("Request received. {}", &log_info);

    // Only cloning an Arc here so no real costs involved.
    let pool = &req.state().db_pool.clone();

    let settings = &req.state().settings.clone();
    let secret = settings.app.secret.as_ref();

    let raw: RawUserData = req.body_json().await?;
    let valid_user_data: ValidUserData = raw.try_into()?;
    debug!("{} Parsed valid username: {:?}", &log_info, &valid_user_data.0.username);

    let user = insert_new_user(pool, valid_user_data, secret, &log_info).instrument(debug_span!("insert_new_user")).await?;
    let token = insert_auth_token(pool, &user.id, &log_info).instrument(debug_span!("insert_auth_token")).await?;
    debug!("{} Inserted user with id {:?}", &log_info, user.id);

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

    info!("{} Successfully created user with id {}", &log_info, &user.id);
    // let _ = span_guard;
    Ok(res)
}

async fn insert_new_user(
    pool: &PgPool,
    user_data: ValidUserData,
    secret: &str,
    log_info: &LogInfo,
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
    .await
    .map_err(|err| {
        error!("Failed to execute query: {:?}", err);
        err
    });

    debug!("{} Inserted user into DB.", log_info);
    res
}

async fn insert_auth_token(pool: &PgPool, user_id: &Uuid, log_info: &LogInfo) -> Result<String, sqlx::Error> {
    let token_id = Uuid::new_v4();

    let token = crate::auth::create(UserId::new(*user_id), "User").unwrap();

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
    .await
    .map_err(|err| {
        error!("Failed to execute query: {:?}", err);
        err
    })?;

    debug!("{} Inserted token into DB.", log_info);
    Ok(record.token)
}

#[instrument(level = "debug", skip(req))]
pub async fn login(mut req: Request<State>) -> tide::Result {
    let log_info = LogInfo::from_req(&req);
    debug!("Request received. {}", &log_info);

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
            debug!("{} Found matching user_id {}", &log_info, &row.user_id);
            (row.user_id, row.hashed_password)
        },
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                error!("{} {:?}", &log_info, err);
                return Ok(Response::new(StatusCode::Unauthorized));
            }
            _ => {
                error!("{} {:?}", &log_info, err);
                return Ok(Response::new(StatusCode::InternalServerError));
            }
        },
    };

    let is_valid = verify_password(&hashed_password, &password, secret)?;

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
    debug!("{} Successfully logged in.", &log_info);
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

#[instrument(level = "debug", skip(req))]
pub async fn get_user(req: Request<State>) -> tide::Result {
    let log_info = LogInfo::from_req(&req);
    debug!("Request received. {}", &log_info);

    let pool = &req.state().db_pool.clone();
    let user_id: &UserId = req.ext().expect("Failed to extract token from request.");

    let user = sqlx::query_as!(
        User,
        r#"
            select * from users where id = $1;
        "#,
        user_id.take(),
    )
    .fetch_one(pool)
    .await;

    let user_data = match user {
        Err(err) => match err {
            // Requested user doesn't exist, e.g. token must be illegal.
            sqlx::Error::RowNotFound => {
                error!("{} {:?}", &log_info, err);
                return Ok(Response::new(StatusCode::Forbidden))
            },
            // Any other sqlx error.
            _ => {
                error!("{} {:?}", &log_info, err);
                return Ok(Response::new(StatusCode::InternalServerError))
            },
        },
        Ok(user) => UserData {
            id: user.id,
            username: user.username,
        },
    };

    let json = serde_json::to_string(&JsonBody::new(user_data))?;

    let mut res = Response::new(StatusCode::Ok);
    res.set_body(json);
    debug!("{} Successfully got user.", &log_info);
    Ok(res)
}

#[instrument(level = "debug", skip(req))]
pub async fn update_user(mut req: Request<State>) -> tide::Result {
    let log_info = LogInfo::from_req(&req);
    debug!("Request received. {}", &log_info);

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

    debug!("{} Successfully updated user.", &log_info);
    Ok(Response::builder(200)
        .body(serde_json::to_string(&JsonBody::new(updated_user))?)
        .build())
}

#[instrument(level = "debug", skip(req))]
pub async fn delete_user(req: Request<State>) -> tide::Result {
    let log_info = LogInfo::from_req(&req);
    debug!("Request received. {}", &log_info);

    // TODO: In middleware, check if token ok && id == token.user_id
    let user_id = Uuid::parse_str(req.param("id")?)?;

    let pool = &req.state().db_pool.clone();

    sqlx::query!(r#" delete from auth_tokens where user_id = $1; "#, user_id)
        .execute(pool)
        .await?;

    sqlx::query!(r#" delete from users where id = $1; "#, user_id)
        .execute(pool)
        .await?;

    debug!("{} Successfully deleted user.", &log_info);
    Ok(Response::builder(200).build())
}
