use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ServiceError;

#[derive(Debug, Deserialize, Serialize)]
pub struct RawUserData {
    pub username: String,
    pub password: String,
}

// Newtype pattern.
pub struct ValidUserData(pub RawUserData);

impl TryFrom<RawUserData> for ValidUserData {
    type Error = ServiceError;

    fn try_from(value: RawUserData) -> Result<Self, Self::Error> {
        // TODO: Add some validation logic.
        let RawUserData { username, password } = value;

        Ok(Self(RawUserData { username, password }))
    }
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub hashed_password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct UserData {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserCreationData {
    pub id: Uuid,
    pub token: String,
}
