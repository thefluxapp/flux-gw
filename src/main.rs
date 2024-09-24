use anyhow::Error;

mod app;

#[tokio::main]
async fn main() -> Result<(), Error> {
    app::run().await?;

    Ok(())
}
