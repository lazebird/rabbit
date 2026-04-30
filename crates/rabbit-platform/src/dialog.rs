//! Platform-specific file dialog support
//!
//! Provides native file/folder selection dialogs

use crate::{PlatformError, Result};
use std::path::PathBuf;

/// Show a file selection dialog
///
/// # Arguments
/// * `title` - Dialog window title
/// * `default_path` - Initial directory path (optional)
/// * `filters` - File type filters, e.g., [("Images", &["png", "jpg"])]
///
/// # Returns
/// Selected file path or None if cancelled
pub fn open_file_dialog(title: &str, default_path: Option<&str>, _filters: &[(&str, &[&str])]) -> Result<Option<PathBuf>> {
    // Platform-specific implementation
    #[cfg(target_os = "linux")]
    {
        open_file_dialog_linux(title, default_path)
    }

    #[cfg(target_os = "windows")]
    {
        open_file_dialog_windows(title, default_path)
    }

    #[cfg(target_os = "macos")]
    {
        open_file_dialog_macos(title, default_path)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(PlatformError::NotSupported)
    }
}

/// Show a folder selection dialog
///
/// # Arguments
/// * `title` - Dialog window title
/// * `default_path` - Initial directory path (optional)
///
/// # Returns
/// Selected folder path or None if cancelled
pub fn open_folder_dialog(title: &str, default_path: Option<&str>) -> Result<Option<PathBuf>> {
    #[cfg(target_os = "linux")]
    {
        open_folder_dialog_linux(title, default_path)
    }

    #[cfg(target_os = "windows")]
    {
        open_folder_dialog_windows(title, default_path)
    }

    #[cfg(target_os = "macos")]
    {
        open_folder_dialog_macos(title, default_path)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(PlatformError::NotSupported)
    }
}

/// Open a URL in the system default browser
pub fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn().map_err(PlatformError::Io)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd").args(["/C", "start", url]).spawn().map_err(PlatformError::Io)?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn().map_err(PlatformError::Io)?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(PlatformError::NotSupported)
    }
}

/// Open a file manager at the specified path
pub fn open_file_manager(path: &str) -> Result<()> {
    let path = std::path::Path::new(path);
    let path_str = path.to_str().unwrap_or(".");

    #[cfg(target_os = "linux")]
    {
        // Try different file managers
        let managers = ["xdg-open", "nautilus", "dolphin", "thunar", "pcmanfm"];
        for manager in &managers {
            if std::process::Command::new(manager).arg(path_str).spawn().is_ok() {
                return Ok(());
            }
        }
        Err(PlatformError::NotSupported)
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer").arg(path_str).spawn().map_err(|e| PlatformError::Io(e))?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path_str).spawn().map_err(|e| PlatformError::Io(e))?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(PlatformError::NotSupported)
    }
}

// Linux implementations using zenity or kdialog
#[cfg(target_os = "linux")]
fn open_file_dialog_linux(title: &str, default_path: Option<&str>) -> Result<Option<PathBuf>> {
    // Try zenity first
    let mut cmd = std::process::Command::new("zenity");
    cmd.arg("--file-selection").arg("--title").arg(title);

    if let Some(path) = default_path {
        cmd.arg("--filename").arg(path);
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                let path = path.trim();
                if path.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(PathBuf::from(path)))
                }
            } else {
                Ok(None) // User cancelled
            }
        }
        Err(_) => {
            // Fallback to kdialog
            open_file_dialog_kdialog(title, default_path)
        }
    }
}

#[cfg(target_os = "linux")]
fn open_folder_dialog_linux(title: &str, default_path: Option<&str>) -> Result<Option<PathBuf>> {
    let mut cmd = std::process::Command::new("zenity");
    cmd.arg("--file-selection").arg("--directory").arg("--title").arg(title);

    if let Some(path) = default_path {
        cmd.arg("--filename").arg(path);
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                let path = path.trim();
                if path.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(PathBuf::from(path)))
                }
            } else {
                Ok(None)
            }
        }
        Err(_) => open_folder_dialog_kdialog(title, default_path),
    }
}

#[cfg(target_os = "linux")]
fn open_file_dialog_kdialog(title: &str, default_path: Option<&str>) -> Result<Option<PathBuf>> {
    let mut cmd = std::process::Command::new("kdialog");
    cmd.arg("--getopenfilename").arg("--title").arg(title);

    if let Some(path) = default_path {
        cmd.arg(path);
    } else {
        cmd.arg(".");
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                let path = path.trim();
                if path.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(PathBuf::from(path)))
                }
            } else {
                Ok(None)
            }
        }
        Err(e) => Err(PlatformError::Io(e)),
    }
}

#[cfg(target_os = "linux")]
fn open_folder_dialog_kdialog(title: &str, default_path: Option<&str>) -> Result<Option<PathBuf>> {
    let mut cmd = std::process::Command::new("kdialog");
    cmd.arg("--getexistingdirectory").arg("--title").arg(title);

    if let Some(path) = default_path {
        cmd.arg(path);
    } else {
        cmd.arg(".");
    }

    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                let path = path.trim();
                if path.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(PathBuf::from(path)))
                }
            } else {
                Ok(None)
            }
        }
        Err(e) => Err(PlatformError::Io(e)),
    }
}

// Windows implementations
#[cfg(target_os = "windows")]
fn open_file_dialog_windows(_title: &str, _default_path: Option<&str>) -> Result<Option<PathBuf>> {
    // Windows file dialog requires COM/OLE which is complex
    // For now, return not supported - can use rfd crate later
    Err(PlatformError::NotSupported)
}

#[cfg(target_os = "windows")]
fn open_folder_dialog_windows(_title: &str, _default_path: Option<&str>) -> Result<Option<PathBuf>> {
    Err(PlatformError::NotSupported)
}

// macOS implementations
#[cfg(target_os = "macos")]
fn open_file_dialog_macos(_title: &str, _default_path: Option<&str>) -> Result<Option<PathBuf>> {
    // macOS file dialog requires Cocoa framework
    // For now, return not supported - can use rfd crate later
    Err(PlatformError::NotSupported)
}

#[cfg(target_os = "macos")]
fn open_folder_dialog_macos(_title: &str, _default_path: Option<&str>) -> Result<Option<PathBuf>> {
    Err(PlatformError::NotSupported)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_open_url() {
        // This would actually open a browser, so we just test the function exists
        // In real tests, we'd mock the Command
    }
}
