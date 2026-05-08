//! Platform-specific autostart support

use super::PlatformError;
use super::Result;

/// Enable or disable auto-start on login/boot
pub fn set_autostart(enabled: bool) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        set_autostart_windows(enabled)
    }

    #[cfg(target_os = "linux")]
    {
        set_autostart_linux(enabled)
    }

    #[cfg(target_os = "macos")]
    {
        set_autostart_macos(enabled)
    }
}

#[cfg(target_os = "windows")]
fn set_autostart_windows(enabled: bool) -> Result<()> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    if enabled {
        // Add to Windows Run registry key
        let exe_path = std::env::current_exe().map_err(|e| PlatformError::Config(format!("Cannot get exe path: {}", e)))?;
        let exe_path_str = exe_path.to_string_lossy();

        let result = Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "Rabbit",
                "/t",
                "REG_SZ",
                "/d",
                &format!("\"{}\"", exe_path_str),
                "/f",
            ])
            .output();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(PlatformError::Config(format!("Failed to set autostart: {}", e))),
        }
    } else {
        // Remove from Windows Run registry key
        let result = Command::new("reg")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["delete", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run", "/v", "Rabbit", "/f"])
            .output();

        // Ignore errors if key doesn't exist
        match result {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

#[cfg(target_os = "linux")]
fn set_autostart_linux(enabled: bool) -> Result<()> {
    use std::fs;

    let autostart_dir = dirs::config_local_dir()
        .ok_or_else(|| PlatformError::Config("Cannot find config directory".to_string()))?
        .join("autostart");

    fs::create_dir_all(&autostart_dir)?;

    let desktop_file = autostart_dir.join("rabbit.desktop");

    if enabled {
        let exe_path = std::env::current_exe().map_err(|e| PlatformError::Config(format!("Cannot get exe path: {}", e)))?;
        let desktop_content = format!(
            "[Desktop Entry]\nType=Application\nName=Rabbit\nExec={}\nHidden=false\nNoDisplay=false\nX-GNOME-Autostart-enabled=true\n",
            exe_path.to_string_lossy()
        );
        fs::write(&desktop_file, desktop_content)?;
    } else {
        if desktop_file.exists() {
            fs::remove_file(&desktop_file)?;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_autostart_macos(enabled: bool) -> Result<()> {
    use std::fs;

    let plist_dir = dirs::config_local_dir()
        .ok_or_else(|| PlatformError::Config("Cannot find config directory".to_string()))?
        .join("LaunchAgents");

    fs::create_dir_all(&plist_dir)?;

    let plist_file = plist_dir.join("com.rabbit.plist");

    if enabled {
        let exe_path = std::env::current_exe().map_err(|e| PlatformError::Config(format!("Cannot get exe path: {}", e)))?;
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.rabbit</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
            exe_path.to_string_lossy()
        );
        fs::write(&plist_file, plist_content)?;
    } else {
        if plist_file.exists() {
            fs::remove_file(&plist_file)?;
        }
    }

    Ok(())
}
