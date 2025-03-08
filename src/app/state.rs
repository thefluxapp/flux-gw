use std::sync::Arc;

use async_nats::jetstream;
use flux_lib::error::Error;
use flux_messages_api::{
    messages_service_client::MessagesServiceClient, streams_service_client::StreamsServiceClient,
};
use flux_notify_api::push_service_client::PushServiceClient;
use flux_users_api::{
    auth_service_client::AuthServiceClient, users_service_client::UsersServiceClient,
};
use tokio::fs;
use tonic::transport::Channel;

use super::{notify::state::NotifyState, settings::AppSettings, AppJS};

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub auth_service_client: AuthServiceClient<Channel>,
    pub users_service_client: UsersServiceClient<Channel>,
    pub streams_service_client: StreamsServiceClient<Channel>,
    pub messages_service_client: MessagesServiceClient<Channel>,
    pub push_service_client: PushServiceClient<Channel>,
    pub public_key: Vec<u8>,
    pub notify: NotifyState,
    pub js: Arc<AppJS>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let notify = NotifyState::new(settings.notify.clone());

        let nats = async_nats::connect(&settings.nats.endpoint).await?;
        let js = Arc::new(jetstream::new(nats));

        let auth_service_client =
            Self::auth_service_client(settings.clients.flux_auth.endpoint.clone()).await?;

        let users_service_client =
            Self::users_service_client(settings.clients.flux_auth.endpoint.clone()).await?;

        let streams_service_client =
            Self::streams_service_client(settings.clients.flux_core.endpoint.clone()).await?;

        let messages_service_client =
            Self::messages_service_client(settings.clients.flux_core.endpoint.clone()).await?;

        let push_service_client =
            Self::push_service_client(settings.clients.flux_notify.endpoint.clone()).await?;

        let public_key = fs::read_to_string(&settings.auth.public_key_file)
            .await?
            .into_bytes();

        Ok(Self {
            settings,
            auth_service_client,
            users_service_client,
            streams_service_client,
            messages_service_client,
            push_service_client,
            public_key,
            notify,
            js,
        })
    }

    async fn auth_service_client(dst: String) -> Result<AuthServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(AuthServiceClient::new(ch))
    }

    async fn users_service_client(dst: String) -> Result<UsersServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(UsersServiceClient::new(ch))
    }

    async fn streams_service_client(dst: String) -> Result<StreamsServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(StreamsServiceClient::new(ch))
    }

    async fn messages_service_client(dst: String) -> Result<MessagesServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(MessagesServiceClient::new(ch))
    }

    async fn push_service_client(dst: String) -> Result<PushServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(PushServiceClient::new(ch))
    }
}
