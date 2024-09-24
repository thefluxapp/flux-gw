use anyhow::Error;
use flux_auth_api::auth_service_client::AuthServiceClient;
use tonic::transport::Channel;
// use flux_auth_api::auth_service_client;

use super::settings::AppSettings;

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub auth_service_client: AuthServiceClient<Channel>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let auth_service_client = AuthServiceClient::connect(settings.clients.flux_auth.endpoint.clone()).await?;

        Ok(Self { settings, auth_service_client })
    }
}
