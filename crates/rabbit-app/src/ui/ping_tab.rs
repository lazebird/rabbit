//! Ping Tab UI Component
//!
//! Layout based on old version screenshot:
//! - Top row: Addr. input | Opt. input | Start/Stop button
//! - Stats line: Timestamp Tx X Rx X Loss X Min X Max X Avg X
//! - Main log area with ping results

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

/// Ping Tab Component
pub struct PingTab;

impl TabComponent for PingTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "Ping").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Top control row - matching old version layout
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(spacing.padding);

        // Addr. label and input
        let _addr_label = Frame::default().with_label("Addr.");

        let mut addr_input = Input::default();
        addr_input.set_value(&defaults::ping_target());

        // Opt. label and input
        let _opt_label = Frame::default().with_label("Opt.");

        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::ping_options());

        // Spacer
        Frame::default();

        // Start/Stop button (green in old version)
        let mut start_btn = Button::default().with_label("Start");
        start_btn.set_color(colors.accent);
        start_btn.set_label_color(fltk::enums::Color::White);

        ctrl_row.end();
        grp.fixed(&ctrl_row, spacing.row_height);

        // Stats line - single row display
        let mut stats_frame = Frame::default().with_label("Ready - Enter address and click Start");
        grp.fixed(&stats_frame, spacing.row_height);

        // Results log area
        let mut results_display = TextDisplay::default();
        let results_buf = TextBuffer::default();
        results_display.set_buffer(Some(results_buf));

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        results_display.set_color(colors.input_bg);
        results_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_display(&mut results_display, &mut stats_frame);

        // Clone for callbacks
        let addr_input_clone = addr_input.clone();
        let opt_input_clone = opt_input.clone();
        let mut start_btn_clone = start_btn.clone();
        let colors_clone = colors.clone();
        let mut results_display_clone = results_display.clone();
        let mut stats_frame_clone = stats_frame.clone();

        // Add button callback
        start_btn.set_callback(move |_| {
            let label = start_btn_clone.label();
            if label == "Start" {
                let target = addr_input_clone.value();
                let _options = opt_input_clone.value();

                if target.is_empty() {
                    fltk::dialog::alert_default("Please enter a target address!");
                    return;
                }

                // Clear previous output
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.ping_output = format!("Pinging {}...\n", target);
                        s.ping_stats = "Pinging...".to_string();
                    }
                }
                Self::refresh_display(&mut results_display_clone, &mut stats_frame_clone);

                send_event(UiEvent::PingStart { target: target.clone(), options: _options });
                
                start_btn_clone.set_label("Stop");
                start_btn_clone.set_color(fltk::enums::Color::from_hex(0xE57373));
            } else {
                send_event(UiEvent::PingStop);
                start_btn_clone.set_label("Start");
                start_btn_clone.set_color(colors_clone.accent);
            }
        });

        // Set up timer to refresh display
        let mut results_display_timer = results_display.clone();
        let mut stats_frame_timer = stats_frame.clone();
        fltk::app::add_idle3(move |_| {
            Self::refresh_display(&mut results_display_timer, &mut stats_frame_timer);
        });

        grp
    }
}

impl PingTab {
    fn refresh_display(display: &mut TextDisplay, stats: &mut Frame) {
        if let Some(state) = UiState::global() {
            if let Ok(mut s) = state.lock() {
                // Update results display
                if let Some(buf) = display.buffer() {
                    let current_text = buf.text();
                    if current_text != s.ping_output {
                        drop(buf);
                        if let Some(mut new_buf) = display.buffer() {
                            new_buf.set_text(&s.ping_output);
                            let lines = new_buf.count_lines(0, new_buf.length());
                            display.set_buffer(Some(new_buf));
                            display.scroll(lines, 0);
                        }
                    }
                }
                // Update stats - use clone to avoid borrow issues
                let stats_text = s.ping_stats.clone();
                s.clear_updated("ping_output");
                drop(s);
                stats.set_label(&stats_text);
            }
        }
    }
}
