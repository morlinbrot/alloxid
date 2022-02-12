use chrono::prelude::*;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

pub mod middleware;
pub mod middleware_ax;
pub(crate) use middleware::*;

use crate::error::ServiceError;

pub const AUTHORIZATION: &str = "Authorization";
pub const BEARER: &str = "Bearer ";
pub const SECRET: &[u8] = b"totally secret";

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn take(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    // Subject of the token
    sub: UserId,
    // Custom data point.
    role: String,
    // Expiration date.
    exp: usize,
}

#[derive(Clone, PartialEq)]
pub enum Role {
    Admin,
    User,
}

impl Role {
    pub fn from_str(role: &str) -> Self {
        match role {
            "Admin" => Self::Admin,
            _ => Self::User,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Admin => write!(f, "Admin"),
            Self::User => write!(f, "User"),
        }
    }
}

pub fn create(user_id: UserId, role: &str) -> Result<String, ServiceError> {
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("Failed to create valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        role: role.to_string(),
        exp: exp as usize,
    };

    let header = Header::new(Algorithm::HS512);

    encode(&header, &claims, &EncodingKey::from_secret(SECRET))
        .map_err(|_| ServiceError::TokenCreationError)
}
