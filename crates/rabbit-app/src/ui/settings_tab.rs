//! Settings Tab UI Component
//!
//! Layout:
//! - All content left-aligned
//! - Auto-save on change (no Save button)

use fltk::{
    button::{Button, CheckButton},
    frame::Frame,
    group::Flex,
    menu::Choice,
    prelude::*,
    enums::Align,
    text::{TextBuffer, TextDisplay, WrapMode},
};

use crate::ui_events::{UiEvent, send_event};
use super::{TabComponent, Colors, Spacing};

/// Settings Tab Component
pub struct SettingsTab;

impl TabComponent for SettingsTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "Setting").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Row 1: Language
        let mut lang_row = Flex::default().row();
        lang_row.set_spacing(10);

        let mut lang_label = Frame::default().with_label("Language");
        lang_label.set_align(Align::Left | Align::Inside);
        lang_row.fixed(&lang_label, 70);

        let mut lang_choice = Choice::default();
        lang_choice.add_choice("English");
        lang_choice.add_choice("中文");
        lang_choice.add_choice("System");
        lang_choice.set_value(0);
        lang_row.fixed(&lang_choice, 100);

        Frame::default(); // Spacer
        lang_row.end();
        grp.fixed(&lang_row, spacing.row_height);

        // Row 2: Checkboxes (left-aligned, individual)
        let mut check_row = Flex::default().row();
        check_row.set_spacing(15);

        let mut tray_check = CheckButton::default().with_label("Tray");
        check_row.fixed(&tray_check, 55);

        let mut top_check = CheckButton::default().with_label("Top");
        check_row.fixed(&top_check, 50);

        let mut autostart_check = CheckButton::default().with_label("AutoStart");
        check_row.fixed(&autostart_check, 80);

        let mut autoupdate_check = CheckButton::default().with_label("AutoUpdate");
        autoupdate_check.set_checked(true);
        check_row.fixed(&autoupdate_check, 90);

        Frame::default(); // Spacer
        check_row.end();
        grp.fixed(&check_row, spacing.row_height);

        // Row 3: Version info (left) + Links (right-aligned: Home | Profile | Help)
        let mut info_row = Flex::default().row();
        info_row.set_spacing(10);

        // Version info (left side)
        let version_label = format!("sRabbit {}", CURRENT_VERSION);
        let mut version_frame = Frame::default().with_label(&version_label);
        version_frame.set_align(Align::Left | Align::Inside);
        version_frame.set_label_color(fltk::enums::Color::Blue);
        info_row.fixed(&version_frame, 120);

        Frame::default(); // Spacer to push buttons to the right

        let mut home_btn = Button::default().with_label("Home");
        home_btn.set_frame(fltk::enums::FrameType::FlatBox);
        home_btn.set_label_color(fltk::enums::Color::Blue);
        info_row.fixed(&home_btn, 50);

        let mut profile_btn = Button::default().with_label("Profile");
        profile_btn.set_frame(fltk::enums::FrameType::FlatBox);
        profile_btn.set_label_color(fltk::enums::Color::Blue);
        info_row.fixed(&profile_btn, 60);

        let mut help_btn = Button::default().with_label("Help");
        help_btn.set_frame(fltk::enums::FrameType::FlatBox);
        help_btn.set_label_color(fltk::enums::Color::Blue);
        info_row.fixed(&help_btn, 50);

        info_row.end();
        grp.fixed(&info_row, spacing.row_height);

        // Output area - fills remaining space
        let mut output_display = TextDisplay::default();
        let output_buf = TextBuffer::default();
        output_display.set_buffer(Some(output_buf));
        output_display.wrap_mode(WrapMode::AtBounds, 0);
        output_display.set_text_size(14);

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        output_display.set_color(colors.input_bg);
        output_display.set_text_color(colors.text);

        // Link callbacks - open URLs in browser
        home_btn.set_callback(|_| {
            open_url(HOME_URL);
        });

        profile_btn.set_callback(|_| {
            // Open config folder in file explorer
            let config_path = get_config_folder();
            // Ensure the directory exists
            if !std::path::Path::new(&config_path).exists() {
                let _ = std::fs::create_dir_all(&config_path);
            }
            open_folder(&config_path);
        });

        help_btn.set_callback(|_| {
            open_url(HELP_URL);
        });

        // Version check callback - click version info to check for updates
        let output_clone_version = output_display.clone();
        let mut version_frame_handle = version_frame.clone();
        version_frame_handle.handle(move |_, ev| {
            if ev == fltk::enums::Event::Push {
                // Copy version to clipboard
                let ver_text = format!("sRabbit {}", CURRENT_VERSION);
                fltk::app::copy(&ver_text);

                // Start version check in background
                let out = output_clone_version.clone();
                std::thread::spawn(move || {
                    // Write "Checking..." message
                    if let Some(mut buf) = out.buffer() {
                        buf.set_text("Checking for updates...\r\n");
                    }

                    let result = check_version_update();
                    if let Some(mut buf) = out.buffer() {
                        buf.set_text(&result);
                    }
                });
                true
            } else {
                false
            }
        });

        // Auto-save callbacks - save immediately on change
        let _lang_choice_clone = lang_choice.clone();
        lang_choice.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        tray_check.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        top_check.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        autostart_check.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        autoupdate_check.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        grp
    }
}

/// Open a URL in the system default browser.
fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg(url)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(url)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(url)
            .spawn();
    }
}

/// Open a folder in the system file explorer.
fn open_folder(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(path)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }
}

/// Current build information
const CURRENT_VERSION: &str = "1.0.0";

// URLs
const HOME_URL: &str = "https://codeup.aliyun.com/60e7f4fa52743a5162b61dd9/lazebird/rabbit/tree/master/release";
const HELP_URL: &str = "https://codeup.aliyun.com/60e7f4fa52743a5162b61dd9/lazebird/rabbit/blob/master/doc/manual.md";
const VERSION_CHECK_URL: &str = "https://codeup.aliyun.com/60e7f4fa52743a5162b61dd9/lazebird/rabbit/raw/master/release/version.txt";

// Config folder path - matches rabbit-platform::config::get_config_dir()
fn get_config_folder() -> String {
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("rabbit").to_string_lossy().to_string();
    }
    String::from(".")
}

/// Check for version updates by fetching remote version info.
/// Returns a message describing the update status.
fn check_version_update() -> String {
    // Try to fetch remote version info
    match fetch_remote_version() {
        Ok(remote_info) => {
            // Show the remote version info
            let mut msg = format!("Current version: {}\r\n", CURRENT_VERSION);
            msg.push_str(&format!("Remote build: {}\r\n", remote_info.get("date").unwrap_or(&"unknown".to_string())));
            msg.push_str(&format!("  Time: {}\r\n", remote_info.get("time").unwrap_or(&"".to_string())));
            msg.push_str(&format!("  Branch: {}\r\n", remote_info.get("branch").unwrap_or(&"".to_string())));
            if let Some(sha) = remote_info.get("sha") {
                msg.push_str(&format!("  Commit: {}\r\n", sha));
            }
            if let Some(extra) = remote_info.get("extra") {
                msg.push_str(&format!("  Extra: {}\r\n", extra));
            }
            msg.push_str("\r\nClick Home link to download the latest version.\r\n");
            msg
        }
        Err(e) => {
            // When offline or server unreachable, show current version info
            format!(
                "Current version: {}\r\n\
                \r\n\
                Unable to check for updates.\r\n\
                Reason: {}\r\n\
                \r\n\
                Visit the Home link to check for updates manually.\r\n",
                CURRENT_VERSION,
                e
            )
        }
    }
}

/// Fetch remote version info from the version.txt file.
fn fetch_remote_version() -> Result<std::collections::HashMap<String, String>, String> {
    // Use a simple HTTP GET request via subprocess
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("powershell")
            .arg("-Command")
            .arg(format!("(Invoke-WebRequest -Uri '{}').Content", VERSION_CHECK_URL))
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("curl")
            .arg("-s")
            .arg("-L")  // Follow redirects
            .arg(VERSION_CHECK_URL)
            .output()
            .map_err(|e| format!("Failed to run curl: {}", e))?
    } else {
        std::process::Command::new("wget")
            .arg("-q")
            .arg("-O-")
            .arg("--no-check-certificate")
            .arg(VERSION_CHECK_URL)
            .output()
            .map_err(|e| format!("Failed to run wget: {}", e))?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("HTTP request failed: {}", stderr.trim()));
    }

    let content = String::from_utf8_lossy(&output.stdout);
    if content.trim().is_empty() {
        return Err("Empty response from server".to_string());
    }
    parse_version_txt(&content)
}

/// Parse the version.txt file format into a HashMap.
/// Format:
///   date=2018/08/02
///   time=21:22:24
///   branch=master
///   sha=8085829
///   extra=Release
fn parse_version_txt(content: &str) -> Result<std::collections::HashMap<String, String>, String> {
    let mut map = std::collections::HashMap::new();
    for line in content.lines() {
        // Skip empty lines and lines without '='
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_string();
            let value = line[pos + 1..].trim().to_string();
            // Only add if key is non-empty
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }
    if map.is_empty() {
        return Err("Empty version info".to_string());
    }
    Ok(map)
}
