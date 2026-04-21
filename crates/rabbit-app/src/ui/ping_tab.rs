//! Ping Tab UI Component
//!
//! Layout matching old version:
//! - Single row: Addr. [input] Opt. [long input] [Start/Stop button]
//! - Stats line below
//! - Results log area fills remaining space

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

/// Ping Tab Component
pub struct PingTab;

impl TabComponent for PingTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "Ping").column();
        grp.set_margin(5);
        grp.set_spacing(4);

        // Control row - matching old version layout
        // Addr. [input] Opt. [long input] [Start button]
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // Addr. label (fixed width)
        let _addr_label = Frame::default().with_label("Addr.");
        ctrl_row.fixed(&_addr_label, 35);

        // Address input (medium width)
        let mut addr_input = Input::default();
        addr_input.set_value(&defaults::ping_target());
        ctrl_row.fixed(&addr_input, 120);

        // Opt. label (fixed width)
        let _opt_label = Frame::default().with_label("Opt.");
        ctrl_row.fixed(&_opt_label, 30);

        // Options input (takes remaining space)
        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::ping_options());

        // Start/Stop button (fixed width, right aligned)
        let mut start_btn = Button::default().with_label("Start");
        start_btn.set_color(colors.accent);
        start_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&start_btn, 70);

        // Register button for state sync
        super::ui_refresh::register_ping_button(start_btn.clone(), colors.accent);

        ctrl_row.end();
        grp.fixed(&ctrl_row, 28);

        // Stats line - single row, no scrollbar, copyable TextDisplay
        let mut stats_editor = TextDisplay::default();
        let stats_buf = TextBuffer::default();
        stats_editor.set_buffer(Some(stats_buf));
        stats_editor.wrap_mode(WrapMode::None, 0);
        stats_editor.set_scrollbar_size(0);
        stats_editor.set_frame(fltk::enums::FrameType::FlatBox);
        stats_editor.set_text_size(14);
        grp.fixed(&stats_editor, 26);

        // Results log area - fills remaining space, word wrap enabled
        let mut results_display = TextDisplay::default();
        let results_buf = TextBuffer::default();
        results_display.set_buffer(Some(results_buf));
        results_display.wrap_mode(WrapMode::AtBounds, 0);

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        results_display.set_color(colors.input_bg);
        results_display.set_text_color(colors.text);
        stats_editor.set_color(colors.background);
        stats_editor.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_display(&mut results_display, &mut stats_editor);

        // Clone for callbacks
        let addr_input_clone = addr_input.clone();
        let opt_input_clone = opt_input.clone();
        let mut start_btn_clone = start_btn.clone();
        let colors_clone = colors.clone();
        let mut results_display_clone = results_display.clone();
        let mut stats_editor_clone = stats_editor.clone();

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
                        s.ping_output.clear();
                        s.ping_stats.clear();
                    }
                }
                Self::refresh_display(&mut results_display_clone, &mut stats_editor_clone);

                send_event(UiEvent::PingStart { target: target.clone(), options: _options });

                start_btn_clone.set_label("Stop");
                start_btn_clone.set_color(fltk::enums::Color::from_hex(0xE57373));
            } else {
                send_event(UiEvent::PingStop);
                start_btn_clone.set_label("Start");
                start_btn_clone.set_color(colors_clone.accent);
            }
        });

        // Register displays with centralized refresh manager
        super::ui_refresh::register_display("ping_output", results_display.clone());
        super::ui_refresh::register_display("ping_stats", stats_editor.clone());

        grp
    }
}

impl PingTab {
    /// Called once during startup from global state (not from idle loop).
    fn refresh_display(display: &mut TextDisplay, stats: &mut TextDisplay) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.ping_output);
                }
                if let Some(mut buf) = stats.buffer() {
                    buf.set_text(&s.ping_stats);
                }
            }
        }
    }
}
