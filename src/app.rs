use anyhow::Error;
use axum::{routing::get, Router};
use settings::AppSettings;
use state::AppState;

mod auth;
mod error;
mod messages;
mod settings;
mod state;
mod streams;
mod user;

pub async fn run() -> Result<(), Error> {
    let settings = AppSettings::new()?;
    let state = AppState::new(settings).await?;

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
                .nest("/messages", messages::router()),
        )
        .with_state(state.to_owned());

    let listener = tokio::net::TcpListener::bind(&state.settings.http.endpoint).await?;

    axum::serve(listener, router).await?;

    Ok(())
}
