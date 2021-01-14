use futures::Future;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::pin::Pin;
use tide::{Next, Request, Response, StatusCode};

use super::{Claims, Role, AUTHORIZATION, BEARER, SECRET};
use crate::{ErrorKind, ServiceError, State};

type BoxedTideResult<'a> = Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>>;

pub fn authorize<'a>(mut req: Request<State>, next: Next<'a, State>) -> BoxedTideResult {
    match claims_from_request(&req, Role::User) {
        Ok(claims) => {
            // TODO: Check if user id exists here.
            req.set_ext(claims.sub);

            return Box::pin(async {
                let res = next.run(req).await;
                tide::log::info!("Token parsed successfully.");
                Ok(res)
            });
        }
        Err(_) => {
            return Box::pin(async {
                let res = Response::new(StatusCode::Unauthorized);
                tide::log::info!("Failed to parse token from request.");
                Ok(res)
            })
        }
    };
}

fn claims_from_request(req: &Request<State>, role: Role) -> Result<Claims, ServiceError> {
    let auth_header = match req.header(AUTHORIZATION) {
        Some(header) => header.as_str(),
        None => {
            return Err(ServiceError::new(ErrorKind::TokenExtractionError));
        }
    };

    if !auth_header.starts_with(BEARER) {
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
        return Err(ServiceError::new(ErrorKind::NoPermissionError));
    }

    Ok(decoded.claims)
}
