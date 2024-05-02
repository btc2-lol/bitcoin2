use axum::response::{IntoResponse, Response};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Outpoint not found")]
    OutpointNotFound,
    #[error("Invalid Script")]
    InvalidScript,
    #[error("Error")]
    Error(String),
    #[error("{0}")]
    IoError(String),
    #[error("{0}")]
    SqlxError(String),
    #[error("Bad Request")]
    BadRequest,
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn _err(description: &str) -> Error {
    Error::Error(description.to_string())
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (format!("{}", self.to_string())).into_response()
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::SqlxError(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}