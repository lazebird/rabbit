//! TFTP Client Tab UI Component
//!
//! Layout based on old version screenshot:
//! - Row 1: IP | Opt. configuration
//! - Row 2: Local file path | Put button
//! - Row 3: Remote filename | Get button
//! - Transfer log

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

/// TFTP Client Tab Component
pub struct TftpcTab;

impl TabComponent for TftpcTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "TFTPC").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Row 1: IP and Opt
        let mut row1 = Flex::default().row();
        row1.set_spacing(spacing.padding);

        let _ip_label = Frame::default().with_label("IP");

        let mut server_input = Input::default();
        server_input.set_value(&defaults::tftpc_server());

        let _opt_label = Frame::default().with_label("Opt.");

        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::tftpc_options());

        Frame::default(); // Spacer
        row1.end();
        grp.fixed(&row1, spacing.row_height);

        // Row 2: Local file and Put button
        let mut row2 = Flex::default().row();
        row2.set_spacing(spacing.padding);

        let _local_label = Frame::default().with_label("Local:");

        let mut local_input = Input::default();
        local_input.set_value("");

        let mut browse_btn = Button::default().with_label("...");
        browse_btn.set_color(colors.accent);

        let mut put_btn = Button::default().with_label("Put");
        put_btn.set_color(colors.accent);
        put_btn.set_label_color(fltk::enums::Color::White);

        Frame::default(); // Spacer
        row2.end();
        grp.fixed(&row2, spacing.row_height);

        // Row 3: Remote filename and Get button
        let mut row3 = Flex::default().row();
        row3.set_spacing(spacing.padding);

        let _remote_label = Frame::default().with_label("Remote:");

        let mut remote_input = Input::default();
        remote_input.set_value("");

        let mut get_btn = Button::default().with_label("Get");
        get_btn.set_color(colors.accent);
        get_btn.set_label_color(fltk::enums::Color::White);

        Frame::default(); // Spacer
        row3.end();
        grp.fixed(&row3, spacing.row_height);

        // Transfer log
        let mut log_display = TextDisplay::default();
        let log_buf = TextBuffer::default();
        log_display.set_buffer(Some(log_buf));

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        log_display.set_color(colors.input_bg);
        log_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_log(&mut log_display);

        // Clone inputs for callbacks
        let server_input_clone = server_input.clone();
        let opt_input_clone = opt_input.clone();
        let local_input_clone = local_input.clone();
        let remote_input_clone = remote_input.clone();
        let mut local_input_browse = local_input.clone();
        let mut remote_input_browse = remote_input.clone();
        let mut log_display_clone = log_display.clone();

        // Browse button callback
        browse_btn.set_callback(move |_| {
            use fltk::dialog::NativeFileChooser;
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseFile);
            dialog.set_title("Select File to Upload");
            dialog.show();
            if let Some(path) = dialog.filename().to_str() {
                local_input_browse.set_value(path);
                // Set remote filename to local filename
                if let Some(filename) = std::path::Path::new(path).file_name() {
                    if let Some(name) = filename.to_str() {
                        remote_input_browse.set_value(name);
                    }
                }
            }
        });

        // Put button callback
        put_btn.set_callback(move |_| {
            let server = server_input_clone.value();
            let local = local_input_clone.value();
            let remote = remote_input_clone.value();
            let options = opt_input_clone.value();

            if server.is_empty() || local.is_empty() || remote.is_empty() {
                fltk::dialog::alert_default("Please fill in all fields!");
                return;
            }

            // Update log
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    s.tftpc_log.push_str(&format!("Uploading {} to {} as {}\n", local, server, remote));
                }
            }
            Self::refresh_log(&mut log_display_clone);

            send_event(UiEvent::TftpClientPut { server, local, remote, options });
        });

        // Get button callback
        get_btn.set_callback(move |_| {
            let server = server_input.value();
            let local = local_input.value();
            let remote = remote_input.value();
            let options = opt_input.value();

            if server.is_empty() || local.is_empty() || remote.is_empty() {
                fltk::dialog::alert_default("Please fill in all fields!");
                return;
            }

            // Update log
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    s.tftpc_log.push_str(&format!("Downloading {} from {} to {}\n", remote, server, local));
                }
            }

            send_event(UiEvent::TftpClientGet { server, local, remote, options });
        });

        // Set up timer to refresh log
        let mut log_display_timer = log_display.clone();
        fltk::app::add_idle3(move |_| {
            Self::refresh_log(&mut log_display_timer);
        });

        grp
    }
}

impl TftpcTab {
    fn refresh_log(display: &mut TextDisplay) {
        if let Some(state) = UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(buf) = display.buffer() {
                    let current_text = buf.text();
                    if current_text != s.tftpc_log {
                        drop(buf);
                        if let Some(mut new_buf) = display.buffer() {
                            new_buf.set_text(&s.tftpc_log);
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
