use anyhow::Error;
use flux_auth_api::auth_service_client::AuthServiceClient;
use tokio::fs;
use tonic::transport::Channel;
// use flux_auth_api::auth_service_client;

use super::settings::AppSettings;

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub auth_service_client: AuthServiceClient<Channel>,
    pub public_key: Vec<u8>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let auth_service_client =
            AuthServiceClient::connect(settings.clients.flux_auth.endpoint.clone()).await?;

        let public_key = fs::read_to_string(&settings.auth.public_key_file)
            .await?
            .into_bytes();

        Ok(Self {
            settings,
            auth_service_client,
            public_key,
        })
    }
}
