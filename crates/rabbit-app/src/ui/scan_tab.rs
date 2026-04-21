//! Scan Tab UI Component
//!
//! Layout matching old version:
//! - Single row: IP [input] - [end] Opt. [long input] [Start button]
//! - Results area fills remaining space

use fltk::{
    button::Button,
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, defaults};

/// Scan Tab Component
pub struct ScanTab;

impl TabComponent for ScanTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "Scan").column();
        grp.set_margin(5);
        grp.set_spacing(4);

        // Control row - matching old version layout
        // IP [input] - [end] Opt. [long input] [Start button]
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // IP label
        let _ip_label = Frame::default().with_label("IP");
        ctrl_row.fixed(&_ip_label, 20);

        // Start IP input
        let mut start_ip_input = Input::default();
        start_ip_input.set_value(&defaults::scan_start_ip());
        ctrl_row.fixed(&start_ip_input, 110);

        // Dash separator
        let _dash_label = Frame::default().with_label("-");
        ctrl_row.fixed(&_dash_label, 10);

        // End IP input (just the last octet)
        let mut end_input = IntInput::default();
        end_input.set_value(&defaults::scan_end_ip());
        ctrl_row.fixed(&end_input, 40);

        // Opt. label
        let _opt_label = Frame::default().with_label("Opt.");
        ctrl_row.fixed(&_opt_label, 30);

        // Options input (takes remaining space)
        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::scan_options());

        // Start/Stop button (fixed width, right aligned)
        let mut start_btn = Button::default().with_label("Start");
        start_btn.set_color(colors.accent);
        start_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&start_btn, 70);

        ctrl_row.end();
        grp.fixed(&ctrl_row, 28);

        // Results area fills remaining space
        let mut results_display = TextDisplay::default();
        let results_buf = TextBuffer::default();
        results_display.set_buffer(Some(results_buf));
        results_display.wrap_mode(WrapMode::AtBounds, 0);

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        results_display.set_color(colors.input_bg);
        results_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_display(&mut results_display);

        // Clone inputs for callback
        let start_ip_input_clone = start_ip_input.clone();
        let end_input_clone = end_input.clone();
        let opt_input_clone = opt_input.clone();
        let start_btn_for_cb = start_btn.clone();

        // Add button callback - just send event, refresh loop updates button
        start_btn.set_callback(move |_| {
            let label = start_btn_for_cb.label();
            if label == "Start" {
                let start_ip = start_ip_input_clone.value();
                let end_suffix = end_input_clone.value();
                let options = opt_input_clone.value();

                if start_ip.is_empty() {
                    fltk::dialog::alert_default("Please enter a start IP address!");
                    return;
                }

                // Build full end IP from start IP prefix + end suffix
                let end_ip = if end_suffix.is_empty() {
                    start_ip.clone()
                } else {
                    // Take first 3 octets of start_ip
                    let parts: Vec<&str> = start_ip.splitn(4, '.').collect();
                    if parts.len() == 4 {
                        format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], end_suffix)
                    } else {
                        fltk::dialog::alert_default("Invalid start IP address format! (expected x.x.x.x)");
                        return;
                    }
                };

                // Clear previous output
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.scan_output = format!("Scanning range {} to {}...\n", start_ip, end_ip);
                        s.updated.insert("scan_output".to_string(), true);
                    }
                }

                send_event(UiEvent::ScanStart { start_ip, end_ip, options });
            } else {
                send_event(UiEvent::ScanStop);
            }
        });

        // Register display and button with centralized refresh manager
        super::ui_refresh::register_display("scan_output", results_display.clone());
        super::ui_refresh::register_scan_button(start_btn, colors.accent);

        grp
    }
}

impl ScanTab {
    fn refresh_display(display: &mut TextDisplay) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.scan_output);
                    let lines = buf.count_lines(0, buf.length());
                    display.scroll(lines, 0);
                }
            }
        }
    }
}
