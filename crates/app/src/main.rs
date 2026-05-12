#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Rabbit Main Application
//!
//! Entry point for the Rabbit application.

use app::App;
use tracing::info;

fn main() -> anyhow::Result<()> {
    rabbit_diag::log("entering main()");
    rabbit_diag::log(&format!("args={:?}", std::env::args().collect::<Vec<_>>()));
    rabbit_diag::log(&format!("DISPLAY={:?}", std::env::var("DISPLAY")));
    rabbit_diag::log(&format!("WAYLAND_DISPLAY={:?}", std::env::var("WAYLAND_DISPLAY")));
    rabbit_diag::log(&format!("XAUTHORITY={:?}", std::env::var("XAUTHORITY")));
    rabbit_diag::log(&format!("HOME={:?}", std::env::var("HOME")));
    rabbit_diag::log(&format!("XDG_RUNTIME_DIR={:?}", std::env::var("XDG_RUNTIME_DIR")));
    rabbit_diag::log(&format!("DBUS_SESSION_BUS_ADDRESS={:?}", std::env::var("DBUS_SESSION_BUS_ADDRESS")));
    rabbit_diag::log(&format!("SUDO_UID={:?}", std::env::var("SUDO_UID")));
    rabbit_diag::log(&format!("PKEXEC_UID={:?}", std::env::var("PKEXEC_UID")));
    rabbit_diag::log(&format!("getuid={:?}", adapter::platform::getuid()));

    adapter::maybe_run_as_helper();
    adapter::pre_main_init();

    rabbit_diag::log("calling ensure_elevated()");
    if let Err(e) = adapter::ensure_elevated() {
        eprintln!("Elevation failed: {e}");
        let msg = format!("Elevation failed: {e}");
        fltk::dialog::alert_default(&msg);
        std::process::exit(1);
    }

    rabbit_diag::log("post-elevation: continuing in main");

    // Initialize logging
    rabbit_diag::log("initializing tracing_subscriber");
    tracing_subscriber::fmt().with_env_filter("info").init();
    rabbit_diag::log("tracing_subscriber initialized");

    rabbit_diag::log("Starting Rabbit application (diag)");
    info!("Starting Rabbit application");

    // Build a custom Tokio runtime with reduced thread stack size and worker count.
    // Default stack per thread is 8MB; with FLTK/Pango font threads also consuming
    // virtual memory, the default settings exhaust the address space under tight
    // ulimit -v constraints. 2MB stacks and 2 workers are sufficient for this app.
    rabbit_diag::log("building tokio runtime");
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(2 * 1024 * 1024) // 2MB per thread
        .enable_all()
        .build()
    {
        Ok(r) => {
            rabbit_diag::log("tokio runtime built");
            r
        }
        Err(e) => {
            rabbit_diag::log(&format!("tokio runtime build FAILED: {e}"));
            return Err(e.into());
        }
    };

    rabbit_diag::log("entering runtime.block_on");
    let result = runtime.block_on(async {
        rabbit_diag::log("inside block_on: calling App::new()");
        let mut app = match App::new().await {
            Ok(a) => {
                rabbit_diag::log("inside block_on: App::new() succeeded");
                a
            }
            Err(e) => {
                rabbit_diag::log(&format!("inside block_on: App::new() FAILED: {e}"));
                return Err(e);
            }
        };
        rabbit_diag::log("inside block_on: calling app.run()");
        let run_result = app.run().await;
        rabbit_diag::log("inside block_on: app.run() returned");
        info!("Rabbit application exited");
        run_result
    });
        rabbit_diag::log("runtime.block_on returned");
    result
}
