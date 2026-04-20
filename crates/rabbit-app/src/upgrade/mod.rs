//! Upgrade Management Module
//!
//! Handles version checking, downloading, verification, and replacement.

pub mod downloader;
pub mod installer;
pub mod models;

pub use models::{UpdateStatus, VersionsManifest, PlatformInfo};
pub use downloader::{download_update, DownloadProgress};
pub use installer::{install_update, get_current_exe_path};
