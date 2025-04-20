use async_nats::jetstream::consumer::pull::Config;
use flux_lib::error::Error;
use tokio_stream::StreamExt as _;
use tracing::error;

use crate::app::state::AppState;

use super::service;

pub async fn event(state: AppState) -> Result<(), Error> {
    let AppState { js, settings, .. } = state.clone();

    let consumer = event::consumer(&js, &settings).await?;
    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        if let Err(err) = event::handler(state.clone(), message?).await {
            error!("{}", err);
        }
    }

    Ok(())
}

mod event {
    use async_nats::jetstream::{
        consumer::{pull::Config, Consumer},
        Message,
    };
    use flux_lib::error::Error;
    use flux_notify_api::event::Payload;
    use prost::Message as _;

    use crate::app::{
        error::AppError,
        notify::service::{
            self,
            event::{Event, Request},
        },
        settings::AppSettings,
        state::AppState,
        AppJS,
    };

    pub async fn consumer(js: &AppJS, settings: &AppSettings) -> Result<Consumer<Config>, Error> {
        Ok(js
            .create_consumer_on_stream(
                Config {
                    durable_name: Some(settings.notify.messaging.event.consumer.clone()),
                    filter_subjects: settings.notify.messaging.event.subjects.clone(),
                    ..Default::default()
                },
                settings.nats.stream.clone(),
            )
            .await?)
    }

    pub async fn handler(state: AppState, message: Message) -> Result<(), Error> {
        service::event(state, message.clone().try_into()?).await?;

        message.ack().await.map_err(Error::msg)?;
        Ok(())
    }

    impl TryFrom<Message> for Request {
        type Error = AppError;

        fn try_from(message: Message) -> Result<Self, Self::Error> {
            let flux_notify_api::Event { payload } =
                flux_notify_api::Event::decode(message.payload.as_ref())?;

            let payload = payload.ok_or(AppError::NoEntity)?;

            Ok(Self { payload })
        }
    }

    impl From<Payload> for Event {
        fn from(payload: Payload) -> Self {
            match payload {
                Payload::Message(message) => Self::Message {
                    message_id: message.message_id().into(),
                    text: message.text().into(),
                },
            }
        }
    }
}

// pub async fn message(state: AppState) -> Result<(), Error> {
//     let AppState {
//         js,
//         settings,
//         users_service_client,
//         ..
//     } = state;

//     let consumer = js
//         .create_consumer_on_stream(
//             Config {
//                 durable_name: Some(settings.notify.messaging.message.consumer.clone()),
//                 filter_subjects: settings.notify.messaging.message.subjects.clone(),
//                 ..Default::default()
//             },
//             settings.nats.stream.clone(),
//         )
//         .await?;

//     let msgs = consumer.messages().await?;
//     tokio::pin!(msgs);

//     while let Some(msg) = msgs.next().await {
//         // TODO: create helper for error handler
//         if let Err(err) = async {
//             let msg = msg.map_err(Error::msg)?;

//             let flux_messages_api::Message { message, stream } =
//                 flux_messages_api::Message::decode(msg.payload.clone())?;

//             if let (Some(message), Some(stream)) = (message, stream) {
//                 let get_users_response = users_service_client
//                     .clone()
//                     .get_users(GetUsersRequest {
//                         user_ids: vec![message.user_id().into()],
//                     })
//                     .await?
//                     .into_inner();

//                 let user = get_users_response
//                     .users
//                     .into_iter()
//                     .find(|x| x.user_id() == message.user_id())
//                     .ok_or(AppError::NoEntity)?;

//                 let event: message::Message = (message, stream, user).try_into()?;

//                 if let Err(err) = state.notify.tx.send(event.try_into()?) {
//                     error!("TX: {}", err);
//                 };
//             };

//             msg.ack().await.map_err(Error::msg)?;

//             Ok::<(), Error>(())
//         }
//         .await
//         {
//             error!("{}", err);
//         }
//     }

//     Ok(())
// }

// pub mod message {
//     use flux_lib::error::Error;
//     use flux_messages_api::message;
//     use flux_users_api::get_users_response;
//     use serde::Serialize;

//     #[derive(Serialize, Clone)]
//     pub struct Message {
//         pub message_id: String,
//         stream: Stream,
//         pub text: String,
//         pub code: String,
//         user: User,
//         order: i64,
//     }

//     #[derive(Serialize, Clone)]
//     struct User {
//         user_id: String,
//         name: String,
//         first_name: String,
//         last_name: String,
//         abbr: String,
//         color: String,
//     }

//     #[derive(Serialize, Clone)]
//     struct Stream {
//         message_id: String,
//         stream_id: String,
//     }

//     impl TryFrom<(message::Message, message::Stream, get_users_response::User)> for Message {
//         type Error = Error;

//         fn try_from(
//             (message, stream, user): (message::Message, message::Stream, get_users_response::User),
//         ) -> Result<Self, Self::Error> {
//             Ok(Self {
//                 message_id: message.message_id().into(),
//                 text: message.text().into(),
//                 code: message.code().into(),
//                 stream: Stream {
//                     message_id: stream.message_id().into(),
//                     stream_id: stream.stream_id().into(),
//                 },
//                 order: message.order(),
//                 user: User {
//                     user_id: user.user_id().into(),
//                     name: user.name().into(),
//                     first_name: user.first_name().into(),
//                     last_name: user.last_name().into(),
//                     abbr: user.abbr().into(),
//                     color: user.color().into(),
//                 },
//             })
//         }
//     }
// }
