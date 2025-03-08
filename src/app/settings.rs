use flux_lib::settings::{HttpSettings, NATSSettings, Settings};
use serde::Deserialize;

use super::{auth::settings::AuthSettings, notify::settings::NotifySettings};

#[derive(Deserialize, Clone)]
pub struct AppSettings {
    pub _name: String,
    pub http: HttpSettings,
    pub auth: AuthSettings,
    pub clients: ClientsSettings,
    pub notify: NotifySettings,
    pub nats: NATSSettings,
}

#[derive(Deserialize, Clone)]
pub struct ClientsSettings {
    pub flux_auth: ClientSettings,
    pub flux_core: ClientSettings,
    pub flux_notify: ClientSettings,
}

#[derive(Deserialize, Clone)]
pub struct ClientSettings {
    pub endpoint: String,
}

impl Settings<AppSettings> for AppSettings {}
