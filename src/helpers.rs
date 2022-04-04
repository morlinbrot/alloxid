use argonautica::Verifier;

use crate::error::ServiceError;

pub fn verify_password(hash: &str, password: &str, secret: &str) -> Result<bool, ServiceError> {
    let mut verifier = Verifier::default();
    verifier
        .with_hash(&hash)
        .with_password(password)
        .with_secret_key(secret)
        .verify()
        .map_err(ServiceError::from)
}
