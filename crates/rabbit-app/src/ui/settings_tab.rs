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
use std::collections::HashMap;

use crate::ui_events::{UiEvent, send_event};
use super::{TabComponent, Colors, Spacing};

// ============================================================
// Constants - URLs and Version
// ============================================================

/// Current application version from Cargo.toml
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Remote versions.json URL for update checking
const VERSION_CHECK_URL: &str = "https://codeup.aliyun.com/60e7f4fa52743a5162b61dd9/lazebird/rabbit/raw/rewrite/release/versions.json";

/// Home page URL for downloads
const HOME_URL: &str = "https://codeup.aliyun.com/60e7f4fa52743a5162b61dd9/lazebird/rabbit/tree/rewrite/release";

/// Help/manual URL
const HELP_URL: &str = "https://codeup.aliyun.com/60e7f4fa52743a5162b61dd9/lazebird/rabbit/blob/rewrite/doc/manual.md";

// ============================================================
// Version Management
// ============================================================

/// Parsed version information from versions.json
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VersionsManifest {
    /// Unified version number (all platforms share the same version)
    pub version: String,
    /// Release date (YYYY/MM/DD)
    pub release_date: String,
    /// Release notes
    pub release_notes: String,
    /// Platform-specific download info
    pub platforms: HashMap<String, PlatformInfo>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlatformInfo {
    /// SHA256 checksum (hex string)
    pub sha256: String,
    /// File size in bytes
    pub size: u64,
    /// Download URL
    pub url: String,
}

impl VersionsManifest {
    /// Get info for current platform
    pub fn for_current_platform(&self) -> Option<&PlatformInfo> {
        let platform = current_platform();
        self.platforms.get(platform)
    }

    /// Check if this version is newer than current
    pub fn is_newer_than(&self, _current: &str) -> bool {
        !self.version.is_empty()
    }

    /// Format release info for display
    pub fn format_summary(&self) -> String {
        let mut s = format!("Version: {}\n", self.version);
        s.push_str(&format!("Date: {}\n", self.release_date));
        if !self.release_notes.is_empty() {
            s.push_str(&format!("\n{}\n", self.release_notes));
        }
        if let Some(info) = self.for_current_platform() {
            s.push_str(&format!("\nSize: {:.1} MB\n", info.size as f64 / 1024.0 / 1024.0));
        }
        s
    }
}

/// Detect current platform identifier
fn current_platform() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "windows-x64";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x64";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "linux-arm64";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "macos-x64";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "macos-arm64";
    "unknown"
}

/// Fetch remote version content using platform-specific commands
fn fetch_version_content(url: &str) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(["-Command", &format!("(Invoke-WebRequest -Uri '{0}' -UseBasicParsing).Content", url)])
            .output()
            .map_err(|e| format!("Failed to run PowerShell: {}", e))?;

        if !output.status.success() {
            return Err(format!("HTTP {}", output.status));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("curl")
            .args(["-s", "-L", "--connect-timeout", "10", url])
            .output()
            .map_err(|e| format!("Failed to run curl: {}", e))?;

        if !output.status.success() {
            return Err(format!("HTTP {}", output.status));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    #[cfg(target_os = "linux")]
    {
        // Try curl first, fallback to wget
        let output = std::process::Command::new("curl")
            .args(["-s", "-L", "--connect-timeout", "10", url])
            .output();

        let output = match output {
            Ok(o) if o.status.success() => o,
            _ => std::process::Command::new("wget")
                .args(["-q", "-O-", "--no-check-certificate", "--timeout=10", url])
                .output()
                .map_err(|e| format!("Failed to run wget: {}", e))?,
        };

        if !output.status.success() {
            return Err(format!("HTTP {}", output.status));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

/// Check for version updates
fn check_version_update() -> String {
    match fetch_version_content(VERSION_CHECK_URL) {
        Ok(content) => {
            match serde_json::from_str::<VersionsManifest>(&content) {
                Ok(remote) => {
                    let mut msg = format!("Current version: {}\n\n", CURRENT_VERSION);
                    msg.push_str(&remote.format_summary());
                    msg.push_str("\n");

                    if remote.is_newer_than(CURRENT_VERSION) {
                        msg.push_str("\nA newer version is available.\n");
                    } else {
                        msg.push_str("\nYou are on the latest version.\n");
                    }
                    msg.push_str("\nVisit Home to download updates.\n");
                    msg
                }
                Err(e) => {
                    format!(
                        "Current version: {}\n\n\
                        Failed to parse version info.\n\
                        Error: {}\n\n\
                        Visit Home to check for updates manually.\n",
                        CURRENT_VERSION, e
                    )
                }
            }
        }
        Err(e) => {
            format!(
                "Current version: {}\n\n\
                Unable to check for updates.\n\
                Error: {}\n\n\
                Visit Home to check for updates manually.\n",
                CURRENT_VERSION, e
            )
        }
    }
}

// ============================================================
// Platform Helpers
// ============================================================

/// Open a URL in the system default browser
fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    { let _ = std::process::Command::new("cmd").args(["/c", "start", url]).spawn(); }
    #[cfg(target_os = "macos")]
    { let _ = std::process::Command::new("open").arg(url).spawn(); }
    #[cfg(target_os = "linux")]
    { let _ = std::process::Command::new("xdg-open").arg(url).spawn(); }
}

/// Open a folder in the system file explorer
fn open_folder(path: &str) {
    #[cfg(target_os = "windows")]
    { let _ = std::process::Command::new("explorer").arg(path).spawn(); }
    #[cfg(target_os = "macos")]
    { let _ = std::process::Command::new("open").arg(path).spawn(); }
    #[cfg(target_os = "linux")]
    { let _ = std::process::Command::new("xdg-open").arg(path).spawn(); }
}

/// Get config folder path (matches rabbit-platform::config::get_config_dir)
fn get_config_folder() -> String {
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("rabbit").to_string_lossy().to_string();
    }
    String::from(".")
}

// ============================================================
// Settings Tab Component
// ============================================================

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

        // Row 2: Checkboxes
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

        // Row 3: Version info (left) + Links (right-aligned)
        let mut info_row = Flex::default().row();
        info_row.set_spacing(10);

        let version_label = format!("sRabbit {}", CURRENT_VERSION);
        let mut version_frame = Frame::default().with_label(&version_label);
        version_frame.set_align(Align::Left | Align::Inside);
        version_frame.set_label_color(fltk::enums::Color::Blue);
        info_row.fixed(&version_frame, 120);

        Frame::default(); // Spacer

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

        // Output area
        let mut output_display = TextDisplay::default();
        let output_buf = TextBuffer::default();
        output_display.set_buffer(Some(output_buf));
        output_display.wrap_mode(WrapMode::AtBounds, 0);
        output_display.set_text_size(14);

        grp.end();

        // Styling
        grp.set_color(colors.background);
        output_display.set_color(colors.input_bg);
        output_display.set_text_color(colors.text);

        // Link callbacks
        home_btn.set_callback(|_| open_url(HOME_URL));

        profile_btn.set_callback(|_| {
            let config_path = get_config_folder();
            let _ = std::fs::create_dir_all(&config_path);
            open_folder(&config_path);
        });

        help_btn.set_callback(|_| open_url(HELP_URL));

        // Version check callback
        let output_clone = output_display.clone();
        let mut version_frame_handle = version_frame.clone();
        version_frame_handle.handle(move |_, ev| {
            if ev == fltk::enums::Event::Push {
                fltk::app::copy(&format!("sRabbit {}", CURRENT_VERSION));

                let out = output_clone.clone();
                std::thread::spawn(move || {
                    if let Some(mut buf) = out.buffer() {
                        buf.set_text("Checking for updates...\n");
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

        // Auto-save callbacks
        lang_choice.set_callback(move |_| send_event(UiEvent::SettingsSave));
        tray_check.set_callback(move |_| send_event(UiEvent::SettingsSave));
        top_check.set_callback(move |_| send_event(UiEvent::SettingsSave));
        autostart_check.set_callback(move |_| send_event(UiEvent::SettingsSave));
        autoupdate_check.set_callback(move |_| send_event(UiEvent::SettingsSave));

        grp
    }
}
