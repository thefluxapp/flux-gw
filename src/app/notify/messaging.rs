use async_nats::jetstream::consumer::pull::Config;
use flux_lib::error::Error;
use flux_users_api::GetUsersRequest;
use log::error;
use prost::Message;
use serde::Serialize;
use tokio_stream::StreamExt as _;

use crate::app::{error::AppError, state::AppState};

pub async fn message(state: AppState) -> Result<(), Error> {
    let AppState {
        js,
        settings,
        users_service_client,
        ..
    } = state;

    let consumer = js
        .create_consumer_on_stream(
            Config {
                durable_name: Some(settings.notify.messaging.message.consumer.clone()),
                filter_subjects: settings.notify.messaging.message.subjects.clone(),
                ..Default::default()
            },
            settings.nats.stream.clone(),
        )
        .await?;

    let msgs = consumer.messages().await?;
    tokio::pin!(msgs);

    while let Some(msg) = msgs.next().await {
        // TODO: create helper for error handler
        if let Err(err) = async {
            let msg = msg.map_err(Error::msg)?;

            let flux_messages_api::Message { message, stream } =
                flux_messages_api::Message::decode(msg.payload.clone())?;

            if let (Some(message), Some(stream)) = (message, stream) {
                let get_users_response = users_service_client
                    .clone()
                    .get_users(GetUsersRequest {
                        user_ids: vec![message.user_id().into()],
                    })
                    .await?
                    .into_inner();

                let user = get_users_response
                    .users
                    .into_iter()
                    .find(|x| x.user_id() == message.user_id())
                    .ok_or(AppError::NoEntity)?;

                let event = Event::Message((message, stream, user).try_into()?);

                if let Err(err) = state.notify.tx.send(event.try_into()?) {
                    error!("{}", err);
                };
            };

            msg.ack().await.map_err(Error::msg)?;

            Ok::<(), Error>(())
        }
        .await
        {
            error!("{}", err);
        }
    }

    Ok(())
}

pub mod message {
    use flux_lib::error::Error;
    use flux_messages_api::message;
    use flux_users_api::get_users_response;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Message {
        message_id: String,
        stream: Stream,
        text: String,
        code: String,
        user: User,
        order: i64,
    }

    #[derive(Serialize)]
    struct User {
        user_id: String,
        name: String,
        first_name: String,
        last_name: String,
        abbr: String,
        color: String,
    }

    #[derive(Serialize)]
    struct Stream {
        message_id: String,
        stream_id: String,
    }

    impl TryFrom<(message::Message, message::Stream, get_users_response::User)> for Message {
        type Error = Error;

        fn try_from(
            (message, stream, user): (message::Message, message::Stream, get_users_response::User),
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                message_id: message.message_id().into(),
                text: message.text().into(),
                code: message.code().into(),
                stream: Stream {
                    message_id: stream.message_id().into(),
                    stream_id: stream.stream_id().into(),
                },
                order: message.order(),
                user: User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    first_name: user.first_name().into(),
                    last_name: user.last_name().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                },
            })
        }
    }

    impl TryFrom<super::Event> for axum::response::sse::Event {
        type Error = Error;

        fn try_from(event: super::Event) -> Result<Self, Self::Error> {
            Ok(Self::default().json_data(event)?)
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Event {
    Message(message::Message),
}
