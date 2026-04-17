//! TFTP Server Tab UI Component
//!
//! Layout matching old version:
//! - Single row: [+] [-] Opt. [long input] [Start/Stop button]
//! - Directory list (fills most space)
//! - Transfer log at bottom

use fltk::{
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

        // Directory list area - fills most space
        let mut dir_display = TextDisplay::default();
        let dir_buf = TextBuffer::default();
        dir_display.set_buffer(Some(dir_buf));
        dir_display.wrap_mode(WrapMode::AtBounds, 0);

        // Transfer log at bottom
        let mut log_display = TextDisplay::default();
        let log_buf = TextBuffer::default();
        log_display.set_buffer(Some(log_buf));
        log_display.wrap_mode(WrapMode::AtBounds, 0);

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        dir_display.set_color(colors.input_bg);
        dir_display.set_text_color(colors.text);
        log_display.set_color(colors.input_bg);
        log_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_dirs(&mut dir_display);
        Self::refresh_log(&mut log_display);

        // Clone inputs for callbacks
        let opt_input_clone = opt_input.clone();
        let mut toggle_btn_clone = toggle_btn.clone();
        let colors_clone = colors.clone();
        let mut log_display_clone = log_display.clone();
        let mut dir_display_clone = dir_display.clone();

        // Add button callbacks
        add_btn.set_callback(move |_| {
            use fltk::dialog::NativeFileChooser;
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
            dialog.set_title("Select TFTP Directory");
            dialog.show();
            if let Some(path) = dialog.filename().to_str() {
                if !path.is_empty() {
                    crate::ui_state::add_tftpd_dir(path);
                    crate::ui_state::append_tftpd_log(&format!("Added directory: {}", path));
                    Self::refresh_dirs(&mut dir_display_clone);
                }
            }
        });

        remove_btn.set_callback(move |_| {
            let remove_path = fltk::dialog::input_default("Enter directory path to remove:", "");
            if let Some(path) = remove_path {
                let path = path.trim();
                if !path.is_empty() {
                    crate::ui_state::remove_tftpd_dir(path);
                    crate::ui_state::append_tftpd_log(&format!("Removed directory: {}", path));
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
        super::ui_refresh::register_display("tftpd_dirs", dir_display.clone());
        super::ui_refresh::register_display("tftpd_log", log_display.clone());

        grp
    }
}

impl TftpdTab {
    /// Called once during button callback for immediate update.
    fn refresh_dirs(display: &mut TextDisplay) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                let dirs_text = if s.tftpd_dirs.is_empty() {
                    "(no directories added)\n".to_string()
                } else {
                    s.tftpd_dirs.join("\n") + "\n"
                };
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&dirs_text);
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
