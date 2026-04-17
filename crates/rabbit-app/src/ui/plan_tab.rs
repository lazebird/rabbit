//! Plan Tab UI Component
//!
//! Layout based on old version screenshot:
//! - Date picker: YYYY年MM月DD日
//! - Time picker: HH:MM
//! - Repeat / count | unit dropdown
//! - Opt. field
//! - +/- buttons
//! - Event list

use fltk::{
    button::Button,
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    menu::Choice,
    prelude::*,
    text::{TextBuffer, TextDisplay},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, Spacing};

/// Plan Tab Component
pub struct PlanTab;

impl TabComponent for PlanTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "PLAN").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Row 1: Date and Time pickers
        let mut row1 = Flex::default().row();
        row1.set_spacing(spacing.padding);

        let _date_label = Frame::default().with_label("Date");

        let mut date_input = Input::default();
        let now = chrono::Local::now();
        date_input.set_value(&now.format("%Y/%m/%d").to_string());

        let _time_label = Frame::default().with_label("Time");

        let mut time_input = Input::default();
        time_input.set_value(&now.format("%H:%M").to_string());

        Frame::default(); // Spacer
        row1.end();
        grp.fixed(&row1, spacing.row_height);

        // Row 2: Repeat and unit
        let mut row2 = Flex::default().row();
        row2.set_spacing(spacing.padding);

        let _repeat_label = Frame::default().with_label("Repeat");

        let mut cycle_input = IntInput::default();
        cycle_input.set_value("0");

        let mut unit_choice = Choice::default();
        unit_choice.add_choice("minute");
        unit_choice.add_choice("hour");
        unit_choice.add_choice("day");
        unit_choice.set_value(0);

        let _opt_label = Frame::default().with_label("Opt.");

        let mut opt_input = Input::default();
        opt_input.set_value("override=false");

        // Add/Remove buttons
        let mut remove_btn = Button::default().with_label("-");
        remove_btn.set_color(fltk::enums::Color::from_hex(0xE57373));

        let mut add_btn = Button::default().with_label("+");
        add_btn.set_color(colors.accent);
        add_btn.set_label_color(fltk::enums::Color::White);

        Frame::default(); // Spacer
        row2.end();
        grp.fixed(&row2, spacing.row_height);

        // Event list
        let mut event_display = TextDisplay::default();
        let event_buf = TextBuffer::default();
        event_display.set_buffer(Some(event_buf));

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
                    if s.plan_list.contains("No scheduled events") {
                        s.plan_list = format!("Scheduled Events:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                    }
                    s.plan_list.push_str(&format!(
                        "[{}] {} {} - {}{}\n",
                        event_id, date, time, msg, cycle_str
                    ));
                    s.updated.insert("plan_list".to_string(), true);
                }
            }

            send_event(UiEvent::PlanAdd { date, time, cycle, unit, msg });
        });

        // Remove button callback - remove last added event for now
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

        // Periodic refresh
        let mut event_display_timer = event_display.clone();
        fltk::app::add_idle3(move |_| {
            Self::refresh_list(&mut event_display_timer);
        });

        grp
    }
}

impl PlanTab {
    fn refresh_list(display: &mut TextDisplay) {
        if let Some(state) = UiState::global() {
            if let Ok(mut s) = state.lock() {
                if let Some(buf) = display.buffer() {
                    let current_text = buf.text();
                    if current_text != s.plan_list {
                        drop(buf);
                        if let Some(mut new_buf) = display.buffer() {
                            new_buf.set_text(&s.plan_list);
                            let lines = new_buf.count_lines(0, new_buf.length());
                            display.set_buffer(Some(new_buf));
                            display.scroll(lines, 0);
                        }
                    }
                }
                s.clear_updated("plan_list");
            }
        }
    }
}
