use flux_lib::error::Error;
use tokio_stream::StreamExt as _;
use tracing::error;

use crate::app::state::AppState;

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
