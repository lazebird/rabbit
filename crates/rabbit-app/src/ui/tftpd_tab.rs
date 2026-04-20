//! TFTP Server Tab UI Component
//!
//! Layout matching old version:
//! - Single row: [+] [-] Opt. [long input] [Start/Stop button]
//! - Directory list (fills most space)
//! - Transfer log at bottom

use fltk::{
    browser::Browser,
    button::Button,
    frame::Frame,
    group::Flex,
    input::Input,
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, defaults};

/// TFTP Server Tab Component
pub struct TftpdTab;

impl TabComponent for TftpdTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "TFTPD").column();
        grp.set_margin(8);
        grp.set_spacing(5);

        // Control row - matching old version layout
        // [+] [-] Opt. [long input] [Start button]
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // Add button (small square)
        let mut add_btn = Button::default().with_label("+");
        add_btn.set_color(colors.accent);
        add_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&add_btn, 28);

        // Remove button (small square)
        let mut remove_btn = Button::default().with_label("-");
        remove_btn.set_color(fltk::enums::Color::from_hex(0xE57373));
        remove_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&remove_btn, 28);

        // Opt. label (fixed width)
        let _opt_label = Frame::default().with_label("Opt.");
        ctrl_row.fixed(&_opt_label, 30);

        // Options input (takes remaining space)
        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::tftpd_options());

        // Start/Stop button (fixed width, right aligned)
        let mut toggle_btn = Button::default().with_label("Start");
        toggle_btn.set_color(colors.accent);
        toggle_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&toggle_btn, 70);

        ctrl_row.end();
        grp.fixed(&ctrl_row, 28);

        // Directory list area - using Browser for proper selection
        let mut dir_browser = Browser::default();
        dir_browser.set_color(colors.input_bg);
        dir_browser.set_text_size(14);
        dir_browser.set_type(fltk::browser::BrowserType::Hold);
        dir_browser.set_selection_color(fltk::enums::Color::from_hex(0x4A90D9));

        // Transfer log at bottom
        let mut log_display = TextDisplay::default();
        let log_buf = TextBuffer::default();
        log_display.set_buffer(Some(log_buf));
        log_display.wrap_mode(WrapMode::AtBounds, 0);

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        log_display.set_color(colors.input_bg);
        log_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_dirs(&mut dir_browser);
        Self::refresh_log(&mut log_display);

        // Clone inputs for callbacks
        let opt_input_clone = opt_input.clone();
        let mut toggle_btn_clone = toggle_btn.clone();
        let colors_clone = colors.clone();
        let mut log_display_clone = log_display.clone();
        let mut dir_browser_add = dir_browser.clone();
        let mut dir_browser_remove = dir_browser.clone();

        // Add button callbacks
        add_btn.set_callback(move |_| {
            use fltk::dialog::NativeFileChooser;
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
            dialog.set_title("Select TFTP Directory");
            dialog.show();
            if let Some(path) = dialog.filename().to_str() {
                if !path.is_empty() {
                    crate::ui_state::add_tftpd_dir(path);
                    crate::ui_state::append_tftpd_log(&format!("Added directory: {}\r\n", path));
                    Self::refresh_dirs(&mut dir_browser_add);
                }
            }
        });

        // Remove button - uses Browser selection
        remove_btn.set_callback(move |_| {
            let selected_idx = dir_browser_remove.value();
            if selected_idx <= 0 {
                // No selection or invalid index
                return;
            }

            // Get the selected directory path (index is 1-based)
            if let Some(text) = dir_browser_remove.text(selected_idx) {
                let remove_path = text.trim().to_string();
                if !remove_path.is_empty() {
                    crate::ui_state::remove_tftpd_dir(&remove_path);
                    crate::ui_state::append_tftpd_log(&format!("Removed directory: {}\r\n", remove_path));
                    Self::refresh_dirs(&mut dir_browser_remove);
                }
            }
        });

        toggle_btn.set_callback(move |_| {
            let label = toggle_btn_clone.label();
            let options = opt_input_clone.value();

            if label == "Start" {
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.tftpd_log = format!("Starting TFTP server...\nOptions: {}\n", options);
                    }
                }
                Self::refresh_log(&mut log_display_clone);

                send_event(UiEvent::TftpServerToggle { options });
                toggle_btn_clone.set_label("Stop");
                toggle_btn_clone.set_color(fltk::enums::Color::from_hex(0xE57373));
            } else {
                send_event(UiEvent::TftpServerToggle { options });
                toggle_btn_clone.set_label("Start");
                toggle_btn_clone.set_color(colors_clone.accent);
            }
        });

        // Register displays with centralized refresh manager
        super::ui_refresh::register_display("tftpd_log", log_display.clone());
        super::ui_refresh::register_browser("tftpd_dirs", dir_browser.clone());

        grp
    }
}

impl TftpdTab {
    /// Refresh directory list in the Browser widget.
    fn refresh_dirs(browser: &mut Browser) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                browser.clear();
                if s.tftpd_dirs.is_empty() {
                    browser.add("(no directories added)");
                } else {
                    for dir in &s.tftpd_dirs {
                        browser.add(dir);
                    }
                }
            }
        }
    }

    fn refresh_log(display: &mut TextDisplay) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.tftpd_log);
                    let lines = buf.count_lines(0, buf.length());
                    display.scroll(lines, 0);
                }
            }
        }
    }
}
