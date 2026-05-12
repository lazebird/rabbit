//! Upgrade Management Module
//!
//! Handles version checking, downloading, verification, and replacement.

pub mod downloader;
pub mod models;

pub use downloader::{download_update, DownloadProgress};
pub use models::{PlatformInfo, UpdateStatus, VersionsManifest};
