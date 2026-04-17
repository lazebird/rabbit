//! HTTP Server Tab UI Component
//!
//! Layout based on old version screenshot:
//! - Top row: Port | Opt. | Shell checkbox | Start/Stop button
//! - Directory list with paths
//! - Access log at bottom

use fltk::{
    button::{Button, CheckButton},
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    prelude::*,
    text::{TextBuffer, TextDisplay},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, Spacing, defaults};

/// HTTP Tab Component
pub struct HttpTab;

impl TabComponent for HttpTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "HTTPD").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Top control row
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(spacing.padding);

        // Port
        let _port_label = Frame::default().with_label("Port");

        let mut port_input = IntInput::default();
        port_input.set_value(&defaults::http_port().to_string());

        // Opt.
        let _opt_label = Frame::default().with_label("Opt.");

        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::http_options());

        // Shell checkbox
        let shell_check = CheckButton::default().with_label("shell");

        // Spacer
        Frame::default();

        // Start/Stop button
        let mut toggle_btn = Button::default().with_label("Start");
        toggle_btn.set_color(colors.accent);
        toggle_btn.set_label_color(fltk::enums::Color::White);

        ctrl_row.end();
        grp.fixed(&ctrl_row, spacing.row_height);

        // Directory management row
        let mut dir_row = Flex::default().row();
        dir_row.set_spacing(spacing.padding);

        // Add/Remove directory buttons
        let mut add_btn = Button::default().with_label("+");
        add_btn.set_color(colors.accent);
        add_btn.set_label_color(fltk::enums::Color::White);

        let mut remove_btn = Button::default().with_label("-");
        ;
        remove_btn.set_color(fltk::enums::Color::from_hex(0xE57373));

        Frame::default(); // Spacer
        dir_row.end();
        grp.fixed(&dir_row, spacing.row_height);

        // Directory list area
        let mut dir_display = TextDisplay::default();
        let dir_buf = TextBuffer::default();
        dir_display.set_buffer(Some(dir_buf));
        grp.fixed(&dir_display, 80);

        // Separator
        let _sep = Frame::default().with_label("");
        grp.fixed(&_sep, 2);

        // Access log
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

        // Clone inputs for callback
        let port_input_clone = port_input.clone();
        let opt_input_clone = opt_input.clone();
        let shell_check_clone = shell_check.clone();
        let mut toggle_btn_clone = toggle_btn.clone();
        let colors_clone = colors.clone();
        let mut log_display_clone = log_display.clone();

        // Add button callbacks
        add_btn.set_callback(move |_| {
            // Use native file dialog to select directory
            use fltk::dialog::NativeFileChooser;
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
            dialog.set_title("Select Directory to Serve");
            dialog.show();
            if let Some(path) = dialog.filename().to_str() {
                if !path.is_empty() {
                    // Add directory to list
                    if let Some(state) = UiState::global() {
                        if let Ok(mut s) = state.lock() {
                            s.http_log.push_str(&format!("Added directory: {}\n", path));
                        }
                    }
                }
            }
        });

        remove_btn.set_callback(move |_| {
            // Show a simple message - in real implementation, would need a list selector
            fltk::dialog::message_default("Select a directory from the list to remove.\n\nThis would remove the selected directory from the served list.");
        });

        toggle_btn.set_callback(move |_| {
            let label = toggle_btn_clone.label();
            let port = port_input_clone.value().parse::<u16>().unwrap_or(8000);
            let options = opt_input_clone.value();
            let shell = shell_check_clone.is_checked();

            if label == "Start" {
                // Update state
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.http_log = format!("HTTP Server Log:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nStarting HTTP server on port {}...\n", port);
                    }
                }
                Self::refresh_log(&mut log_display_clone);

                send_event(UiEvent::HttpToggle { port, options, shell });
                toggle_btn_clone.set_label("Stop");
                toggle_btn_clone.set_color(fltk::enums::Color::from_hex(0xE57373));
            } else {
                send_event(UiEvent::HttpToggle { port, options, shell });
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

impl HttpTab {
    fn refresh_log(display: &mut TextDisplay) {
        if let Some(state) = UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(buf) = display.buffer() {
                    let current_text = buf.text();
                    if current_text != s.http_log {
                        drop(buf);
                        if let Some(mut new_buf) = display.buffer() {
                            new_buf.set_text(&s.http_log);
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
