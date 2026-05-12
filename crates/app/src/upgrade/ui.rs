//! Upgrade UI flow orchestration
//!
//! Handles version-check-result dispatch, upgrade dialog, and
//! background download / install with progress feedback.

use super::{PlatformInfo, VersionsManifest};
use adapter::Lifecycle;
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Handle version check result and show dialog (reused by both auto-check
/// and manual check).
pub fn handle_version_check_result(lifecycle: Option<&Lifecycle>) {
    match crate::ui::check_version_update() {
        crate::upgrade::UpdateStatus::UpdateAvailable(remote, platform_info) => {
            // Check shutdown flag before outputting
            if let Some(lc) = lifecycle {
                if lc.is_shutdown_requested() {
                    info!("Shutdown requested, skipping version update output");
                    return;
                }
            }

            info!("Update available: {}", remote.version);
            let msg = remote.format_prompt();
            crate::ui_state::write_to("settings_output", &crate::ui_state::raw_log(&msg));
            crate::ui_state::write_to("settings_output", &crate::ui_state::raw_log(""));

            // Show dialog on main thread
            fltk::app::awake_callback({
                let remote = remote.clone();
                let platform_info = platform_info.clone();
                move || {
                    show_upgrade_dialog(&remote, &platform_info);
                }
            });
        }
        crate::upgrade::UpdateStatus::UpToDate => {
            if let Some(lc) = lifecycle {
                if lc.is_shutdown_requested() {
                    return;
                }
            }

            info!("Application is up to date");
            crate::ui_state::write_to(
                "settings_output",
                &crate::ui_state::fmt_log("Application is up to date"),
            );
        }
        crate::upgrade::UpdateStatus::CheckError(e) => {
            if let Some(lc) = lifecycle {
                if lc.is_shutdown_requested() {
                    return;
                }
            }

            warn!("Failed to check for updates: {}", e);
            crate::ui_state::write_to(
                "settings_output",
                &crate::ui_state::fmt_log(&format!("Update check failed: {}", e)),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Show upgrade dialog and handle user choice (reused by both auto-check
/// and manual check).
fn show_upgrade_dialog(remote: &VersionsManifest, platform_info: &PlatformInfo) {
    let prompt = remote.format_prompt();
    let choice = fltk::dialog::choice2_default(&prompt, "Update", "Later", "Skip This Version");

    if choice == Some(0) {
        crate::ui_state::write_to(
            "settings_output",
            &crate::ui_state::fmt_log("Downloading and installing update..."),
        );
        // Spawn background thread for download/install to avoid blocking UI
        let remote = remote.clone();
        let platform_info = platform_info.clone();
        std::thread::spawn(move || {
            perform_startup_upgrade(&remote, &platform_info);
        });
    } else if choice == Some(1) {
        crate::ui_state::write_to(
            "settings_output",
            &crate::ui_state::fmt_log("Update deferred"),
        );
    } else if choice == Some(2) {
        crate::ui_state::write_to(
            "settings_output",
            &crate::ui_state::fmt_log(&format!("Version {} skipped", remote.version)),
        );
    }
}

/// Perform upgrade during startup (used by auto-check).
fn perform_startup_upgrade(remote: &VersionsManifest, platform_info: &PlatformInfo) {
    info!("Starting upgrade download for version: {}", remote.version);

    // Create temporary download path
    let temp_dir = std::env::temp_dir().join("rabbit_update");
    let _ = std::fs::create_dir_all(&temp_dir);
    let temp_exe = temp_dir.join(format!("rabbit-{}", remote.version));

    crate::ui_state::write_to(
        "settings_output",
        &crate::ui_state::fmt_log(&format!(
            "Downloading: {:.1} MB",
            platform_info.size as f64 / 1024.0 / 1024.0
        )),
    );

    // Download with progress
    let result = crate::upgrade::download_update(
        platform_info,
        &temp_exe,
        Some(&|progress: crate::upgrade::DownloadProgress| {
            let pct = progress.percentage;
            let downloaded_mb = progress.downloaded as f64 / 1024.0 / 1024.0;
            let total_mb = progress.total as f64 / 1024.0 / 1024.0;
            info!(
                "Downloading: {:.1} MB / {:.1} MB ({:.0}%)",
                downloaded_mb, total_mb, pct
            );
            crate::ui_state::update_settings_line(
                -1,
                &format!("  {:.0}% - {:.1} MB / {:.1} MB", pct, downloaded_mb, total_mb),
            );
        }),
    );

    match result {
        Ok(_) => {
            info!("Download complete. Verifying and installing...");
            crate::ui_state::write_to(
                "settings_output",
                &crate::ui_state::fmt_log("Download complete. Verifying and installing..."),
            );
            match adapter::installer::install_update(&temp_exe, &platform_info.sha256) {
                Ok(_) => {
                    // install_update calls std::process::exit(), so we won't reach here
                }
                Err(e) => {
                    error!("Installation failed: {}", e);
                    crate::ui_state::write_to(
                        "settings_output",
                        &crate::ui_state::fmt_log(&format!("Installation failed: {}", e)),
                    );
                }
            }
        }
        Err(e) => {
            error!("Download failed: {}", e);
            crate::ui_state::write_to(
                "settings_output",
                &crate::ui_state::fmt_log(&format!("Download failed: {}", e)),
            );
        }
    }
}
