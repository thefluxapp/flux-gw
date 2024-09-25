use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{error::AppError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        // .route("/login", post(controller::login))
        .route("/join", post(join))
        .route("/complete", post(complete))
    // .route("/me", get(controller::me))
}

async fn join(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    Json(data): Json<JoinRequest>,
) -> Result<Json<Value>, AppError> {
    let request: flux_auth_api::JoinRequest = data.into();
    let response = auth_service_client
        .clone()
        .join(request)
        .await?
        .into_inner();

    Ok(Json(serde_json::from_str(response.response())?))
}

#[derive(Deserialize)]
struct JoinRequest {
    pub email: Option<String>,
}

impl From<JoinRequest> for flux_auth_api::JoinRequest {
    fn from(request: JoinRequest) -> Self {
        Self {
            email: request.email,
        }
    }
}

async fn complete(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    Json(data): Json<Value>,
) -> Result<Json<CompleteResponse>, AppError> {
    let request = flux_auth_api::CompleteRequest {
        request: Some(data.to_string()),
    };
    let response = auth_service_client
        .clone()
        .complete(request)
        .await?
        .into_inner();

    Ok(Json(response.into()))
}

impl Into<CompleteResponse> for flux_auth_api::CompleteResponse {
    fn into(self) -> CompleteResponse {
        CompleteResponse {
            jwt: self.jwt().into(),
        }
    }
}

#[derive(Serialize)]
struct CompleteResponse {
    pub jwt: String,
}
