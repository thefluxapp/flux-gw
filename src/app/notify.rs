use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
    Router,
};
use uuid::Uuid;

use super::{error::AppError, state::AppState};

mod messaging;
mod service;
pub(super) mod settings;
pub(super) mod state;

pub fn router() -> Router<AppState> {
    Router::new().route("/", any(notify))
}

async fn notify(
    State(AppState { notify, .. }): State<AppState>,
    wsu: WebSocketUpgrade,
) -> Result<impl IntoResponse, AppError> {
    let notify_id = Uuid::now_v7();

    let res = wsu.on_upgrade(move |ws| async move {
        let _ = service::notify(ws, notify.clone(), notify_id).await;
    });

    Ok(res)
}

pub async fn messaging(state: &AppState) -> Result<(), AppError> {
    tokio::spawn(messaging::event(state.clone()));

    Ok(())
}
