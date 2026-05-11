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
pub mod network;
pub mod notification;
pub mod ping;
pub mod shell;
pub mod taskbar;
pub mod window;
pub mod x11_diag;

pub use autostart::*;
pub use dialog::*;
pub use elevation::*;
pub use network::*;
pub use notification::*;
pub use ping::*;
pub use shell::*;
pub use taskbar::*;
pub use window::*;

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
