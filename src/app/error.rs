use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::typed_header::TypedHeaderRejection;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Status(status) => match status.code() {
                tonic::Code::InvalidArgument => (StatusCode::UNPROCESSABLE_ENTITY, "".to_string()),
                code => (StatusCode::BAD_REQUEST, code.to_string()),
            },
            error => (StatusCode::BAD_REQUEST, error.to_string()),
        }
        .into_response()
    }
}

impl From<uuid::Error> for AppError {
    fn from(error: uuid::Error) -> Self {
        AppError::Other(flux_lib::error::Error::new(error))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("there is not entity")]
    NoEntity,
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
    #[error(transparent)]
    Auth(#[from] TypedHeaderRejection),
    #[error("RECV")]
    Recv(#[from] tokio::sync::broadcast::error::RecvError),
    #[error(transparent)]
    Other(#[from] flux_lib::error::Error),
}
