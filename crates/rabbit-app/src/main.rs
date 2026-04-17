//! Rabbit Main Application
//!
//! Entry point for the Rabbit application.

use rabbit_app::App;
use tracing::info;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("Starting Rabbit application");

    // Build a custom Tokio runtime with reduced thread stack size and worker count.
    // Default stack per thread is 8MB; with FLTK/Pango font threads also consuming
    // virtual memory, the default settings exhaust the address space under tight
    // ulimit -v constraints. 2MB stacks and 2 workers are sufficient for this app.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(2 * 1024 * 1024) // 2MB per thread
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let app = App::new().await?;
        app.run().await?;
        info!("Rabbit application exited");
        Ok(())
    })
}
