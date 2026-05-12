//! Unified system tray lifecycle management.
//!
//! Encapsulates platform-specific tray logic:
//! - Linux: prefers tray_helper (IPC subprocess for D-Bus after elevation),
//!   falls back to local ksni when helper is unavailable.
//! - Other platforms: native tray via tray-icon crate.

use crate::Lifecycle;
use tracing::warn;

#[cfg(target_os = "linux")]
use fltk::prelude::WidgetExt;

/// Initialize the system tray at startup.
///
/// On Linux this first attempts to connect to an existing tray helper
/// subprocess (spawned before elevation).  If the helper socket exists
/// the parent connects and tells the helper to show/hide the tray.
/// Without a helper, or on non-Linux, the local systray is initialised
/// directly.
pub async fn init(lifecycle: &Lifecycle, enabled: bool, main_win: fltk::window::Window) {
    // On non-Linux `main_win` is unused; consume it explicitly to suppress clippy.
    #[cfg(not(target_os = "linux"))]
    let _ = main_win;

    rabbit_diag::log(&format!("systray::init(enabled={enabled})"));

    #[cfg(target_os = "linux")]
    {
        let has_sock = crate::tray_helper::has_helper_socket();
        rabbit_diag::log(&format!("systray::init: has_helper_socket={has_sock}"));

        let helper_connected = if has_sock {
            let win = main_win.clone();
            let show = move || win.clone().show();
            let hide = move || main_win.clone().hide();
            let connected = crate::tray_helper::connect(lifecycle.clone(), show, hide);
            rabbit_diag::log(&format!("systray::init: connect returned {connected}"));
            connected
        } else {
            false
        };

        if helper_connected {
            if enabled {
                rabbit_diag::log("systray::init: helper connected + enabled → show_tray");
                crate::tray_helper::show_tray();
            } else {
                rabbit_diag::log("systray::init: helper connected + disabled → no tray");
            }
        } else if enabled {
            rabbit_diag::log("systray::init: no helper, falling back to local ksni");
            if let Err(e) = crate::tray::init_systray(lifecycle.clone()).await {
                warn!("Failed to initialize systray: {e}");
                rabbit_diag::log(&format!("systray::init: init_systray failed: {e}"));
            } else {
                rabbit_diag::log("systray::init: init_systray succeeded");
            }
        } else {
            rabbit_diag::log("systray::init: no helper, disabled → nothing");
        }
    }

    #[cfg(not(target_os = "linux"))]
    if enabled {
        if let Err(e) = crate::tray::init_systray(lifecycle.clone()).await {
            warn!("Failed to initialize systray: {e}");
        }
    }
}

/// Cleanup the system tray on app exit.
///
/// On Linux: shut down the tray helper IPC connection (if any) and
/// remove the local ksni tray.  On other platforms: remove the tray.
pub fn cleanup() {
    rabbit_diag::log("systray::cleanup");

    #[cfg(target_os = "linux")]
    {
        rabbit_diag::log("systray::cleanup: calling shutdown_helper");
        crate::tray_helper::shutdown_helper();
    }

    rabbit_diag::log("systray::cleanup: calling remove_systray");
    crate::tray::remove_systray();
}

/// Update the system tray enabled state at runtime (settings toggle).
///
/// On Linux with an active helper connection the show/hide is sent over
/// IPC so the helper process manages the tray without restarting.
/// Otherwise (no helper or non-Linux) the local systray is toggled.
pub async fn update(enabled: bool) {
    rabbit_diag::log(&format!("systray::update(enabled={enabled})"));

    #[cfg(target_os = "linux")]
    if crate::tray_helper::is_connected() {
        rabbit_diag::log("systray::update: helper connected, using IPC");
        if enabled {
            crate::tray_helper::show_tray();
        } else {
            crate::tray_helper::hide_tray();
        }
        return;
    }

    crate::tray::update_systray(enabled).await;
}
