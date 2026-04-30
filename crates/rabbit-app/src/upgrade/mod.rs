//! Upgrade Management Module
//!
//! Handles version checking, downloading, verification, and replacement.

pub mod downloader;
pub mod installer;
pub mod models;

pub use downloader::{download_update, DownloadProgress};
pub use installer::{get_current_exe_path, install_update};
pub use models::{PlatformInfo, UpdateStatus, VersionsManifest};
