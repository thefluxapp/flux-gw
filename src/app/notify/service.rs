use std::collections::HashSet;

use axum::extract::ws::{self, WebSocket};
use tracing::error;
use uuid::Uuid;

use crate::app::{error::AppError, state::AppState};

use super::state::{NotifyState, SubscribedStreams};

pub async fn event(state: AppState, req: event::Request) -> Result<(), AppError> {
    let event: event::Event = req.payload.into();

    if let Err(err) = state.notify.tx.send(event) {
        error!("{}", err);
    };

    Ok(())
}

pub mod event {
    use flux_notify_api::event::Payload;
    use serde::Serialize;

    pub struct Request {
        pub payload: Payload,
    }

    #[derive(Debug, Clone, Serialize)]
    pub enum Event {
        Message { message_id: String, text: String },
    }
}

pub async fn notify(
    mut ws: WebSocket,
    notify: NotifyState,
    notify_id: Uuid,
) -> Result<(), AppError> {
    let mut rx = notify.tx.subscribe();
    let streams = notify.streams;

    loop {
        tokio::select! {
            res = rx.recv() => {
                if let Ok(event) = res {
                    let _ = ws.send(event.try_into()?).await;
                } else {
                    continue;
                }
            }
            res = ws.recv() => {
                if let Some(Ok(ws::Message::Text(message))) = res {
                    match serde_json::from_slice::<notify::Request>(message.as_bytes()) {
                        Ok(notify::Request::Subscribe{stream_ids}) => subscribe(&streams, notify_id, stream_ids).await,
                        _ => {},
                    };
                } else {
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn subscribe(streams: &SubscribedStreams, notify_id: Uuid, stream_ids: Vec<String>) {
    let stream_ids = HashSet::from_iter(
        stream_ids
            .iter()
            .map(|v| Uuid::parse_str(v).unwrap())
            .collect::<Vec<Uuid>>(),
    );

    *streams.write().await.entry(notify_id).or_insert(stream_ids) = stream_ids.clone();
}

mod notify {
    use ::serde::Deserialize;
    use axum::extract::ws;

    use crate::app::error::AppError;

    use super::event::Event;

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Request {
        Subscribe { stream_ids: Vec<String> },
    }

    impl TryFrom<Event> for ws::Message {
        type Error = AppError;

        fn try_from(event: Event) -> Result<Self, Self::Error> {
            let res = serde_json::to_string(&event)?;

            Ok(Self::Text(res.into()))
        }
    }
}
