use async_nats::jetstream;
use axum::{routing::get, Router};
use flux_lib::error::Error;
use settings::AppSettings;
use state::AppState;
use tracing::info;

mod auth;
mod error;
mod locale;
mod messages;
mod notify;
mod pushes;
mod settings;
mod state;
mod streams;
mod user;

pub async fn run() -> Result<(), Error> {
    let settings = AppSettings::new()?;
    let state = AppState::new(settings).await?;

    messaging(&state).await?;
    http(&state).await?;

    Ok(())
}

async fn http(state: &AppState) -> Result<(), Error> {
    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/healthz", get(|| async {}))
                .nest("/auth", auth::router())
                .nest("/streams", streams::router())
                .nest("/messages", messages::router())
                .nest("/pushes", pushes::router())
                .nest("/notify", notify::router()),
        )
        .with_state(state.to_owned());

    let listener = tokio::net::TcpListener::bind(&state.settings.http.endpoint).await?;

    info!("app: started on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}

async fn messaging(state: &AppState) -> Result<(), Error> {
    notify::messaging(&state).await.unwrap();

    info!("messaging: started");

    Ok(())
}

pub type AppJS = jetstream::Context;
