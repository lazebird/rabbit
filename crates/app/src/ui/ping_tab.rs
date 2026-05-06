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

use super::{defaults, Colors, TabComponent};
use crate::ui_events::{send_event, UiEvent};
use crate::ui_state::UiState;
use schema::config::ConfigValue;
use adapter::config::{load_config, save_config};

/// Ping Tab Component
pub struct PingTab;

impl TabComponent for PingTab {
    fn build(x: i32, y: i32, w: i32, h: i32, config: &schema::AppConfig) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "Ping").column();
        grp.set_margin(0); // Remove margin to match old version - no extra padding
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
        ctrl_row.add(&opt_input);

        // Start/Stop button (fixed width, right aligned)
        let is_running = config.modules.get_bool("ping", "running").unwrap_or(false);
        let mut start_btn = Button::default().with_label(if is_running { "Stop" } else { "Start" });
        start_btn.set_color(if is_running { fltk::enums::Color::from_hex(super::ui_refresh::HTTP_STOP_COLOR) } else { colors.accent });
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
        results_display.set_frame(fltk::enums::FrameType::FlatBox); // Remove border to match old version
        results_display.set_scrollbar_size(10); // Smaller scrollbar to save space

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
            let target = addr_input_clone.value();
            let options = opt_input_clone.value();

            // Parse options
            let mut interval = 1000i64;
            let mut count = -1i64;
            let mut stop_on_loss = false;

            for opt in options.split(';') {
                let parts: Vec<&str> = opt.splitn(2, '=').collect();
                if parts.len() == 2 {
                    match parts[0].trim() {
                        "interval" => {
                            if let Ok(v) = parts[1].parse::<i64>() {
                                interval = v;
                            }
                        }
                        "count" => {
                            if let Ok(v) = parts[1].parse::<i64>() {
                                count = v;
                            }
                        }
                        "stoponloss" => {
                            if let Ok(v) = parts[1].parse::<bool>() {
                                stop_on_loss = v;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Always save config on button click
            if let Ok(mut config) = load_config() {
                config.modules.insert("ping", "interval", ConfigValue::Integer(interval));
                config.modules.insert("ping", "count", ConfigValue::Integer(count));
                config.modules.insert("ping", "stoponloss", ConfigValue::Boolean(stop_on_loss));
                config.modules.insert("ping", "target", ConfigValue::String(target.clone()));
                let _ = save_config(&config);
            }

            if label == "Start" {
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

                send_event(UiEvent::ModuleToggle { module: "ping".into() });

                start_btn_clone.set_label("Stop");
                start_btn_clone.set_color(fltk::enums::Color::from_hex(0xE57373));
            } else {
                send_event(UiEvent::ModuleToggle { module: "ping".into() });
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
