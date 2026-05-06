//! File download functionality

use super::models::PlatformInfo;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// Download progress callback data
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percentage: f64,
}

/// Download update file with progress tracking
pub fn download_update(platform_info: &PlatformInfo, dest: &Path, progress_cb: Option<&dyn Fn(DownloadProgress)>) -> Result<u64, String> {
    // Create temporary directory for download
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create download directory: {}", e))?;
    }

    // Download file using ureq
    let response = ureq::get(&platform_info.url)
        .timeout(std::time::Duration::from_secs(300))
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;

    // Read response body
    let mut reader = response.into_reader();
    let mut file = File::create(dest).map_err(|e| format!("Failed to create file: {}", e))?;

    let mut buffer = [0u8; 8192];
    let mut downloaded: u64 = 0;
    let total = platform_info.size;

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| format!("Read error: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read]).map_err(|e| format!("Write error: {}", e))?;

        downloaded += bytes_read as u64;

        // Report progress
        if let Some(cb) = progress_cb {
            let percentage = if total > 0 { (downloaded as f64 / total as f64) * 100.0 } else { 0.0 };

            cb(DownloadProgress { downloaded, total, percentage });
        }
    }

    file.flush().map_err(|e| format!("Flush error: {}", e))?;

    Ok(downloaded)
}
