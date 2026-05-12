//! Settings Tab UI Component
//!
//! Layout:
//! - All content left-aligned
//! - Auto-save on change (no Save button)

use fltk::{
    button::{Button, CheckButton},
    enums::Align,
    frame::Frame,
    group::Flex,
    menu::Choice,
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
};

use super::{Colors, Spacing, TabComponent};
use crate::ui_events::{send_event, UiEvent};
use crate::upgrade::{self, VersionsManifest};

// ============================================================
// Version Check Helper
// ============================================================

/// Fetch remote version content using ureq
fn fetch_version_content(url: &str) -> Result<String, String> {
    ureq::get(url)
        .timeout(std::time::Duration::from_secs(15))
        .call()
        .map_err(|e| format!("HTTP error: {}", e))?
        .into_string()
        .map_err(|e| format!("Read error: {}", e))
}

/// Check for version updates
pub fn check_version_update() -> upgrade::UpdateStatus {
    match fetch_version_content(VERSION_CHECK_URL) {
        Ok(content) => {
            let trimmed = content.trim();
            if !trimmed.starts_with('{') {
                return upgrade::UpdateStatus::CheckError("Server returned non-JSON response".to_string());
            }

            match serde_json::from_str::<VersionsManifest>(trimmed) {
                Ok(remote) => {
                    if remote.is_newer_than(CURRENT_VERSION) {
                        if let Some(platform_info) = remote.clone().for_current_platform() {
                            upgrade::UpdateStatus::UpdateAvailable(remote, platform_info.clone())
                        } else {
                            upgrade::UpdateStatus::CheckError("No build for current platform".to_string())
                        }
                    } else {
                        upgrade::UpdateStatus::UpToDate
                    }
                }
                Err(e) => upgrade::UpdateStatus::CheckError(format!("Failed to parse: {}", e)),
            }
        }
        Err(e) => upgrade::UpdateStatus::CheckError(format!("Network error: {}", e)),
    }
}

// ============================================================
// Constants - URLs and Version
// ============================================================

/// Current application version from Cargo.toml
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Remote versions.json URL for update checking
const VERSION_CHECK_URL: &str = "https://raw.githubusercontent.com/lazebird/rabbit/rewrite/release/versions.json";

/// Home page URL for downloads
const HOME_URL: &str = "https://github.com/lazebird/rabbit/tree/rewrite/release";

/// Help/manual URL
const HELP_URL: &str = "https://github.com/lazebird/rabbit/blob/rewrite/doc/manual.md";

// ============================================================
// Platform Helpers
// ============================================================

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
    fn build(x: i32, y: i32, w: i32, h: i32, config: &schema::AppConfig) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "Settings").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Row 1: Language
        let mut lang_row = Flex::default().row();
        lang_row.set_spacing(10);

        let mut lang_label = Frame::default().with_label("Language");
        lang_label.set_align(Align::Left | Align::Inside);
        lang_row.fixed(&lang_label, 70);

        // Use the passed config reference
        let modules = &config.modules;

        let lang_str = modules.get_string("global", "language").unwrap_or_else(|| "System".to_string());
        let lang_idx = match lang_str.as_str() {
            "English" => 0,
            "中文" => 1,
            _ => 2,
        };
        let systray = modules.get_bool("global", "systray").unwrap_or(true);
        let top = modules.get_bool("global", "top").unwrap_or(false);
        let autostart = modules.get_bool("global", "autostart").unwrap_or(false);
        let autoupdate = modules.get_bool("global", "autoupdate").unwrap_or(true);

        let mut lang_choice = Choice::default();
        lang_choice.add_choice("English");
        lang_choice.add_choice("中文");
        lang_choice.add_choice("System");
        lang_choice.set_value(lang_idx);
        lang_row.fixed(&lang_choice, 100);

        Frame::default(); // Spacer
        lang_row.end();
        grp.fixed(&lang_row, spacing.row_height);

        // Row 2: Checkboxes
        let mut check_row = Flex::default().row();
        check_row.set_spacing(15);

        let mut tray_check = CheckButton::default().with_label("Tray");
        tray_check.set_checked(systray);
        check_row.fixed(&tray_check, 55);

        let mut top_check = CheckButton::default().with_label("Top");
        top_check.set_checked(top);
        check_row.fixed(&top_check, 50);

        let mut autostart_check = CheckButton::default().with_label("AutoStart");
        autostart_check.set_checked(autostart);
        check_row.fixed(&autostart_check, 80);

        let mut autoupdate_check = CheckButton::default().with_label("AutoUpdate");
        autoupdate_check.set_checked(autoupdate);
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
        output_display.set_frame(fltk::enums::FrameType::FlatBox); // Remove border

        // Initialize output from ui_state
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                if !s.settings_output.is_empty() {
                    if let Some(mut buf) = output_display.buffer() {
                        buf.set_text(&s.settings_output);
                    }
                }
            }
        }

        grp.end();

        // Styling
        grp.set_color(colors.background);
        output_display.set_color(colors.input_bg);
        output_display.set_text_color(colors.text);

        // Link callbacks
        home_btn.set_callback(|_| { let _ = adapter::dialog::open_url(HOME_URL); });

        profile_btn.set_callback(|_| {
            let config_path = get_config_folder();
            let _ = std::fs::create_dir_all(&config_path);
            let _ = adapter::dialog::open_file_manager(&config_path);
        });

        help_btn.set_callback(|_| { let _ = adapter::dialog::open_url(HELP_URL); });

        // Version check callback
        let _output_clone = output_display.clone();
        let mut version_frame_handle = version_frame.clone();
        version_frame_handle.handle(move |_, ev| {
            if ev == fltk::enums::Event::Push {
                fltk::app::copy(&format!("sRabbit {}", CURRENT_VERSION));

                crate::ui_state::write_to("settings_output", &crate::ui_state::raw_log(""));
                crate::ui_state::write_to("settings_output", &crate::ui_state::fmt_log("Checking for updates..."));

                std::thread::spawn(|| {
                    crate::app::handle_version_check_result(None);
                });
                true
            } else {
                false
            }
        });

        // Auto-save callbacks - update config then save
        let lang_choice_clone = lang_choice.clone();
        let tray_check_clone = tray_check.clone();
        let top_check_clone = top_check.clone();
        let autostart_check_clone = autostart_check.clone();
        let autoupdate_check_clone = autoupdate_check.clone();

        lang_choice.set_callback(move |_| {
            let idx = lang_choice_clone.value();
            let lang = match idx {
                0 => "English",
                1 => "中文",
                _ => "System",
            };
            crate::ui_state::set_language(lang);
            send_event(UiEvent::SettingsSave);
        });
        tray_check.set_callback(move |_| {
            crate::ui_state::set_systray(tray_check_clone.is_checked());
            send_event(UiEvent::SettingsSave);
        });
        top_check.set_callback(move |_| {
            crate::ui_state::set_top(top_check_clone.is_checked());
            send_event(UiEvent::SettingsSave);
        });
        autostart_check.set_callback(move |_| {
            crate::ui_state::set_autostart(autostart_check_clone.is_checked());
            send_event(UiEvent::SettingsSave);
        });
        autoupdate_check.set_callback(move |_| {
            crate::ui_state::set_autoupdate(autoupdate_check_clone.is_checked());
            send_event(UiEvent::SettingsSave);
        });

        // Register settings output for centralized refresh
        crate::ui::ui_refresh::register_display("settings_output", output_display);

        grp
    }
}
