//! Platform-specific installation logic

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Get current executable path
pub fn get_current_exe_path() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("Failed to get current exe path: {}", e))
}

/// Verify SHA256 checksum of downloaded file
fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = format!("{:x}", hasher.finalize());

    if result == expected.to_lowercase() {
        Ok(())
    } else {
        Err(format!("SHA256 mismatch: expected {}, got {}", expected, result))
    }
}

/// Install update by replacing current executable
/// This function:
/// 1. Verifies the downloaded file
/// 2. Creates a platform-specific update script
/// 3. Executes the script to replace the current binary and restart
pub fn install_update(new_version_path: &Path, expected_sha256: &str) -> Result<(), String> {
    // Step 1: Verify SHA256
    verify_sha256(new_version_path, expected_sha256)?;

    // Step 2: Get current executable path
    let current_exe = get_current_exe_path()?;
    let current_dir = current_exe.parent().ok_or("Failed to get current directory")?.to_path_buf();

    // Step 3: Create and execute platform-specific update script
    #[cfg(target_os = "windows")]
    {
        install_windows(new_version_path, current_exe, &current_dir)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        install_unix(new_version_path, current_exe, &current_dir)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported platform".to_string())
    }
}

#[cfg(target_os = "windows")]
fn install_windows(new_version_path: &Path, current_exe: PathBuf, current_dir: &Path) -> Result<(), String> {
    use std::fs;

    let update_dir = std::env::temp_dir().join("rabbit_update");
    fs::create_dir_all(&update_dir).map_err(|e| format!("Failed to create update directory: {}", e))?;

    // Copy new version to update directory
    let temp_exe = update_dir.join("rabbit.exe");
    fs::copy(new_version_path, &temp_exe).map_err(|e| format!("Failed to copy new version: {}", e))?;

    // Create update script
    let script_path = update_dir.join("update.bat");
    let script_content = format!(
        r#"@echo off
timeout /t 2 /nobreak >nul
copy /y "{}" "{}"
start "" "{}"
rmdir /s /q "{}"
"#,
        temp_exe.display(),
        current_exe.display(),
        current_exe.display(),
        update_dir.display()
    );

    fs::write(&script_path, script_content).map_err(|e| format!("Failed to write update script: {}", e))?;

    // Execute update script
    std::process::Command::new("cmd")
        .args(["/c", script_path.to_str().unwrap()])
        .spawn()
        .map_err(|e| format!("Failed to execute update script: {}", e))?;

    // Exit current process
    std::process::exit(0);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn install_unix(new_version_path: &Path, current_exe: PathBuf, _current_dir: &Path) -> Result<(), String> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let update_dir = PathBuf::from("/tmp/rabbit_update");
    fs::create_dir_all(&update_dir).map_err(|e| format!("Failed to create update directory: {}", e))?;

    // Copy new version to update directory
    let temp_exe = update_dir.join("rabbit");
    fs::copy(new_version_path, &temp_exe).map_err(|e| format!("Failed to copy new version: {}", e))?;

    // Set executable permission
    let mut perms = fs::metadata(&temp_exe).map_err(|e| format!("Failed to get file metadata: {}", e))?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&temp_exe, perms).map_err(|e| format!("Failed to set permissions: {}", e))?;

    // Create update script
    let script_path = update_dir.join("update.sh");
    let script_content = format!(
        r#"#!/bin/bash
sleep 2
cp -f "{}" "{}"
chmod +x "{}"
"{}" &
rm -rf /tmp/rabbit_update
"#,
        temp_exe.display(),
        current_exe.display(),
        current_exe.display(),
        current_exe.display()
    );

    fs::write(&script_path, script_content).map_err(|e| format!("Failed to write update script: {}", e))?;

    // Set script executable permission
    let mut script_perms = fs::metadata(&script_path).map_err(|e| format!("Failed to get script metadata: {}", e))?.permissions();
    script_perms.set_mode(0o755);
    fs::set_permissions(&script_path, script_perms).map_err(|e| format!("Failed to set script permissions: {}", e))?;

    // Execute update script
    std::process::Command::new("bash")
        .arg(&script_path)
        .spawn()
        .map_err(|e| format!("Failed to execute update script: {}", e))?;

    // Exit current process
    std::process::exit(0);
}
