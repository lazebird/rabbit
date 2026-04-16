//! Rabbit Main Application
//!
//! Entry point for the Rabbit application.

use rabbit_app::App;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting Rabbit application");

    // Create and run the application
    let app = App::new().await?;
    app.run().await?;

    info!("Rabbit application exited");
    Ok(())
}
