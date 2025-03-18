use std::time::Duration;

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::get,
    Router,
};
use tokio_stream::{
    wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    Stream, StreamExt,
};

use super::{error::AppError, state::AppState, user::AppUser};

mod messaging;
pub(super) mod settings;
pub(super) mod state;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(notify))
}

async fn notify(
    State(AppState {
        notify, shutdown, ..
    }): State<AppState>,
    user: Option<AppUser>,
) -> Sse<impl Stream<Item = Result<Event, BroadcastStreamRecvError>>> {
    // TODO: How to handle BroadcastStreamRecvError?

    if let Some(u) = user {
        println!("{}", &u.id);
    } else {
        println!("NO USER");
    };

    let mut rxx = shutdown.rx;

    let rx = notify.tx.subscribe();
    let mut stream = BroadcastStream::new(rx);
    let stream = async_stream::stream! {
        loop {
            tokio::select! {
                Some(item) = stream.next() => {
                    yield item
                },
                _ = rxx.changed() => {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(1)))
}

pub async fn messaging(state: &AppState) -> Result<(), AppError> {
    tokio::spawn(messaging::message(state.clone()));

    Ok(())
}
