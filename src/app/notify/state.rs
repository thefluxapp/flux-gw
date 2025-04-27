use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use super::{service::event::Event, settings::NotifySettings};

#[derive(Clone)]
pub struct NotifyState {
    pub tx: broadcast::Sender<Event>,
    pub streams: SubscribedStreams,
}

impl NotifyState {
    pub fn new(settings: NotifySettings) -> Self {
        let tx = broadcast::Sender::new(settings.capacity);
        let streams = SubscribedStreams::default();

        Self { tx, streams }
    }
}

pub type SubscribedStreams = Arc<RwLock<HashMap<Uuid, HashSet<Uuid>>>>;
