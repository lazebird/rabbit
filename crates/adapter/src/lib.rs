//! Rabbit Platform Abstraction Layer
//!
//! Provides platform-specific implementations for:
//! - Configuration storage
//! - Network interface enumeration
//! - ICMP ping (raw socket requirements)
//! - Taskbar/system tray integration
//! - Notifications

pub mod autostart;
pub mod dialog;
pub mod elevation;
pub mod icon;
pub mod installer;
pub mod lifecycle;
pub mod network;
pub mod notification;
pub mod ping;
pub mod platform;
pub mod shell;
pub mod systray;
pub mod taskbar;
pub mod tray;
#[cfg(target_os = "linux")]
pub mod tray_helper;
pub mod window;
pub mod x11_diag;

pub use autostart::*;
pub use dialog::*;
pub use elevation::*;
pub use icon::IconData;
pub use lifecycle::Lifecycle;
pub use network::*;
pub use notification::*;
pub use ping::*;
pub use platform::*;
pub use shell::*;
pub use taskbar::*;

/// Install platform-specific error / diagnostic handlers.
///
/// Currently only installs the Linux X11 error handler for diagnostic
/// logging. Called once during application startup.
pub fn install_platform_handlers() {
    #[cfg(target_os = "linux")]
    crate::x11_diag::install();
}

/// If the process was spawned as `--tray-helper`, run the helper loop and exit.
///
/// On non-Linux platforms this is a no-op — the helper process concept
/// is Linux-specific (D-Bus tray ownership after `sudo` elevation).
pub fn maybe_run_as_helper() {
    #[cfg(target_os = "linux")]
    crate::tray_helper::maybe_run_as_helper();
}

/// Platform-specific pre-initialisation before privilege elevation.
///
/// On Linux (release build, non-elevated): spawns the tray helper
/// subprocess so it can own the D-Bus tray icon as the original user.
/// On other platforms or debug builds: no-op.
pub fn pre_main_init() {
    #[cfg(target_os = "linux")]
    crate::tray_helper::pre_main_init();
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Not supported on this platform")]
    NotSupported,
}

pub type Result<T> = std::result::Result<T, PlatformError>;
