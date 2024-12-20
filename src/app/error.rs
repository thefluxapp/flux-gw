use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let payload = json!({
            "code": "XXX", "message": self.0.to_string()
        });

        (StatusCode::BAD_REQUEST, Json(payload)).into_response()
    }
}

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// impl From<async_nats::error::Error<async_nats::jetstream::stream::ConsumerErrorKind>> for AppError {
//     fn from(_: async_nats::error::Error<async_nats::jetstream::stream::ConsumerErrorKind>) -> Self {
//         Self::DUMMY
//     }
// }

// #[derive(Error, Debug)]
// pub enum AppError {
//     #[error("entity not found")]
//     DUMMY,
//     #[error(transparent)]
//     Other(#[from] anyhow::Error),
// }
