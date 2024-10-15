use axum::{extract::State, routing::get, Json, Router};
use get_streams::Response;

use super::{error::AppError, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_streams))
}

async fn get_streams(
    State(AppState {
        streams_service_client,
        ..
    }): State<AppState>,
) -> Result<Json<Response>, AppError> {
    let request = flux_core_api::GetStreamsRequest {};
    let response = streams_service_client
        .clone()
        .get_streams(request)
        .await?
        .into_inner();

    Ok(Json(response.into()))
}

mod get_streams {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Response {
        pub streams: Vec<Stream>,
    }

    #[derive(Serialize)]
    pub struct Stream {
        pub id: String,
        pub message_id: String,
        pub text: Option<String>,
    }

    impl Into<Response> for flux_core_api::GetStreamsResponse {
        fn into(self) -> Response {
            Response {
                streams: self
                    .streams
                    .iter()
                    .map(|m| Stream {
                        id: m.id().into(),
                        message_id: m.message_id().into(),
                        text: m.text.clone(),
                    })
                    .collect(),
            }
        }
    }
}
