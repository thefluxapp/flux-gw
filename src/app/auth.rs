use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::Value;

use super::{error::AppError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        // .route("/login", post(controller::login))
        .route("/join", post(join))
    // .route("/complete", post(controller::complete))
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
    let response = auth_service_client.clone().join(request).await?.into_inner();

    Ok(Json(serde_json::from_str(response.response())?))
}


#[derive(Deserialize)]
struct JoinRequest {
    pub email: Option<String>,
}

impl From<JoinRequest> for flux_auth_api::JoinRequest {
    fn from(request: JoinRequest) -> Self {
        Self { email: request.email }
    }
}
