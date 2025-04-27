use axum::extract::ws::{self, WebSocket};
use tracing::error;
use uuid::Uuid;

use crate::app::{error::AppError, state::AppState};

use super::state::NotifyState;

pub async fn event(state: AppState, req: event::Request) -> Result<(), AppError> {
    let event: event::Event = req.payload.try_into()?;

    if let Err(err) = state.notify.tx.send(event) {
        error!("{}", err);
    };

    Ok(())
}

pub mod event {
    use flux_notify_api::event::Payload;
    use serde::Serialize;

    use crate::app::error::AppError;

    pub struct Request {
        pub payload: Payload,
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Event {
        Message(Message),
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct Message {
        pub message_id: String,
        pub stream: Option<Stream>,
        pub text: String,
        pub code: String,
        pub user: User,
        pub order: i64,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub first_name: String,
        pub last_name: String,
        pub abbr: String,
        pub color: String,
    }

    #[derive(Debug, Clone, Serialize)]
    pub struct Stream {
        pub stream_id: String,
        pub message_id: String,
        pub text: Option<String>,
        pub users: Vec<User>,
    }

    impl TryFrom<Payload> for Event {
        type Error = AppError;

        fn try_from(payload: Payload) -> Result<Self, Self::Error> {
            Ok(match payload {
                Payload::Message(message) => message.try_into()?,
            })
        }
    }

    impl TryFrom<flux_notify_api::Message> for Event {
        type Error = AppError;

        fn try_from(message: flux_notify_api::Message) -> Result<Self, Self::Error> {
            let user = message.user.clone().ok_or(AppError::NoEntity)?;

            Ok(Self::Message(Message {
                message_id: message.message_id().into(),
                stream: None,
                text: message.text().into(),
                code: message.code().into(),
                user: user.into(),
                order: message.order(),
            }))
        }
    }

    impl From<flux_notify_api::message::User> for User {
        fn from(user: flux_notify_api::message::User) -> Self {
            Self {
                user_id: user.user_id().into(),
                name: user.name().into(),
                first_name: user.first_name().into(),
                last_name: user.last_name().into(),
                abbr: user.abbr().into(),
                color: user.color().into(),
            }
        }
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
                        Ok(notify::Request::Subscribe{stream_ids}) => notify::subscribe(&streams, notify_id, stream_ids).await,
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

mod notify {
    use std::collections::HashSet;

    use ::serde::Deserialize;
    use axum::extract::ws;
    use uuid::Uuid;

    use crate::app::{error::AppError, notify::state::SubscribedStreams};

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

    pub async fn subscribe(streams: &SubscribedStreams, notify_id: Uuid, stream_ids: Vec<String>) {
        let stream_ids = HashSet::from_iter(
            stream_ids
                .iter()
                .map(|v| Uuid::parse_str(v).unwrap())
                .collect::<Vec<Uuid>>(),
        );

        *streams.write().await.entry(notify_id).or_insert(stream_ids) = stream_ids.clone();
    }
}
