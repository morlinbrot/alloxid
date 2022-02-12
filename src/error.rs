use std::fmt;

use axum::body;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Debug)]
pub enum ServiceError {
    InvalidInputError,
    InvalidTokenError,
    NoPermissionError,
    ParseError,
    ParseUserDataError,
    TokenCreationError,
    TokenExtractionError,

    ArgonauticaError(argonautica::Error),
    SQLXError(sqlx::Error),
    SerdeError(serde_json::Error),

    WithStatusCode(StatusCode),
}

impl std::error::Error for ServiceError {}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Self::NoPermissionError => "Permission denied.".to_string(),
            Self::TokenExtractionError => "No auth header found.".to_string(),
            Self::TokenCreationError => "Failed to encode token".to_string(),
            Self::ParseUserDataError => "Failed to parse user data".to_string(),
            Self::SQLXError(err) => err.to_string(),
            Self::SerdeError(err) => err.to_string(),
            Self::ArgonauticaError(err) => err.to_string(),
            _ => "Unspecified Error occured.".to_string(),
        };

        write!(f, "ServiceError::{:?}: {}", self, msg)
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = match self {
            ServiceError::WithStatusCode(status) => status,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Response::builder()
            .status(status)
            .body(body::boxed(body::Full::from(self.to_string())))
            .unwrap()
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        Self::SQLXError(err)
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeError(err)
    }
}

impl From<argonautica::Error> for ServiceError {
    fn from(err: argonautica::Error) -> Self {
        Self::ArgonauticaError(err)
    }
}
