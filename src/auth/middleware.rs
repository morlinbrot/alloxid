use futures::Future;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::pin::Pin;
use tide::{Next, Request, Response, StatusCode};
use tracing::{debug, error, instrument};

use super::{Claims, Role, AUTHORIZATION, BEARER, SECRET};
use crate::{ErrorKind, ServiceError, State};

type BoxedTideResult<'a> = Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>>;

#[instrument(level = "debug", skip(req, next))]
pub fn authorize<'a>(mut req: Request<State>, next: Next<'a, State>) -> BoxedTideResult {
    match claims_from_request(&req, Role::User) {
        Ok(claims) => {
            // TODO: Check if user id exists here.
            req.set_ext(claims.sub);

            return Box::pin(async {
                let res = next.run(req).await;
                debug!("Token parsed successfully");
                Ok(res)
            });
        }
        Err(_) => {
            return Box::pin(async {
                let res = Response::new(StatusCode::Unauthorized);
                error!("Failed to parse token from request");
                Ok(res)
            })
        }
    };
}

#[instrument(level = "debug", skip(req, role))]
fn claims_from_request(req: &Request<State>, role: Role) -> Result<Claims, ServiceError> {
    let auth_header = match req.header(AUTHORIZATION) {
        Some(header) => header.as_str(),
        None => {
            return Err(ServiceError::new(ErrorKind::TokenExtractionError));
        }
    };

    if !auth_header.starts_with(BEARER) {
        error!("Failed to extract token from header");
        return Err(ServiceError::new(ErrorKind::TokenExtractionError));
    }

    let token = auth_header.trim_start_matches(BEARER);

    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(SECRET),
        &Validation::new(Algorithm::HS512),
    )
    .expect("Failed to decode token.");

    if role == Role::Admin && Role::from_str(&decoded.claims.role) != Role::Admin {
        error!("Role permissions not sufficient");
        return Err(ServiceError::new(ErrorKind::NoPermissionError));
    }

    Ok(decoded.claims)
}
