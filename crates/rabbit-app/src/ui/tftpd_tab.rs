//! TFTP Server Tab UI Component
//!
//! Layout based on old version screenshot:
//! - +/- buttons for directory management
//! - Opt. configuration field
//! - Directory list
//! - Transfer log with format: I: [###:IP]:PORT filename SIZE/XX.Xs @X,XXX.X pps/X,XXX,XXX.X Bps

use fltk::{
    button::Button,
    frame::Frame,
    group::Flex,
    input::Input,
    prelude::*,
    text::{TextBuffer, TextDisplay},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, Spacing, defaults};

/// TFTP Server Tab Component
pub struct TftpdTab;

impl TabComponent for TftpdTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "TFTPD").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Top control row
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(spacing.padding);

        // Add/Remove buttons
        let mut add_btn = Button::default().with_label("+");
        add_btn.set_color(colors.accent);
        add_btn.set_label_color(fltk::enums::Color::White);

        let mut remove_btn = Button::default().with_label("-");
        remove_btn.set_color(fltk::enums::Color::from_hex(0xE57373));

        // Opt.
        let _opt_label = Frame::default().with_label("Opt.");

        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::tftpd_options());

        // Spacer
        Frame::default();

        // Start/Stop button
        let mut toggle_btn = Button::default().with_label("Start");
        toggle_btn.set_color(colors.accent);
        toggle_btn.set_label_color(fltk::enums::Color::White);

        ctrl_row.end();
        grp.fixed(&ctrl_row, spacing.row_height);

        // Directory list area
        let mut dir_display = TextDisplay::default();
        let dir_buf = TextBuffer::default();
        dir_display.set_buffer(Some(dir_buf));
        grp.fixed(&dir_display, 80);

        // Transfer log
        let mut log_display = TextDisplay::default();
        let log_buf = TextBuffer::default();
        log_display.set_buffer(Some(log_buf));

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        dir_display.set_color(colors.input_bg);
        dir_display.set_text_color(colors.text);
        log_display.set_color(colors.input_bg);
        log_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_log(&mut log_display);

        // Clone inputs for callbacks
        let opt_input_clone = opt_input.clone();
        let mut toggle_btn_clone = toggle_btn.clone();
        let colors_clone = colors.clone();
        let mut log_display_clone = log_display.clone();

        // Add button callbacks
        add_btn.set_callback(move |_| {
            use fltk::dialog::NativeFileChooser;
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
            dialog.set_title("Select TFTP Directory");
            dialog.show();
            if let Some(path) = dialog.filename().to_str() {
                if !path.is_empty() {
                    if let Some(state) = UiState::global() {
                        if let Ok(mut s) = state.lock() {
                            s.tftpd_log.push_str(&format!("Added directory: {}\n", path));
                        }
                    }
                }
            }
        });

        remove_btn.set_callback(move |_| {
            fltk::dialog::message_default("Select a directory from the list to remove.");
        });

        toggle_btn.set_callback(move |_| {
            let label = toggle_btn_clone.label();
            let options = opt_input_clone.value();

            if label == "Start" {
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.tftpd_log = format!("TFTP Transfer Log:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nStarting TFTP server...\nOptions: {}\n", options);
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

        // Set up timer to refresh log
        let mut log_display_timer = log_display.clone();
        fltk::app::add_idle3(move |_| {
            Self::refresh_log(&mut log_display_timer);
        });

        grp
    }
}

impl TftpdTab {
    fn refresh_log(display: &mut TextDisplay) {
        if let Some(state) = UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(buf) = display.buffer() {
                    let current_text = buf.text();
                    if current_text != s.tftpd_log {
                        drop(buf);
                        if let Some(mut new_buf) = display.buffer() {
                            new_buf.set_text(&s.tftpd_log);
                            let lines = new_buf.count_lines(0, new_buf.length());
                            display.set_buffer(Some(new_buf));
                            display.scroll(lines, 0);
                        }
                    }
                }
            }
        }
    }
}
