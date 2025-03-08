use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::typed_header::TypedHeaderRejection;
use log::debug;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        debug!("{}", self.to_string());

        match self {
            AppError::Status(status) => match status.code() {
                tonic::Code::InvalidArgument => StatusCode::UNPROCESSABLE_ENTITY,
                _ => StatusCode::BAD_REQUEST,
            },
            _ => StatusCode::BAD_REQUEST,
        }
        .into_response()
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
    Auth(#[from] TypedHeaderRejection),
    #[error(transparent)]
    Other(#[from] flux_lib::error::Error),
}
