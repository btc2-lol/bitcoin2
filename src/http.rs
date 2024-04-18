use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
#[derive(Debug, Clone)]
pub struct Error(pub StatusCode, pub String);
pub type Result<T> = core::result::Result<T, Error>;

pub fn err(description: &str) -> Error {
    Error(StatusCode::INTERNAL_SERVER_ERROR, description.to_string())
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error,
{
    fn from(err: E) -> Self {
        Error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}
