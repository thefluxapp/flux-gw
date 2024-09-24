use anyhow::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct AppError(Error);

impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        dbg!(&self.0);

        let status = match self {
            _ => StatusCode::BAD_REQUEST,
        };

        (status, self.0.to_string()).into_response()
    }
}
