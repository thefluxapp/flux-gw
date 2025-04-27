use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct NotifySettings {
    pub messaging: MessagingSettings,
    pub capacity: usize,
}

#[derive(Deserialize, Clone)]
pub struct MessagingSettings {
    pub event: MessagingEventSettings,
}

#[derive(Deserialize, Clone)]
pub struct MessagingEventSettings {
    pub subjects: Vec<String>,
    pub consumer: String,
}
