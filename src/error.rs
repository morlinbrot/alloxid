use axum::body;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(Clone, Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Failed to create token")]
    TokenCreationError,
    #[error("Failed to extract token")]
    TokenExtractionError,
    #[error("Insufficient permissions")]
    TokenPermissionError,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Library error")]
    LibError(String),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let status = match self {
            ServiceError::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let msg = self.to_string();

        Response::builder()
            .status(status)
            .body(body::boxed(body::Full::from(msg)))
            .unwrap()
    }
}

impl From<argonautica::Error> for ServiceError {
    fn from(err: argonautica::Error) -> Self {
        Self::LibError(err.to_string())
    }
}

impl From<config::ConfigError> for ServiceError {
    fn from(err: config::ConfigError) -> Self {
        Self::LibError(err.to_string())
    }
}

impl From<serde_json::Error> for ServiceError {
    fn from(err: serde_json::Error) -> Self {
        Self::LibError(err.to_string())
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        Self::LibError(err.to_string())
    }
}
