use axum::response::sse::Event;
use tokio::sync::broadcast;

use super::settings::NotifySettings;

#[derive(Clone)]
pub struct NotifyState {
    pub tx: broadcast::Sender<Event>,
}

impl NotifyState {
    pub fn new(settings: NotifySettings) -> Self {
        let tx = broadcast::Sender::new(settings.capacity);

        Self { tx }
    }
}
