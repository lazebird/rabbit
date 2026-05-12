//! Version and platform data models

use serde::Deserialize;
use std::collections::HashMap;

/// Remote version information (unified version scheme)
#[derive(Debug, Clone, Deserialize)]
pub struct VersionsManifest {
    /// Unified version number (all platforms share the same version)
    pub version: String,
    /// Release date (YYYY/MM/DD)
    pub release_date: String,
    /// Release notes
    pub release_notes: String,
    /// Platform-specific download info
    pub platforms: HashMap<String, PlatformInfo>,
}

/// Platform-specific download information
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformInfo {
    /// SHA256 checksum (hex string)
    pub sha256: String,
    /// File size in bytes
    pub size: u64,
    /// Download URL
    pub url: String,
}

impl VersionsManifest {
    /// Get info for current platform
    pub fn for_current_platform(&self) -> Option<&PlatformInfo> {
        let platform = current_platform();
        self.platforms.get(platform)
    }

    /// Check if this version is newer than current using semver
    pub fn is_newer_than(&self, current: &str) -> bool {
        let current_ver = semver::Version::parse(current).ok();
        let remote_ver = semver::Version::parse(&self.version).ok();

        match (current_ver, remote_ver) {
            (Some(curr), Some(remote)) => remote > curr,
            _ => !self.version.is_empty(),
        }
    }

    /// Format release info for display (compact)
    pub fn format_summary(&self) -> String {
        let mut s = format!("Version: {}  ({})", self.version, self.release_date);
        if let Some(info) = self.for_current_platform() {
            s.push_str(&format!(", {:.1} MB", info.size as f64 / 1024.0 / 1024.0));
        }
        if !self.release_notes.is_empty() {
            s.push_str(&format!("\n{}", self.release_notes));
        }
        s
    }

    /// Format for dialog prompt (matches log output format)
    pub fn format_prompt(&self) -> String {
        let mut s = format!("Update available: {} ({})", self.version, self.release_date);
        if let Some(info) = self.for_current_platform() {
            s.push_str(&format!(", {:.1} MB", info.size as f64 / 1024.0 / 1024.0));
        }
        // Add release notes
        let notes = self.release_notes.trim();
        if !notes.is_empty() {
            s.push_str(&format!("\n\n{}", notes));
        }
        s
    }
}

/// Update status after checking
pub enum UpdateStatus {
    /// Already up to date
    UpToDate,
    /// Update available with platform info
    UpdateAvailable(VersionsManifest, PlatformInfo),
    /// Error during check
    CheckError(String),
}

/// Download progress callback data
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percentage: f64,
}

/// Detect current platform identifier — delegates to adapter layer
fn current_platform() -> &'static str {
    adapter::platform::current_platform()
}
