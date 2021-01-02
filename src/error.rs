use std::error;
use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    Argonautica,
    CrateError,
    InvalidData,
}

#[derive(Debug)]
pub struct Error {
    msg: String,
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind, msg: String) -> Self {
        return Self { kind, msg };
    }

    #[allow(dead_code)]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.msg)
    }
}

impl From<argonautica::Error> for Error {
    fn from(err: argonautica::Error) -> Self {
        let msg = err.to_string();
        Self::new(ErrorKind::Argonautica, msg)
    }
}
