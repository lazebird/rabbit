//! Application lifecycle controller.
//!
//! Provides a unified abstraction for application hide/exit state management.
//! Replaces the previous approach of separate `shutdown_flag: Arc<AtomicBool>`
//! in `app.rs` and `QUIT_REQUESTED` static in `systray_linux.rs` with a single
//! `Lifecycle` handle that can be cloned and shared across all exit paths:
//!
//! - ESC key / window close button
//! - Ctrl+C signal
//! - System tray "Quit" menu
//!
//! # Usage
//!
//! ```ignore
//! let lifecycle = Lifecycle::new();
//! let lc = lifecycle.clone();
//! ctrlc::set_handler(move || {
//!     lc.request_shutdown();
//!     fltk::app::awake_callback(|| fltk::app::quit());
//! }).ok();
//!
//! // In the main event loop:
//! lifecycle.run_event_loop();
//! // Returns when shutdown is requested.
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared lifecycle controller for application hide/exit management.
///
/// Cloning is cheap (`Arc`-based) — pass it to systray, signal handlers,
/// and any other component that needs to signal or check shutdown state.
#[derive(Clone)]
pub struct Lifecycle {
    inner: Arc<Inner>,
}

struct Inner {
    shutdown_requested: AtomicBool,
}

impl Lifecycle {
    /// Create a new lifecycle controller (initial state: running).
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                shutdown_requested: AtomicBool::new(false),
            }),
        }
    }

    /// Request a graceful shutdown.
    ///
    /// Safe to call from any thread. After this, `is_shutdown_requested()`
    /// returns `true` and `run_event_loop()` will exit on its next check.
    pub fn request_shutdown(&self) {
        self.inner.shutdown_requested.store(true, Ordering::SeqCst);
    }

    /// Check whether a shutdown has been requested.
    pub fn is_shutdown_requested(&self) -> bool {
        self.inner.shutdown_requested.load(Ordering::SeqCst)
    }

    /// Run the FLTK event loop with shutdown-aware blocking.
    ///
    /// Uses `fltk::app::wait_for(0.05)` which blocks for up to 50 ms
    /// waiting for events (user input, `awake_callback` from tray, etc.).
    /// Unlike plain `wait()`, this does **not** return immediately when
    /// all windows are hidden — required for tray "Hide" support.
    ///
    /// Returns when `is_shutdown_requested()` is `true` or the event
    /// loop encounters a fatal error.
    pub fn run_event_loop(&self) {
        loop {
            match fltk::app::wait_for(0.05) {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("FLTK event loop interrupted: {e}");
                    break;
                }
            }
            if self.is_shutdown_requested() {
                break;
            }
        }
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initially_not_shutdown() {
        let lc = Lifecycle::new();
        assert!(!lc.is_shutdown_requested());
    }

    #[test]
    fn shutdown_after_request() {
        let lc = Lifecycle::new();
        lc.request_shutdown();
        assert!(lc.is_shutdown_requested());
    }

    #[test]
    fn clone_shares_state() {
        let lc = Lifecycle::new();
        let lc2 = lc.clone();
        lc2.request_shutdown();
        assert!(lc.is_shutdown_requested());
    }
}
