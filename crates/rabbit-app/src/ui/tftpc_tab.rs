//! TFTP Client Tab UI Component
//!
//! Layout matching old version:
//! - Row 1: IP [input] Opt. [long input]
//! - Row 2: Local [long input] [Put button]
//! - Row 3: Remote [long input] [Get button]
//! - Transfer log fills remaining space

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

/// TFTP Client Tab Component
pub struct TftpcTab;

impl TabComponent for TftpcTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "TFTPC").column();
        grp.set_margin(0);  // Remove margin to match old version
        grp.set_spacing(4);

        // Row 1: IP and Opt
        let mut row1 = Flex::default().row();
        row1.set_spacing(5);

        let _ip_label = Frame::default().with_label("IP");
        row1.fixed(&_ip_label, 20);

        let mut server_input = Input::default();
        server_input.set_value(&defaults::tftpc_server());
        row1.fixed(&server_input, 120);

        let _opt_label = Frame::default().with_label("Opt.");
        row1.fixed(&_opt_label, 30);

        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::tftpc_options());

        row1.end();
        grp.fixed(&row1, 28);

        // Row 2: Local file and Put button
        let mut row2 = Flex::default().row();
        row2.set_spacing(5);

        let _local_label = Frame::default().with_label("Local:");
        row2.fixed(&_local_label, 40);

        let mut local_input = Input::default();
        local_input.set_value("");

        let mut put_btn = Button::default().with_label("Put");
        put_btn.set_color(colors.accent);
        put_btn.set_label_color(fltk::enums::Color::White);
        row2.fixed(&put_btn, 70);

        row2.end();
        grp.fixed(&row2, 28);

        // Row 3: Remote filename and Get button
        let mut row3 = Flex::default().row();
        row3.set_spacing(5);

        let _remote_label = Frame::default().with_label("Remote:");
        row3.fixed(&_remote_label, 50);

        let mut remote_input = Input::default();
        remote_input.set_value("");

        let mut get_btn = Button::default().with_label("Get");
        get_btn.set_color(colors.accent);
        get_btn.set_label_color(fltk::enums::Color::White);
        row3.fixed(&get_btn, 70);

        row3.end();
        grp.fixed(&row3, 28);

        // Transfer log fills remaining space
        let mut log_display = TextDisplay::default();
        let log_buf = TextBuffer::default();
        log_display.set_buffer(Some(log_buf));
        log_display.wrap_mode(WrapMode::AtBounds, 0);
        log_display.set_frame(fltk::enums::FrameType::FlatBox);  // Remove border

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
        let _local_input_browse = local_input.clone();
        let _remote_input_browse = remote_input.clone();
        let mut log_display_clone = log_display.clone();

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

        // Register display with centralized refresh manager
        super::ui_refresh::register_display("tftpc_log", log_display.clone());
        
        // Register Put button for Enter key support (primary action)
        super::ui_refresh::register_tftpc_button(put_btn.clone(), colors.accent);

        grp
    }
}

impl TftpcTab {
    fn refresh_log(display: &mut TextDisplay) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.tftpc_log);
                    let lines = buf.count_lines(0, buf.length());
                    display.scroll(lines, 0);
                }
            }
        }
    }
}
