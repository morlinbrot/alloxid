use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    ArgonauticaError(argonautica::ErrorKind),
    InvalidInputError,
    ParseError,
    TokenCreationError,
    InvalidTokenError,
    TokenExtractionError,
    NoPermissionError,
}

#[derive(Debug)]
pub struct ServiceError {
    msg: String,
    kind: ErrorKind,
    source: Option<&'static (dyn std::error::Error + 'static + Sync)>,
}

impl std::error::Error for ServiceError {}

impl ServiceError {
    pub fn new(kind: ErrorKind) -> Self {
        let msg = match kind {
            ErrorKind::NoPermissionError => "Permission denied.",
            ErrorKind::TokenExtractionError => "No auth header found.",
            ErrorKind::TokenCreationError => "Failed to encode token",
            _ => "Unspecified Error occured.",
        };

        Self {
            msg: msg.to_string(),
            kind,
            source: None,
        }
    }

    pub fn from_str(_msg: &str) -> Self {
        todo!()
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::ArgonauticaError(argo_kind) => {
                write!(
                    f,
                    "ServiceError::{:?}::{:?}: {}",
                    self.kind, argo_kind, self.msg
                )
            }
            _ => {
                write!(f, "ServiceError::{:?}: {}", self.kind, self.msg)
            }
        }
    }
}

impl Into<tide::Response> for ServiceError {
    fn into(self) -> tide::Response {
        tide::log::error!("{:?}", self.to_string());

        match self.kind {
            ErrorKind::InvalidTokenError => tide::Response::builder(403).build(),
            _ => tide::Response::builder(500).build(),
        }
    }
}

impl From<argonautica::Error> for ServiceError {
    fn from(err: argonautica::Error) -> Self {
        Self {
            kind: ErrorKind::ArgonauticaError(err.kind()),
            msg: err.to_string(),
            source: None,
        }
    }
}
