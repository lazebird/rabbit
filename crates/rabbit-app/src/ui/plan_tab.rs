//! Plan Tab UI Component
//!
//! Layout matching old version:
//! - Single row: Date [input] Time [input] Repeat/ [input] [unit dropdown] Opt. [input] [+] [-]
//! - Event list fills remaining space

use fltk::{
    button::Button,
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    menu::Choice,
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors};

/// Plan Tab Component
pub struct PlanTab;

impl TabComponent for PlanTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "PLAN").column();
        grp.set_margin(8);
        grp.set_spacing(5);

        // Control row - matching old version compact layout
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // Date label
        let _date_label = Frame::default().with_label("Date");
        ctrl_row.fixed(&_date_label, 30);

        // Date input
        let mut date_input = Input::default();
        let now = chrono::Local::now();
        date_input.set_value(&now.format("%Y/%m/%d").to_string());
        ctrl_row.fixed(&date_input, 85);

        // Time label
        let _time_label = Frame::default().with_label("Time");
        ctrl_row.fixed(&_time_label, 30);

        // Time input
        let mut time_input = Input::default();
        time_input.set_value(&now.format("%H:%M").to_string());
        ctrl_row.fixed(&time_input, 45);

        // Now button - fills date/time with current time
        let mut now_btn = Button::default().with_label("Now");
        ctrl_row.fixed(&now_btn, 35);

        // Repeat label
        let _repeat_label = Frame::default().with_label("Repeat/");
        ctrl_row.fixed(&_repeat_label, 45);

        // Cycle input (small)
        let mut cycle_input = IntInput::default();
        cycle_input.set_value("0");
        ctrl_row.fixed(&cycle_input, 35);

        // Unit dropdown
        let mut unit_choice = Choice::default();
        unit_choice.add_choice("minute");
        unit_choice.add_choice("hour");
        unit_choice.add_choice("day");
        unit_choice.set_value(0);
        ctrl_row.fixed(&unit_choice, 70);

        // Opt. label
        let _opt_label = Frame::default().with_label("Opt.");
        ctrl_row.fixed(&_opt_label, 30);

        // Options input (takes remaining space)
        let mut opt_input = Input::default();
        opt_input.set_value("override=false");

        // Add button (small square)
        let mut add_btn = Button::default().with_label("+");
        add_btn.set_color(colors.accent);
        add_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&add_btn, 28);

        // Remove button (small square)
        let mut remove_btn = Button::default().with_label("-");
        remove_btn.set_color(fltk::enums::Color::from_hex(0xE57373));
        remove_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&remove_btn, 28);

        ctrl_row.end();
        grp.fixed(&ctrl_row, 28);

        // Event list fills remaining space
        let mut event_display = TextDisplay::default();
        let event_buf = TextBuffer::default();
        event_display.set_buffer(Some(event_buf));
        event_display.wrap_mode(WrapMode::AtBounds, 0);

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        event_display.set_color(colors.input_bg);
        event_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_list(&mut event_display);

        // Clone inputs for callbacks
        let date_input_clone = date_input.clone();
        let time_input_clone = time_input.clone();
        let cycle_input_clone = cycle_input.clone();
        let unit_choice_clone = unit_choice.clone();
        let opt_input_clone = opt_input.clone();

        // Now button - fills date and time with current time
        let mut date_input_now = date_input.clone();
        let mut time_input_now = time_input.clone();
        now_btn.set_callback(move |_| {
            let now = chrono::Local::now();
            date_input_now.set_value(&now.format("%Y/%m/%d").to_string());
            time_input_now.set_value(&now.format("%H:%M").to_string());
        });

        // Add button callback
        add_btn.set_callback(move |_| {
            let date = date_input_clone.value();
            let time = time_input_clone.value();
            let cycle = cycle_input_clone.value().parse::<i32>().unwrap_or(0);
            let unit = match unit_choice_clone.value() {
                0 => "minute",
                1 => "hour",
                2 => "day",
                _ => "minute",
            }.to_string();
            let msg = opt_input_clone.value();

            if date.is_empty() || time.is_empty() {
                fltk::dialog::alert_default("Please enter date and time!");
                return;
            }

            // Generate event ID and update UI state
            let event_id = format!("{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    let cycle_str = if cycle > 0 {
                        format!(" (Repeat every {} {})", cycle, unit)
                    } else {
                        " (One-time)".to_string()
                    };
                    s.plan_list.push_str(&format!(
                        "[{}] {} {} - {}{}\n",
                        event_id, date, time, msg, cycle_str
                    ));
                    s.updated.insert("plan_list".to_string(), true);
                }
            }

            send_event(UiEvent::PlanAdd { date, time, cycle, unit, msg });
        });

        // Remove button callback
        let mut event_display_remove = event_display.clone();
        remove_btn.set_callback(move |_| {
            let remove_id = fltk::dialog::input_default("Enter event ID to remove:", "");
            if let Some(id) = remove_id {
                let id = id.trim();
                if !id.is_empty() {
                    crate::ui_state::remove_plan_event(id);
                    Self::refresh_list(&mut event_display_remove);
                    send_event(UiEvent::PlanRemove { id: id.to_string() });
                }
            }
        });

        // Register display with centralized refresh manager
        super::ui_refresh::register_display("plan_list", event_display.clone());

        grp
    }
}

impl PlanTab {
    fn refresh_list(display: &mut TextDisplay) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.plan_list);
                    let lines = buf.count_lines(0, buf.length());
                    display.scroll(lines, 0);
                }
            }
        }
    }
}
