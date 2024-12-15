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
    Stream,
};

use super::{error::AppError, state::AppState, user::AppUser};

mod messaging;
pub(super) mod settings;
pub(super) mod state;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(notify))
}

async fn notify(
    State(AppState { notify, .. }): State<AppState>,
    user: Option<AppUser>,
) -> Sse<impl Stream<Item = Result<Event, BroadcastStreamRecvError>>> {
    // TODO: How to handle BroadcastStreamRecvError?

    if let Some(u) = user {
        println!("{}", &u.id);
    } else {
        println!("NO USER");
    };

    let rx = notify.tx.subscribe();

    Sse::new(BroadcastStream::new(rx)).keep_alive(KeepAlive::default())
}

pub async fn messaging(state: &AppState) -> Result<(), AppError> {
    tokio::spawn(messaging::message(state.clone()));

    Ok(())
}
