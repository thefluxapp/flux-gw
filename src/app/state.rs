use anyhow::Error;
use flux_auth_api::auth_service_client::AuthServiceClient;
use flux_core_api::streams_service_client::StreamsServiceClient;
use tokio::fs;
use tonic::transport::Channel;

use super::settings::AppSettings;

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub auth_service_client: AuthServiceClient<Channel>,
    pub streams_service_client: StreamsServiceClient<Channel>,
    pub public_key: Vec<u8>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let auth_service_client =
            Self::auth_service_client(settings.clients.flux_auth.endpoint.clone()).await?;

        let streams_service_client =
            Self::streams_service_client(settings.clients.flux_core.endpoint.clone()).await?;

        let public_key = fs::read_to_string(&settings.auth.public_key_file)
            .await?
            .into_bytes();

        Ok(Self {
            settings,
            auth_service_client,
            streams_service_client,
            public_key,
        })
    }

    async fn auth_service_client(dst: String) -> Result<AuthServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(AuthServiceClient::new(ch))
    }

    async fn streams_service_client(dst: String) -> Result<StreamsServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(StreamsServiceClient::new(ch))
    }
}
