//! Rabbit Platform Abstraction Layer
//!
//! Provides platform-specific implementations for:
//! - Configuration storage
//! - Network interface enumeration
//! - ICMP ping (raw socket requirements)
//! - Taskbar/system tray integration
//! - Notifications

pub mod autostart;
pub mod config;
pub mod dialog;
pub mod network;
pub mod notification;
pub mod ping;
pub mod elevation;
pub mod shell;
pub mod taskbar;

pub use autostart::*;
pub use config::*;
pub use dialog::*;
pub use network::*;
pub use notification::*;
pub use ping::*;
pub use elevation::*;
pub use shell::*;
pub use taskbar::*;

use std::path::PathBuf;
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

/// Get the configuration directory for the application
pub fn config_dir() -> Result<PathBuf> {
    config::get_config_dir()
}

/// Get the data directory for the application
pub fn data_dir() -> Result<PathBuf> {
    config::get_data_dir()
}
