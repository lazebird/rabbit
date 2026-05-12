//! Upgrade Management Module
//!
//! Handles version checking, downloading, verification, replacement,
//! and UI flow orchestration.

pub mod downloader;
pub mod models;
pub mod ui;

pub use downloader::{download_update, DownloadProgress};
pub use models::{PlatformInfo, UpdateStatus, VersionsManifest};
pub use ui::handle_version_check_result;
