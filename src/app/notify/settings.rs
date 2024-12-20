use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct NotifySettings {
    pub messaging: MessagingSettings,
    pub capacity: usize,
}

#[derive(Deserialize, Clone)]
pub struct MessagingSettings {
    pub message: MessagingMessageSettings,
}

#[derive(Deserialize, Clone)]
pub struct MessagingMessageSettings {
    pub subjects: Vec<String>,
    pub consumer: String,
}
