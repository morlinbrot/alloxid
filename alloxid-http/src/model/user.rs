use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ServiceError;

// Input to the create endpoint.
#[derive(Debug, Deserialize, Serialize)]
pub struct UserCreateRaw {
    pub username: String,
    pub password: String,
}

// Returned by the create and login endpoints.
#[derive(Debug, Deserialize, Serialize)]
pub struct UserAuthData {
    pub id: Uuid,
    pub token: String,
}

// Newtype pattern.
pub struct ValidUserData(pub UserCreateRaw);

impl TryFrom<UserCreateRaw> for ValidUserData {
    type Error = ServiceError;

    fn try_from(value: UserCreateRaw) -> Result<Self, Self::Error> {
        // TODO: Add some validation logic.
        let UserCreateRaw { username, password } = value;

        Ok(Self(UserCreateRaw { username, password }))
    }
}

// The full user as it is stored in the db.
#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub(crate) struct UserEntry {
    pub id: Uuid,
    pub username: String,
    pub hashed_password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// The public user data.
#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct UserData {
    pub id: Uuid,
    pub username: String,
}

// Input to the update endpoint.
#[derive(Debug, Deserialize, Serialize)]
pub struct UserUpdateRaw {
    pub username: String,
}
