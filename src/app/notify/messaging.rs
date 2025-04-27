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
        self,
        consumer::{pull::Config, Consumer},
    };
    use flux_lib::error::Error;
    use prost::Message as _;

    use crate::app::{
        error::AppError,
        notify::service::{self, event::Request},
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

    pub async fn handler(state: AppState, message: jetstream::Message) -> Result<(), Error> {
        service::event(state, message.clone().try_into()?).await?;

        message.ack().await.map_err(Error::msg)?;
        Ok(())
    }

    impl TryFrom<jetstream::Message> for Request {
        type Error = AppError;

        fn try_from(message: jetstream::Message) -> Result<Self, Self::Error> {
            let flux_notify_api::Event { payload } =
                flux_notify_api::Event::decode(message.payload.as_ref())?;

            let payload = payload.ok_or(AppError::NoEntity)?;

            Ok(Self { payload })
        }
    }
}
