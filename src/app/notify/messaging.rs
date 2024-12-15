use anyhow::Error;
use async_nats::jetstream::consumer::pull::Config;
use log::error;
use prost::Message;
use serde::Serialize;
use tokio_stream::StreamExt as _;

use crate::app::state::AppState;

pub async fn message(state: AppState) -> Result<(), Error> {
    let AppState { js, settings, .. } = state;

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

            let flux_core_api::Message { message, stream } =
                flux_core_api::Message::decode(msg.payload.clone())?;

            // if flux_core_api::Message::decode(message.payload.clone())? {}

            if let (Some(message), Some(stream)) = (message, stream) {
                let event = Event::Message((message, stream).try_into()?);

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
    use anyhow::Error;
    // use async_nats::jetstream::message;
    use flux_core_api::message;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Message {
        message_id: String,
        stream: Stream,
        text: String,
        code: String,
        user: Option<User>,
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

    impl TryFrom<(message::Message, message::Stream)> for Message {
        type Error = Error;

        fn try_from(
            (message, stream): (message::Message, message::Stream),
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
                user: None,
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
