#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Rabbit Main Application
//!
//! Entry point for the Rabbit application.

use app::App;
use tracing::info;

fn main() -> anyhow::Result<()> {
    adapter::diag::log("entering main()");
    adapter::diag::log(&format!("args={:?}", std::env::args().collect::<Vec<_>>()));
    adapter::diag::log(&format!("DISPLAY={:?}", std::env::var("DISPLAY")));
    adapter::diag::log(&format!("WAYLAND_DISPLAY={:?}", std::env::var("WAYLAND_DISPLAY")));
    adapter::diag::log(&format!("XAUTHORITY={:?}", std::env::var("XAUTHORITY")));
    adapter::diag::log(&format!("HOME={:?}", std::env::var("HOME")));
    adapter::diag::log(&format!("XDG_RUNTIME_DIR={:?}", std::env::var("XDG_RUNTIME_DIR")));
    adapter::diag::log(&format!("DBUS_SESSION_BUS_ADDRESS={:?}", std::env::var("DBUS_SESSION_BUS_ADDRESS")));
    adapter::diag::log(&format!("SUDO_UID={:?}", std::env::var("SUDO_UID")));
    adapter::diag::log(&format!("PKEXEC_UID={:?}", std::env::var("PKEXEC_UID")));
    adapter::diag::log(&format!("getuid={}", unsafe { libc::getuid() }));

    // ── tray-helper mode (spawned from the non-elevated parent) ──
    #[cfg(target_os = "linux")]
    if std::env::args().any(|a| a == "--tray-helper") {
        adapter::diag::log("starting tray helper mode");
        return app::tray_helper::run().map_err(|e| anyhow::anyhow!("{e}"));
    }

    // Ensure elevated privileges FIRST to avoid redundant initialization if restarting.
    // On Linux, spawn the tray helper before elevation so it can own the D-Bus tray icon
    // as the original (non-root) user.
    //
    // Skip spawn() when already elevated (root) — the helper was already spawned by the
    // pre-elevation process and a root-level helper would be unable to access the user's
    // D-Bus session bus for the tray icon.
    #[cfg(not(debug_assertions))]
    {
        #[cfg(target_os = "linux")]
        if !adapter::is_elevated() {
            adapter::diag::log("spawning tray helper");
            if let Err(e) = app::tray_helper::spawn() {
                eprintln!("Warning: failed to spawn tray helper: {e}");
                // Continue without helper — tray icon won't be available after elevation.
            }
        }
        adapter::diag::log("calling ensure_elevated()");
        adapter::elevation::ensure_elevated();
    }

    adapter::diag::log("post-elevation: continuing in main");

    // Initialize logging
    adapter::diag::log("initializing tracing_subscriber");
    tracing_subscriber::fmt().with_env_filter("info").init();
    adapter::diag::log("tracing_subscriber initialized");

    adapter::diag::log("Starting Rabbit application (diag)");
    info!("Starting Rabbit application");

    // Build a custom Tokio runtime with reduced thread stack size and worker count.
    // Default stack per thread is 8MB; with FLTK/Pango font threads also consuming
    // virtual memory, the default settings exhaust the address space under tight
    // ulimit -v constraints. 2MB stacks and 2 workers are sufficient for this app.
    adapter::diag::log("building tokio runtime");
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .thread_stack_size(2 * 1024 * 1024) // 2MB per thread
        .enable_all()
        .build()
    {
        Ok(r) => {
            adapter::diag::log("tokio runtime built");
            r
        }
        Err(e) => {
            adapter::diag::log(&format!("tokio runtime build FAILED: {e}"));
            return Err(e.into());
        }
    };

    adapter::diag::log("entering runtime.block_on");
    let result = runtime.block_on(async {
        adapter::diag::log("inside block_on: calling App::new()");
        let mut app = match App::new().await {
            Ok(a) => {
                adapter::diag::log("inside block_on: App::new() succeeded");
                a
            }
            Err(e) => {
                adapter::diag::log(&format!("inside block_on: App::new() FAILED: {e}"));
                return Err(e);
            }
        };
        adapter::diag::log("inside block_on: calling app.run()");
        let run_result = app.run().await;
        adapter::diag::log("inside block_on: app.run() returned");
        info!("Rabbit application exited");
        run_result
    });
        adapter::diag::log("runtime.block_on returned");
    result
}
