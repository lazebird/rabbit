//! Cross-platform privilege elevation module.
//! Required for binding privileged ports (e.g., TFTP port 69).

#[cfg(not(debug_assertions))]
use std::process::exit;

#[cfg(not(debug_assertions))]
use std::process::Command as StdCommand;

#[cfg(not(debug_assertions))]
use elevated_command::Command as ElevatedCommand;

/// Show an error dialog (Linux: zenity/kdialog/xmessage, Windows: MessageBox).
#[cfg(all(target_os = "linux", not(debug_assertions)))]
fn show_error_dialog(title: &str, message: &str) {
    let msg = format!("{}: {}", title, message);
    for (cmd, args) in [
        ("zenity", vec!["--error", "--title", title, "--text", message]),
        ("kdialog", vec!["--error", message, "--title", title]),
    ] {
        if StdCommand::new(cmd).args(&args).status().is_ok() {
            return;
        }
    }
    let _ = StdCommand::new("xmessage").arg("-center").arg(&msg).status();
    eprintln!("\n[ERROR] {}\n", msg);
}

#[cfg(all(target_os = "windows", not(debug_assertions)))]
fn show_error_dialog(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK};

    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    let msg_wide: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(std::ptr::null_mut(), msg_wide.as_ptr(), title_wide.as_ptr(), MB_OK | MB_ICONERROR);
    }
}

/// Check if already running with elevated privileges.
#[cfg(not(debug_assertions))]
fn is_elevated() -> bool {
    ElevatedCommand::is_elevated()
}

/// Restart with elevated privileges using elevated-command crate.
#[cfg(not(debug_assertions))]
fn restart_with_elevation() -> Result<(), String> {
    // For AppImage, use APPIMAGE env var to get the real path
    let exe_path = std::env::var("APPIMAGE")
        .ok()
        .or_else(|| std::env::args().next())
        .ok_or_else(|| "Failed to get executable path".to_string())?;

    ElevatedCommand::new(StdCommand::new(&exe_path))
        .output()
        .map_err(|e| format!("Failed to restart with elevated privileges: {}", e))?;

    Ok(())
}

/// Ensure elevated privileges. Skipped in debug mode for development.
/// Only applies to desktop platforms (Windows/Linux), not mobile.
pub fn ensure_elevated() {
    #[cfg(debug_assertions)]
    {
        return;
    }

    #[cfg(not(debug_assertions))]
    {
        // Skip elevation check on mobile platforms
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            return;
        }

        // Desktop platforms require elevation
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            if is_elevated() {
                return;
            }

            if let Err(e) = restart_with_elevation() {
                let hint = if cfg!(target_os = "linux") {
                    "\nHint: Install polkit (pkexec) for privilege elevation."
                } else {
                    "\nHint: Run as Administrator."
                };
                show_error_dialog("Privilege Elevation Failed", &format!("{}{}", e, hint));
                exit(1);
            }
            exit(0);
        }
    }
}
