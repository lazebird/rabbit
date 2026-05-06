//! Plan Tab UI Component
//!
//! Layout matching old version:
//! - Single row: Date [input] Time [input] Repeat/ [input] [unit dropdown] Opt. [input] [+] [-]
//! - Event list fills remaining space

use fltk::{
    button::Button,
    enums::{Color, Event},
    frame::Frame,
    group::Flex,
    input::Input,
    menu::Choice,
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
    window::Window,
};

use super::{defaults, Colors, TabComponent};
use crate::ui_events::{send_event, UiEvent};
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};

/// Plan Tab Component
pub struct PlanTab;

impl TabComponent for PlanTab {
    fn build(x: i32, y: i32, w: i32, h: i32, _config: &rabbit_models::AppConfig) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "PLAN").column();
        grp.set_margin(0);
        grp.set_spacing(4);

        // Control row
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // Date label
        let _date_label = Frame::default().with_label("Date");
        ctrl_row.fixed(&_date_label, 30);

        // Date input
        let mut date_input = Input::default();
        let plan_date_val = defaults::plan_date();
        if plan_date_val.is_empty() {
            let now = Local::now();
            date_input.set_value(&now.format("%Y/%m/%d").to_string());
        } else {
            date_input.set_value(&plan_date_val);
        }
        ctrl_row.fixed(&date_input, 85);

        // Date picker
        let mut date_input_clone = date_input.clone();
        date_input.handle(move |_, ev| {
            if ev == Event::Push {
                show_date_picker(&mut date_input_clone);
                true
            } else {
                false
            }
        });

        // Time label
        let _time_label = Frame::default().with_label("Time");
        ctrl_row.fixed(&_time_label, 30);

        // Time input
        let mut time_input = Input::default();
        let plan_time_val = defaults::plan_time();
        if plan_time_val.is_empty() {
            let now = Local::now();
            time_input.set_value(&now.format("%H:%M").to_string());
        } else {
            time_input.set_value(&plan_time_val);
        }
        ctrl_row.fixed(&time_input, 45);

        // Time picker
        let mut time_input_clone = time_input.clone();
        time_input.handle(move |_, ev| {
            if ev == Event::Push {
                show_time_picker(&mut time_input_clone);
                true
            } else {
                false
            }
        });

        // Now button
        let mut now_btn = Button::default().with_label("Now");
        ctrl_row.fixed(&now_btn, 35);

        // Now button action
        let mut date_now = date_input.clone();
        let mut time_now = time_input.clone();
        now_btn.set_callback(move |_| {
            let now = Local::now();
            date_now.set_value(&now.format("%Y/%m/%d").to_string());
            time_now.set_value(&now.format("%H:%M").to_string());
        });

        // Repeat label
        let _repeat_label = Frame::default().with_label("Repeat/");
        ctrl_row.fixed(&_repeat_label, 45);

        // Cycle input
        let mut cycle_input = Input::default();
        cycle_input.set_value(&defaults::plan_cycle());
        ctrl_row.fixed(&cycle_input, 30);

        // Unit dropdown
        let mut unit_choice = Choice::default();
        unit_choice.add_choice("minute");
        unit_choice.add_choice("hour");
        unit_choice.add_choice("day");
        unit_choice.set_value(defaults::plan_unit());
        ctrl_row.fixed(&unit_choice, 70);

        // Opt. label
        let _opt_label = Frame::default().with_label("Opt.");
        ctrl_row.fixed(&_opt_label, 30);

        // Options input
        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::plan_options());

        // Add button
        let mut add_btn = Button::default().with_label("+");
        add_btn.set_color(colors.accent);
        add_btn.set_label_color(Color::White);
        ctrl_row.fixed(&add_btn, 28);

        // Remove button
        let mut remove_btn = Button::default().with_label("-");
        remove_btn.set_color(Color::from_hex(0xE57373));
        remove_btn.set_label_color(Color::White);
        ctrl_row.fixed(&remove_btn, 28);

        ctrl_row.end();
        grp.fixed(&ctrl_row, 28);

        // Event list
        let mut event_display = TextDisplay::default();
        let event_buf = TextBuffer::default();
        event_display.set_buffer(Some(event_buf));
        event_display.wrap_mode(WrapMode::AtBounds, 0);
        event_display.set_frame(fltk::enums::FrameType::FlatBox);

        // Apply styling
        grp.set_color(colors.background);
        event_display.set_color(colors.input_bg);
        event_display.set_text_color(colors.text);

        grp.end();

        // Clone values for callbacks
        let date_clone = date_input.clone();
        let time_clone = time_input.clone();
        let cycle_clone = cycle_input.clone();
        let unit_clone = unit_choice.clone();
        let opt_clone = opt_input.clone();
        let mut display_clone = event_display.clone();

        // Add button callback
        add_btn.set_callback(move |_| {
            let date = date_clone.value();
            let time = time_clone.value();
            let cycle = cycle_clone.value().parse::<i32>().unwrap_or(0);
            let unit = match unit_clone.value() {
                0 => "minute",
                1 => "hour",
                2 => "day",
                _ => "minute",
            }
            .to_string();
            let msg = opt_clone.value();

            if date.is_empty() || time.is_empty() {
                fltk::dialog::alert_default("Please enter date and time!");
                return;
            }

            // Generate event ID and update display directly
            let event_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
            let cycle_str = if cycle > 0 { format!(" (Repeat every {} {})", cycle, unit) } else { " (One-time)".to_string() };
            let new_line = format!("[{}] {} {} - {}{}\n", event_id, date, time, msg, cycle_str);

            // Update display directly
            if let Some(mut buf) = display_clone.buffer() {
                let current = buf.text();
                if current.contains("No scheduled events") || current.is_empty() {
                    buf.set_text(&format!("Scheduled Events:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n{}", new_line));
                } else {
                    buf.append(&new_line);
                }
                let lines = buf.count_lines(0, buf.length());
                display_clone.scroll(lines, 0);
            }

            // Save config and send event
            crate::ui_state::sync_plan_config(date.clone(), time.clone(), cycle, &unit, msg.clone());
            send_event(UiEvent::PlanAdd { date, time, cycle, unit, msg });
        });

        // Remove button callback
        let mut display_clone2 = event_display.clone();
        remove_btn.set_callback(move |_| {
            let remove_id = fltk::dialog::input_default("Enter event ID to remove:", "");
            if let Some(id) = remove_id {
                let id = id.trim();
                if !id.is_empty() {
                    // Remove from display directly
                    if let Some(mut buf) = display_clone2.buffer() {
                        let text = buf.text();
                        let lines: Vec<&str> = text.lines().collect();
                        let mut new_text = String::from("Scheduled Events:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                        let mut has_events = false;
                        for line in lines {
                            if line.starts_with("Scheduled") || line.starts_with("━") || line.trim().is_empty() {
                                continue;
                            }
                            if !line.starts_with(&format!("[{}]", id)) {
                                new_text.push_str(line);
                                new_text.push('\n');
                                has_events = true;
                            }
                        }
                        if !has_events {
                            new_text.push_str("No scheduled events.\n\nUse + button to add a new reminder.\n");
                        }
                        buf.set_text(&new_text);
                        let lines = buf.count_lines(0, buf.length());
                        display_clone2.scroll(lines, 0);
                    }
                    send_event(UiEvent::PlanRemove { id: id.to_string() });
                }
            }
        });

        // Register add button for Enter key support
        super::ui_refresh::register_plan_button(add_btn, colors.accent);

        grp
    }
}

impl PlanTab {
    // Plan list now updated directly in callbacks (event-driven)
}

/// Show date picker dialog
fn show_date_picker(date_input: &mut Input) {
    let current_val = date_input.value();
    let init_date = NaiveDate::parse_from_str(&current_val, "%Y/%m/%d").unwrap_or_else(|_| Local::now().naive_local().date());
    let today = Local::now().naive_local().date();
    let current_year = today.year();
    let init_year = init_date.year();
    let days_in_month = get_days_in_month(init_year, init_date.month());

    let win_w = 280;
    let win_h = 90;
    let mut win = Window::new(0, 0, win_w, win_h, "Select Date");
    win.make_modal(true);

    // Year/Month/Day row
    let mut ymd_row = Flex::default().row().with_pos(8, 6).with_size(win_w - 16, 28);
    ymd_row.set_spacing(6);

    let mut year_choice = Choice::default();
    for y in (current_year - 5)..=(current_year + 5) {
        year_choice.add_choice(&y.to_string());
        if y == init_year {
            year_choice.set_value(y - (current_year - 5));
        }
    }
    ymd_row.fixed(&year_choice, 80);

    let mut month_choice = Choice::default();
    for m in 1..=12 {
        month_choice.add_choice(&m.to_string());
        if m == init_date.month() {
            month_choice.set_value((m - 1) as i32);
        }
    }
    ymd_row.fixed(&month_choice, 60);

    let mut day_choice = Choice::default();
    for d in 1..=days_in_month {
        day_choice.add_choice(&d.to_string());
        if d == init_date.day() {
            day_choice.set_value((d - 1) as i32);
        }
    }
    ymd_row.fixed(&day_choice, 60);

    ymd_row.end();

    // Button row
    let mut btn_row = Flex::default().row().with_pos(8, 52).with_size(win_w - 16, 28);
    btn_row.set_spacing(6);

    let mut today_btn = Button::default().with_label("Today");
    btn_row.fixed(&today_btn, 80);

    let mut ok_btn = Button::default().with_label("OK");
    ok_btn.set_color(Color::from_rgb(76, 175, 80));
    ok_btn.set_label_color(Color::White);
    btn_row.fixed(&ok_btn, 80);

    let mut cancel_btn = Button::default().with_label("Cancel");
    cancel_btn.set_color(Color::from_rgb(229, 115, 115));
    cancel_btn.set_label_color(Color::White);
    btn_row.fixed(&cancel_btn, 80);

    btn_row.end();
    win.end();
    win.show();

    // Today button action
    let mut date_input_today = date_input.clone();
    today_btn.set_callback(move |_| {
        let now = Local::now().naive_local().date();
        date_input_today.set_value(&now.format("%Y/%m/%d").to_string());
    });

    // OK button action
    let mut win_ok = win.clone();
    let mut date_input_ok = date_input.clone();
    ok_btn.set_callback(move |_| {
        let y = year_choice.value() + (current_year - 5);
        let m = month_choice.value() as u32 + 1;
        let d = day_choice.value() as u32;
        let selected = NaiveDate::from_ymd_opt(y, m, d).unwrap_or_else(|| Local::now().naive_local().date());
        date_input_ok.set_value(&selected.format("%Y/%m/%d").to_string());
        win_ok.hide();
    });

    // Cancel button action
    let mut win_cancel = win.clone();
    cancel_btn.set_callback(move |_| {
        win_cancel.hide();
    });

    while win.shown() {
        fltk::app::wait();
    }
}

/// Show time picker dialog
fn show_time_picker(time_input: &mut Input) {
    let current_val = time_input.value();
    let init_time = NaiveTime::parse_from_str(&current_val, "%H:%M").unwrap_or_else(|_| Local::now().naive_local().time());

    let win_w = 200;
    let win_h = 90;
    let mut win = Window::new(0, 0, win_w, win_h, "Select Time");
    win.make_modal(true);

    // Hour/Minute row
    let mut hm_row = Flex::default().row().with_pos(8, 6).with_size(win_w - 16, 28);
    hm_row.set_spacing(6);

    let mut hour_choice = Choice::default();
    for h in 0..24 {
        hour_choice.add_choice(&h.to_string());
        if h == init_time.hour() {
            hour_choice.set_value(h as i32);
        }
    }
    hm_row.fixed(&hour_choice, 60);

    let mut minute_choice = Choice::default();
    for m in 0..60 {
        minute_choice.add_choice(&m.to_string());
        if m == init_time.minute() {
            minute_choice.set_value(m as i32);
        }
    }
    hm_row.fixed(&minute_choice, 60);

    hm_row.end();

    // Button row
    let mut btn_row = Flex::default().row().with_pos(8, 52).with_size(win_w - 16, 28);
    btn_row.set_spacing(6);

    let mut now_btn = Button::default().with_label("Now");
    btn_row.fixed(&now_btn, 80);

    let mut ok_btn = Button::default().with_label("OK");
    ok_btn.set_color(Color::from_rgb(76, 175, 80));
    ok_btn.set_label_color(Color::White);
    btn_row.fixed(&ok_btn, 80);

    let mut cancel_btn = Button::default().with_label("Cancel");
    cancel_btn.set_color(Color::from_rgb(229, 115, 115));
    cancel_btn.set_label_color(Color::White);
    btn_row.fixed(&cancel_btn, 80);

    btn_row.end();
    win.end();
    win.show();

    // Now button action
    let mut time_input_now = time_input.clone();
    now_btn.set_callback(move |_| {
        let now = Local::now().naive_local().time();
        time_input_now.set_value(&now.format("%H:%M").to_string());
    });

    // OK button action
    let mut win_ok = win.clone();
    let mut time_input_ok = time_input.clone();
    ok_btn.set_callback(move |_| {
        let h = hour_choice.value() as u32;
        let m = minute_choice.value() as u32;
        let selected = NaiveTime::from_hms_opt(h, m, 0).unwrap_or_else(|| Local::now().naive_local().time());
        time_input_ok.set_value(&selected.format("%H:%M").to_string());
        win_ok.hide();
    });

    // Cancel button action
    let mut win_cancel = win.clone();
    cancel_btn.set_callback(move |_| {
        win_cancel.hide();
    });

    while win.shown() {
        fltk::app::wait();
    }
}

/// Get days in month
fn get_days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let this_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let next_month = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
    (next_month - this_month).num_days() as u32
}
