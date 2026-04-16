//! Platform notification support

use super::Result;
use rabbit_models::plan::Task;

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
    // Windows notification implementation
    // Could use winrt-notification or windows crate
    println!("[Notification] {}: {}", title, message);
    Ok(())
}

#[cfg(target_os = "linux")]
fn show_notification_linux(title: &str, message: &str) -> Result<()> {
    // Try notify-send first
    use std::process::Command;
    
    let result = Command::new("notify-send")
        .args(&[title, message])
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

#[cfg(target_os = "macos")]
fn show_notification_macos(title: &str, message: &str) -> Result<()> {
    use std::process::Command;
    
    let result = Command::new("osascript")
        .args(&[
            "-e",
            &format!(r#"display notification "{}" with title "{}""#, message, title)
        ])
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
pub fn show_task_reminder(task: &Task) -> Result<()> {
    show_notification(
        &format!("Reminder: {}", task.title),
        task.description.as_deref().unwrap_or("Time's up!")
    )
}
