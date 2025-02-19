use anyhow::Error;
use async_nats::jetstream;
use axum::{routing::get, Router};
use log::info;
use settings::AppSettings;
use state::AppState;

mod auth;
mod error;
mod messages;
mod notify;
mod push;
mod settings;
mod state;
mod streams;
mod user;

pub async fn run() -> Result<(), Error> {
    let settings = AppSettings::new()?;
    let state = AppState::new(settings).await?;

    messaging(&state).await?;
    http_and_grpc(&state).await?;

    Ok(())
}

async fn http_and_grpc(state: &AppState) -> Result<(), Error> {
    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/healthz", get(|| async {}))
                .nest("/auth", auth::router())
                .nest("/streams", streams::router())
                .nest("/messages", messages::router())
                .nest("/push", push::router())
                .nest("/notify", notify::router()),
        )
        .with_state(state.to_owned());

    let listener = tokio::net::TcpListener::bind(&state.settings.http.endpoint).await?;

    info!("app: started");
    axum::serve(listener, router).await?;

    Ok(())
}

async fn messaging(state: &AppState) -> Result<(), Error> {
    notify::messaging(&state).await.unwrap();

    info!("messaging: started");

    Ok(())
}

pub type AppJS = jetstream::Context;
