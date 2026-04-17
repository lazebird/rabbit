//! Scan Tab UI Component
//!
//! Layout based on old version screenshot:
//! - Top row: IP input | Range (- 254) | Opt. | Start button
//! - Grid display of IP addresses (1-254) with green highlighting for online hosts

use fltk::{
    button::Button,
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    prelude::*,
    text::{TextBuffer, TextDisplay},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, Spacing, defaults};

/// Scan Tab Component
pub struct ScanTab;

impl TabComponent for ScanTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "Scan").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Top control row
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(spacing.padding);

        // IP input
        let _ip_label = Frame::default().with_label("IP");

        let mut start_ip_input = Input::default();
        start_ip_input.set_value(&defaults::scan_start_ip());

        // Range separator and input
        let _dash_label = Frame::default().with_label("-");

        let mut end_input = IntInput::default();
        end_input.set_value(&defaults::scan_end_ip());

        // Opt.
        let _opt_label = Frame::default().with_label("Opt.");

        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::scan_options());

        // Spacer
        Frame::default();

        // Start button
        let mut start_btn = Button::default().with_label("Start");
        start_btn.set_color(colors.accent);
        start_btn.set_label_color(fltk::enums::Color::White);

        ctrl_row.end();
        grp.fixed(&ctrl_row, spacing.row_height);

        // Progress/status line
        let progress_frame = Frame::default().with_label("Progress: 0% | Found: 0 | Scanning: 0/254");
        grp.fixed(&progress_frame, spacing.row_height);

        // Results area - text display for now (grid would require custom widget)
        let mut results_display = TextDisplay::default();
        let results_buf = TextBuffer::default();
        results_display.set_buffer(Some(results_buf));

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        results_display.set_color(colors.input_bg);
        results_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_display(&mut results_display, &progress_frame);

        // Clone inputs for callback
        let start_ip_input_clone = start_ip_input.clone();
        let end_input_clone = end_input.clone();
        let opt_input_clone = opt_input.clone();
        let mut start_btn_clone = start_btn.clone();
        let colors_clone = colors.clone();
        let mut results_display_clone = results_display.clone();
        let progress_frame_clone = progress_frame.clone();

        // Add button callback
        start_btn.set_callback(move |_| {
            let label = start_btn_clone.label();
            if label == "Start" {
                let start_ip = start_ip_input_clone.value();
                let end_ip = end_input_clone.value();
                let options = opt_input_clone.value();

                if start_ip.is_empty() {
                    fltk::dialog::alert_default("Please enter a start IP address!");
                    return;
                }

                // Clear previous output
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.scan_output = format!("Scanning range {} to {}...\n", start_ip, end_ip);
                    }
                }
                Self::refresh_display(&mut results_display_clone, &progress_frame_clone);

                send_event(UiEvent::ScanStart { start_ip, end_ip, options });
                start_btn_clone.set_label("Stop");
                start_btn_clone.set_color(fltk::enums::Color::from_hex(0xE57373));
            } else {
                send_event(UiEvent::ScanStop);
                start_btn_clone.set_label("Start");
                start_btn_clone.set_color(colors_clone.accent);
            }
        });

        // Set up timer to refresh display
        let mut results_display_timer = results_display.clone();
        let progress_frame_timer = progress_frame.clone();
        fltk::app::add_idle3(move |_| {
            Self::refresh_display(&mut results_display_timer, &progress_frame_timer);
        });

        grp
    }
}

impl ScanTab {
    fn refresh_display(display: &mut TextDisplay, progress: &Frame) {
        if let Some(state) = UiState::global() {
            if let Ok(s) = state.lock() {
                // Update results display
                if let Some(buf) = display.buffer() {
                    let current_text = buf.text();
                    if current_text != s.scan_output {
                        drop(buf);
                        if let Some(mut new_buf) = display.buffer() {
                            new_buf.set_text(&s.scan_output);
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
