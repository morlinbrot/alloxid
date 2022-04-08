// Adapted from:
// https://github.com/launchbadge/realworld-axum-sqlx/blob/main/src/http/extractor.rs
use axum::extract::{FromRequest, RequestParts};
use http::header::AUTHORIZATION;
use http::HeaderValue;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use tracing::instrument;
use tracing::{debug, error};

use super::ServiceError;
use super::UserId;
use super::{Claims, Role, SCHEME_PREFIX, SECRET};

#[derive(Debug)]
pub struct AuthUser {
    pub user_id: UserId,
}

impl AuthUser {
    #[instrument]
    pub fn from_auth_header(auth_header: &HeaderValue) -> Result<Self, ServiceError> {
        debug!("Called with auth_header: {:?}", &auth_header);

        let auth_header = auth_header.to_str().map_err(|_| {
            error!("Authorization header is not UTF-8");
            ServiceError::Unauthorized
        })?;
        debug!("Parsed auth_header: {:?}", &auth_header);

        if !auth_header.starts_with(SCHEME_PREFIX) {
            error!("Malformed authorization token");
            return Err(ServiceError::Unauthorized);
        }

        let token = auth_header.trim_start_matches(SCHEME_PREFIX);

        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(SECRET),
            &Validation::new(Algorithm::HS512),
        )
        .expect("Failed to decode token.");

        // TODO: Move this to a new struct `AdminUser`.
        let role = Role::User;
        if role == Role::Admin && Role::from_str(&decoded.claims.role) != Role::Admin {
            error!("Role permissions not sufficient");
            return Err(ServiceError::TokenPermissionError);
        }

        Ok(Self {
            user_id: decoded.claims.sub,
        })
    }
}

#[async_trait::async_trait]
impl<B> FromRequest<B> for AuthUser
where
    B: Send,
{
    type Rejection = ServiceError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let auth_header = req
            .headers()
            //.ok_or(ServiceError::Unauthorized)?
            .get(AUTHORIZATION)
            .ok_or(ServiceError::Unauthorized)?;

        Self::from_auth_header(auth_header)
    }
}
