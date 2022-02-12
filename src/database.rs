use argonautica::Hasher;
use async_std::task;
use chrono::prelude::*;
use sqlx::PgPool;
use tracing::{debug, debug_span, error, Instrument};
use uuid::Uuid;

use crate::auth::UserId;
use crate::model::user::{RawUserData, User, ValidUserData};

pub async fn insert_new_user(
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

pub async fn insert_auth_token(pool: &PgPool, user_id: &Uuid) -> Result<String, sqlx::Error> {
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
