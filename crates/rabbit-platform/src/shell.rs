//! Windows Shell Context Menu Integration
//!
//! Implements right-click menu integration for Windows Explorer:
//! - Register/unregister file context menu
//! - Register/unregister directory context menu
//! - Uses Windows registry to add "Open with Rabbit HTTP Server" option

#[cfg(target_os = "windows")]
mod windows {
    use std::process::Command;

    /// Register HTTP server context menu for directories
    pub fn register_directory_context(exe_path: &str) -> Result<(), String> {
        // Registry path for directory context menu
        let reg_path = r"HKEY_CLASSES_ROOT\Directory\shell\RabbitHTTPServer";

        // Set display name for the menu item
        Command::new("reg")
            .args(&["add", reg_path, "/ve", "/d", "Open with Rabbit HTTP Server", "/f"])
            .output()
            .map_err(|e| format!("Failed to register context menu: {}", e))?;

        // Set command to execute
        let cmd_path = format!("{}\\command", reg_path);
        Command::new("reg")
            .args(&["add", &cmd_path, "/ve", "/d", &format!("\"{}\" --http-server \"%1\"", exe_path), "/f"])
            .output()
            .map_err(|e| format!("Failed to register command: {}", e))?;

        Ok(())
    }

    /// Unregister HTTP server context menu for directories
    pub fn unregister_directory_context() -> Result<(), String> {
        let reg_path = r"HKEY_CLASSES_ROOT\Directory\shell\RabbitHTTPServer";

        Command::new("reg")
            .args(&["delete", reg_path, "/f"])
            .output()
            .map(|_| ())
            .map_err(|e| format!("Failed to unregister context menu: {}", e))
    }

    /// Register HTTP server context menu for files
    pub fn register_file_context(exe_path: &str) -> Result<(), String> {
        let reg_path = r"HKEY_CLASSES_ROOT\*\shell\RabbitHTTPServer";

        Command::new("reg")
            .args(&["add", reg_path, "/ve", "/d", "Open with Rabbit HTTP Server", "/f"])
            .output()
            .map_err(|e| format!("Failed to register file context menu: {}", e))?;

        let cmd_path = format!("{}\\command", reg_path);
        Command::new("reg")
            .args(&["add", &cmd_path, "/ve", "/d", &format!("\"{}\" --http-server \"%1\"", exe_path), "/f"])
            .output()
            .map_err(|e| format!("Failed to register file command: {}", e))?;

        Ok(())
    }

    /// Unregister HTTP server context menu for files
    pub fn unregister_file_context() -> Result<(), String> {
        let reg_path = r"HKEY_CLASSES_ROOT\*\shell\RabbitHTTPServer";

        Command::new("reg")
            .args(&["delete", reg_path, "/f"])
            .output()
            .map(|_| ())
            .map_err(|e| format!("Failed to unregister file context menu: {}", e))
    }

    /// Toggle shell integration based on enabled flag
    pub fn set_shell_integration(enabled: bool, exe_path: &str) -> Result<(), String> {
        if enabled {
            // Register both directory and file context menus
            register_directory_context(exe_path)?;
            register_file_context(exe_path)?;
        } else {
            // Unregister both
            unregister_directory_context().ok();
            unregister_file_context().ok();
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub use windows::set_shell_integration;

#[cfg(not(target_os = "windows"))]
mod non_windows {
    /// Placeholder for non-Windows platforms
    pub fn set_shell_integration(_enabled: bool, _exe_path: &str) -> Result<(), String> {
        // No-op on non-Windows platforms
        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub use non_windows::set_shell_integration;
