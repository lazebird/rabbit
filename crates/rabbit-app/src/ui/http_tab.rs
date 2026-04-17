//! HTTP Server Tab UI Component
//!
//! Layout matching old version:
//! - Single row: Port [input] Opt. [long input] [shell checkbox] [Start/Stop button]
//! - Directory list (fills most space)
//! - Access log at bottom

use fltk::{
    button::{Button, CheckButton},
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, defaults};

/// HTTP Tab Component
pub struct HttpTab;

impl TabComponent for HttpTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "HTTPD").column();
        grp.set_margin(8);
        grp.set_spacing(5);

        // Control row - matching old version layout
        // Port [input] Opt. [long input] [shell checkbox] [Start button]
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // Port label (fixed width)
        let _port_label = Frame::default().with_label("Port");
        ctrl_row.fixed(&_port_label, 30);

        // Port input (small width)
        let mut port_input = IntInput::default();
        port_input.set_value(&defaults::http_port().to_string());
        ctrl_row.fixed(&port_input, 50);

        // Opt. label (fixed width)
        let _opt_label = Frame::default().with_label("Opt.");
        ctrl_row.fixed(&_opt_label, 30);

        // Options input (takes remaining space)
        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::http_options());

        // Shell checkbox (fixed width)
        let shell_check = CheckButton::default().with_label("shell");
        ctrl_row.fixed(&shell_check, 60);

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

        // Access log at bottom (smaller area)
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
        Self::refresh_log(&mut log_display);

        // Clone inputs for callback
        let port_input_clone = port_input.clone();
        let opt_input_clone = opt_input.clone();
        let shell_check_clone = shell_check.clone();
        let mut toggle_btn_clone = toggle_btn.clone();
        let mut log_display_clone = log_display.clone();

        // Toggle button callback
        toggle_btn.set_callback(move |_| {
            let label = toggle_btn_clone.label();
            let port = port_input_clone.value().parse::<u16>().unwrap_or(8000);
            let options = opt_input_clone.value();
            let shell = shell_check_clone.is_checked();

            if label == "Start" {
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.http_log = format!("Starting HTTP server on port {}...\n", port);
                        s.updated.insert("http_log".to_string(), true);
                    }
                }
                Self::refresh_log(&mut log_display_clone);
                send_event(UiEvent::HttpToggle { port, options, shell });
            } else {
                send_event(UiEvent::HttpToggle { port, options, shell });
            }
        });

        // Register display with centralized refresh manager
        super::ui_refresh::register_display("http_log", log_display.clone());
        super::ui_refresh::register_http_button(toggle_btn.clone(), colors.accent);

        grp
    }
}

impl HttpTab {
    fn refresh_log(display: &mut TextDisplay) {
        if let Some(state) = UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.http_log);
                    let lines = buf.count_lines(0, buf.length());
                    display.scroll(lines, 0);
                }
            }
        }
    }
}
