//! Platform notification support

use super::Result;

/// Show a notification
pub fn show_notification(title: &str, message: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        show_notification_windows(title, message)
    }

    #[cfg(target_os = "linux")]
    {
        show_notification_linux(title, message)
    }

    #[cfg(target_os = "macos")]
    {
        show_notification_macos(title, message)
    }
}

#[cfg(target_os = "windows")]
fn show_notification_windows(title: &str, message: &str) -> Result<()> {
    // Use PowerShell to show a Windows 10+ toast notification
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // Escape strings for PowerShell
    let ps_title = title.replace('"', "\\\"").replace('\n', " ");
    let ps_message = message.replace('"', "\\\"").replace('\n', " ");

    // PowerShell script to show toast notification using BALLOON tip (simpler, more reliable)
    let script = format!(
        r#"
        Add-Type -AssemblyName System.Windows.Forms
        $balloon = New-Object System.Windows.Forms.NotifyIcon
        $balloon.Icon = [System.Drawing.SystemIcons]::Information
        $balloon.BalloonTipIcon = [System.Windows.Forms.ToolTipIcon]::Info
        $balloon.BalloonTipTitle = "{}"
        $balloon.BalloonTipText = "{}"
        $balloon.Visible = $true
        $balloon.ShowBalloonTip(5000)
        Start-Sleep -Seconds 2
        $balloon.Dispose()
        "#,
        ps_title, ps_message
    );

    let result = Command::new("powershell")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .args(&["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .output();

    match result {
        Ok(_) => Ok(()),
        Err(_) => {
            // Fallback to console
            println!("[Notification] {}: {}", title, message);
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
fn show_notification_linux(title: &str, message: &str) -> Result<()> {
    // Try notify-send first
    use std::process::Command;

    let result = Command::new("notify-send").args([title, message]).output();

    match result {
        Ok(_) => Ok(()),
        Err(_) => {
            // Fallback to console
            println!("[Notification] {}: {}", title, message);
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
fn show_notification_macos(title: &str, message: &str) -> Result<()> {
    use std::process::Command;

    let result = Command::new("osascript")
        .args(&["-e", &format!(r#"display notification "{}" with title "{}""#, message, title)])
        .output();

    match result {
        Ok(_) => Ok(()),
        Err(_) => {
            println!("[Notification] {}: {}", title, message);
            Ok(())
        }
    }
}

/// Show task reminder notification
pub fn show_task_reminder(title: &str, description: Option<&str>) -> Result<()> {
    show_notification(&format!("Reminder: {}", title), description.unwrap_or("Time's up!"))
}
