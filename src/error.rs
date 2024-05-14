use axum::response::{IntoResponse, Response};
use serde::ser::StdError;
use std::convert::Infallible;

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
    ParseError(String),
    #[error("{0}")]
    SqlxError(String),
    #[error("{0}")]
    UnsupportedMethod(String),
    #[error("Invalid transaction")]
    InvalidTransaction,
    #[error("Invalid Signature")]
    InvalidSignature,
    #[error("Function not found")]
    FunctionNotFound,
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

impl From<Vec<u8>> for Error {
    fn from(err: Vec<u8>) -> Self {
        Error::Error(hex::encode(err))
    }
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Self {
        Error::Error(err.to_string())
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        Error::Error(err.to_string())
    }
}

impl From<alloy_rlp::Error> for Error {
    fn from(err: alloy_rlp::Error) -> Self {
        Error::Error(err.to_string())
    }
}

impl From<std::boxed::Box<dyn StdError>> for Error {
    fn from(err: std::boxed::Box<dyn StdError>) -> Self {
        Error::Error(err.to_string())
    }
}
