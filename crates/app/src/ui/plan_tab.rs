//! Plan Tab UI Component
//!
//! Layout:
//! - Row 1: Date [input] Time [input] Repeat/ [input] [unit dropdown] [msg input] Opt [input] [+][-]
//! - Row 2: Task list browser (fixed height, scrollable, double-click to fill config)
//! - Row 3: Log output (fills remaining space)

use fltk::{
    browser::{Browser, BrowserType},
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
use schema::config::PlanTask;
use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};

/// Plan Tab Component
pub struct PlanTab;

impl TabComponent for PlanTab {
    fn build(x: i32, y: i32, w: i32, h: i32, _config: &schema::AppConfig) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "PLAN").column();
        grp.set_margin(0);
        grp.set_spacing(4);

        // ===== Row 1: Config bar =====
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // Date label
        let _date_label = Frame::default().with_label("Date");
        ctrl_row.fixed(&_date_label, 30);

        // Date input
        let mut date_input = Input::default();
        let now = Local::now();
        date_input.set_value(&now.format("%Y/%m/%d").to_string());
        ctrl_row.fixed(&date_input, 85);

        // Date picker
        let date_input_clone = date_input.clone();
        date_input.handle(move |_, ev| {
            if ev == Event::Push {
                show_date_picker(&date_input_clone, fltk::app::event_x_root(), fltk::app::event_y_root());
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
        let now = Local::now();
        time_input.set_value(&now.format("%H:%M").to_string());
        ctrl_row.fixed(&time_input, 45);

        // Time picker
        let time_input_clone = time_input.clone();
        time_input.handle(move |_, ev| {
            if ev == Event::Push {
                show_time_picker(&time_input_clone, fltk::app::event_x_root(), fltk::app::event_y_root());
                true
            } else {
                false
            }
        });

        // Repeat label
        let _repeat_label = Frame::default().with_label("Repeat/");
        ctrl_row.fixed(&_repeat_label, 45);

        // Cycle input
        let mut cycle_input = Input::default();
        cycle_input.set_value("0");
        ctrl_row.fixed(&cycle_input, 30);

        // Unit dropdown
        let mut unit_choice = Choice::default();
        unit_choice.add_choice("minute|hour|day");
        unit_choice.set_value(0);
        ctrl_row.fixed(&unit_choice, 70);

        // Message input (no label)
        let mut msg_input = Input::default();
        msg_input.set_value("");

        // Opt label
        let _opt_label = Frame::default().with_label("Opt");
        ctrl_row.fixed(&_opt_label, 25);

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

        // ===== Row 2: Task list browser (fixed height) =====
        let mut task_browser = Browser::default();
        task_browser.set_type(BrowserType::Hold);
        task_browser.set_text_size(13);
        task_browser.set_color(colors.input_bg);
        grp.fixed(&task_browser, 300);

        // ===== Row 3: Log output (fills remaining space) =====
        let mut log_display = TextDisplay::default();
        let log_buf = TextBuffer::default();
        log_display.set_buffer(Some(log_buf));
        log_display.wrap_mode(WrapMode::AtBounds, 0);
        log_display.set_frame(fltk::enums::FrameType::FlatBox);
        log_display.set_color(colors.input_bg);
        log_display.set_text_color(colors.text);
        crate::ui::ui_refresh::register_display("plan_output", log_display.clone());

        grp.end();

        // Apply styling
        grp.set_color(colors.background);

        // Load saved tasks from config (structured storage)
        let saved_tasks: Vec<PlanTask> = crate::ui_state::load_plan_tasks();
        for task in &saved_tasks {
            // Display format: "msg (Repeat every X unit)\t@date time"
            let cycle_str = if task.cycle > 0 {
                format!(" (Repeat every {} {})", task.cycle, task.unit)
            } else {
                String::new()
            };
            let entry = format!("{}{}\t@{} {}", task.msg, cycle_str, task.date, task.time);
            task_browser.add(&entry);
        }

        // ===== Callbacks =====

        // Clone values for add callback
        let date_clone = date_input.clone();
        let time_clone = time_input.clone();
        let cycle_clone = cycle_input.clone();
        let unit_clone = unit_choice.clone();
        let opt_clone = opt_input.clone();
        let msg_clone = msg_input.clone();
        let mut browser_add = task_browser.clone();
        let log_add = log_display.clone();

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
            let _opt = opt_clone.value();
            let msg = msg_clone.value();

            if msg.is_empty() {
                if let Some(mut buf) = log_add.buffer() {
                    buf.append(&format!("[{}] Error: Msg is empty\n", chrono::Local::now().format("%H:%M:%S")));
                }
                return;
            }

            if date.is_empty() || time.is_empty() {
                if let Some(mut buf) = log_add.buffer() {
                    buf.append(&format!("[{}] Error: Date or time is empty\n", chrono::Local::now().format("%H:%M:%S")));
                }
                return;
            }

            // Format: "[Msg] \t @[date] [time]"
            let cycle_str = if cycle > 0 { format!(" (Repeat every {} {})", cycle, unit) } else { String::new() };
            let entry = format!("{}{}\t@{} {}", msg, cycle_str, date, time);

            // Check if already exists, update or add
            let mut found = false;
            for i in 1..=browser_add.size() {
                if let Some(text) = browser_add.text(i) {
                    // Match by msg part (before \t)
                    if let Some(tab_pos) = text.find('\t') {
                        let existing_msg = &text[..tab_pos];
                        // Check if msg matches (handle repeat suffix)
                        if existing_msg == msg || existing_msg.starts_with(&format!("{} (", msg)) {
                            browser_add.remove(i);
                            browser_add.insert(i, &entry);
                            found = true;
                            break;
                        }
                    }
                }
            }
            if !found {
                browser_add.add(&entry);
            }

            // Log
            if let Some(mut buf) = log_add.buffer() {
                buf.append(&format!("[{}] Added: {}\n", chrono::Local::now().format("%H:%M:%S"), entry));
            }

            // Save tasks to config (structured)
            let mut plan_tasks: Vec<PlanTask> = Vec::new();
            for i in 1..=browser_add.size() {
                if let Some(text) = browser_add.text(i) {
                    // Parse display format: "msg (Repeat every X unit)\t@date time"
                    let parts: Vec<&str> = text.split('\t').collect();
                    if parts.len() >= 2 {
                        let msg_part = parts[0];
                        let datetime_part = parts[1].trim_start_matches('@');

                        let mut msg = msg_part.to_string();
                        let mut cycle = 0;
                        let mut unit = "minute".to_string();

                        // Parse repeat info from msg_part
                        if let Some(idx) = msg_part.find(" (Repeat every ") {
                            msg = msg_part[..idx].to_string();
                            let repeat_str = &msg_part[idx + 15..]; // Skip " (Repeat every "
                            let repeat_parts: Vec<&str> = repeat_str.trim_end_matches(')').splitn(2, ' ').collect();
                            if repeat_parts.len() == 2 {
                                cycle = repeat_parts[0].parse().unwrap_or(0);
                                unit = repeat_parts[1].to_string();
                            }
                        }

                        // Parse date and time
                        let dt_parts: Vec<&str> = datetime_part.splitn(2, ' ').collect();
                        if dt_parts.len() == 2 {
                            let date = dt_parts[0].to_string();
                            let time = dt_parts[1].to_string();
                            plan_tasks.push(PlanTask {
                                msg,
                                date,
                                time,
                                cycle,
                                unit,
                            });
                        }
                    }
                }
            }
            crate::ui_state::save_plan_tasks(&plan_tasks);

            // Send event
            send_event(UiEvent::PlanAdd { date, time, cycle, unit, msg });
        });

        // Clone values for remove callback
        let msg_remove = msg_input.clone();
        let mut browser_remove = task_browser.clone();
        let log_remove = log_display.clone();

        // Remove button callback
        remove_btn.set_callback(move |_| {
            let msg = msg_remove.value();

            if msg.is_empty() {
                if let Some(mut buf) = log_remove.buffer() {
                    buf.append(&format!("[{}] Error: Msg is empty\n", chrono::Local::now().format("%H:%M:%S")));
                }
                return;
            }

            // Find and remove from browser by msg (before \t)
            let mut removed = false;
            for i in 1..=browser_remove.size() {
                if let Some(text) = browser_remove.text(i) {
                    if let Some(tab_pos) = text.find('\t') {
                        let existing_msg = &text[..tab_pos];
                        if existing_msg == msg || existing_msg.starts_with(&format!("{} (", msg)) {
                            browser_remove.remove(i);
                            removed = true;
                            break;
                        }
                    }
                }
            }

            // Log
            if let Some(mut buf) = log_remove.buffer() {
                if removed {
                    buf.append(&format!("[{}] Removed: {}\n", chrono::Local::now().format("%H:%M:%S"), msg));
                } else {
                    buf.append(&format!("[{}] Not found: {}\n", chrono::Local::now().format("%H:%M:%S"), msg));
                }
            }

            // Save tasks to config (structured)
            if removed {
                let mut plan_tasks: Vec<PlanTask> = Vec::new();
                for i in 1..=browser_remove.size() {
                    if let Some(text) = browser_remove.text(i) {
                        // Parse display format: "msg (Repeat every X unit)\t@date time"
                        let parts: Vec<&str> = text.split('\t').collect();
                        if parts.len() >= 2 {
                            let msg_part = parts[0];
                            let datetime_part = parts[1].trim_start_matches('@');

                            let mut msg = msg_part.to_string();
                            let mut cycle = 0;
                            let mut unit = "minute".to_string();

                            // Parse repeat info from msg_part
                            if let Some(idx) = msg_part.find(" (Repeat every ") {
                                msg = msg_part[..idx].to_string();
                                let repeat_str = &msg_part[idx + 15..]; // Skip " (Repeat every "
                                let repeat_parts: Vec<&str> = repeat_str.trim_end_matches(')').splitn(2, ' ').collect();
                                if repeat_parts.len() == 2 {
                                    cycle = repeat_parts[0].parse().unwrap_or(0);
                                    unit = repeat_parts[1].to_string();
                                }
                            }

                            // Parse date and time
                            let dt_parts: Vec<&str> = datetime_part.splitn(2, ' ').collect();
                            if dt_parts.len() == 2 {
                                let date = dt_parts[0].to_string();
                                let time = dt_parts[1].to_string();
                                plan_tasks.push(PlanTask {
                                    msg,
                                    date,
                                    time,
                                    cycle,
                                    unit,
                                });
                            }
                        }
                    }
                }
                crate::ui_state::save_plan_tasks(&plan_tasks);
            }

            send_event(UiEvent::PlanRemove { msg });
        });

        // Double-click on task browser to fill config fields
        let mut date_fill = date_input.clone();
        let mut time_fill = time_input.clone();
        let mut cycle_fill = cycle_input.clone();
        let mut unit_fill = unit_choice.clone();
        let mut msg_fill = msg_input.clone();

        task_browser.handle(move |browser, ev| {
            if ev == Event::Push && fltk::app::event_clicks() {
                if let Some(text) = browser.text(browser.value()) {
                    // Format: "[Msg] (Repeat every X unit)\t@[date] [time]"
                    let parts: Vec<&str> = text.split('\t').collect();
                    if parts.len() >= 2 {
                        // Parse date/time from right part: "@2026/05/06 12:00"
                        let datetime_part = parts[1].trim_start_matches('@');
                        let dt_parts: Vec<&str> = datetime_part.splitn(2, ' ').collect();
                        if dt_parts.len() == 2 {
                            date_fill.set_value(dt_parts[0]);
                            time_fill.set_value(dt_parts[1]);
                        }

                        // Parse msg and repeat from left part
                        let left = parts[0];
                        if let Some(idx) = left.find(" (Repeat every ") {
                            let msg_part = &left[..idx];
                            let repeat_part = &left[idx + 15..]; // Skip " (Repeat every "
                            let repeat_parts: Vec<&str> = repeat_part.trim_end_matches(')').splitn(2, ' ').collect();
                            if repeat_parts.len() == 2 {
                                cycle_fill.set_value(repeat_parts[0]);
                                let unit_val = match repeat_parts[1] {
                                    "minute" => 0,
                                    "hour" => 1,
                                    "day" => 2,
                                    _ => 0,
                                };
                                unit_fill.set_value(unit_val);
                            }
                            msg_fill.set_value(msg_part);
                        } else {
                            msg_fill.set_value(left);
                            cycle_fill.set_value("0");
                        }
                    }
                }
                true
            } else {
                false
            }
        });

        // Register add button for Enter key support
        super::ui_refresh::register_plan_button(add_btn, colors.accent);

        grp
    }
}

/// Show date picker dialog
fn show_date_picker(date_input: &Input, px: i32, py: i32) {
    let current_val = date_input.value();
    let init_date = NaiveDate::parse_from_str(&current_val, "%Y/%m/%d").unwrap_or_else(|_| Local::now().naive_local().date());
    let today = Local::now().naive_local().date();
    let current_year = today.year();
    let init_year = init_date.year();
    let days_in_month = get_days_in_month(init_year, init_date.month());

    let win_w = 280;
    let win_h = 90;
    let mut win = Window::new(px, py, win_w, win_h, "Select Date");
    win.make_modal(true);

    // Year/Month/Day row
    let mut ymd_row = Flex::default().row().with_pos(8, 6).with_size(win_w - 16, 28);
    ymd_row.set_spacing(6);

    let mut year_choice = Choice::default();
    for y in (current_year - 5)..=(current_year + 5) {
        year_choice.add_choice(&y.to_string());
    }
    year_choice.set_value(init_year - (current_year - 5));
    ymd_row.fixed(&year_choice, 80);

    let mut month_choice = Choice::default();
    for m in 1..=12 {
        month_choice.add_choice(&m.to_string());
    }
    month_choice.set_value((init_date.month() as i32) - 1);
    ymd_row.fixed(&month_choice, 60);

    let mut day_choice = Choice::default();
    for d in 1..=days_in_month {
        day_choice.add_choice(&d.to_string());
    }
    day_choice.set_value((init_date.day() as i32) - 1);
    ymd_row.fixed(&day_choice, 60);

    ymd_row.end();

    // Button row
    let btn_w = (win_w - 32) / 3;
    let mut btn_row = Flex::default().row().with_pos(8, 52).with_size(win_w - 16, 28);
    btn_row.set_spacing(6);

    let mut today_btn = Button::default().with_label("Today");
    btn_row.fixed(&today_btn, btn_w);

    let mut ok_btn = Button::default().with_label("OK");
    ok_btn.set_color(Color::from_rgb(76, 175, 80));
    ok_btn.set_label_color(Color::White);
    btn_row.fixed(&ok_btn, btn_w);

    let mut cancel_btn = Button::default().with_label("Cancel");
    cancel_btn.set_color(Color::from_rgb(229, 115, 115));
    cancel_btn.set_label_color(Color::White);
    btn_row.fixed(&cancel_btn, btn_w);

    btn_row.end();
    win.end();
    win.show();

    // Today button action - only update dropdowns inside the picker window
    let mut year_choice_today = year_choice.clone();
    let mut month_choice_today = month_choice.clone();
    let mut day_choice_today = day_choice.clone();
    today_btn.set_callback(move |_| {
        let now = Local::now();
        let today = now.naive_local().date();
        year_choice_today.set_value(today.year() - (current_year - 5));
        month_choice_today.set_value((today.month() as i32) - 1);
        // Refresh day_choice items for the correct month
        day_choice_today.clear();
        let days = get_days_in_month(today.year(), today.month());
        for d in 1..=days {
            day_choice_today.add_choice(&d.to_string());
        }
        day_choice_today.set_value((today.day() as i32) - 1);
    });

    // OK button action
    let mut win_ok = win.clone();
    let mut date_input_ok = date_input.clone();
    ok_btn.set_callback(move |_| {
        let y = year_choice.value() + (current_year - 5);
        let m = month_choice.value() as u32 + 1;
        let d = day_choice.value() as u32 + 1; // day_choice.value() is 0-based index
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
fn show_time_picker(time_input: &Input, px: i32, py: i32) {
    let current_val = time_input.value();
    let init_time = NaiveTime::parse_from_str(&current_val, "%H:%M").unwrap_or_else(|_| Local::now().naive_local().time());

    let win_w = 270;
    let win_h = 90;
    let mut win = Window::new(px, py, win_w, win_h, "Select Time");
    win.make_modal(true);

    // Hour/Minute row
    let mut hm_row = Flex::default().row().with_pos(8, 6).with_size(win_w - 16, 28);
    hm_row.set_spacing(6);

    let mut hour_choice = Choice::default();
    for h in 0..24 {
        hour_choice.add_choice(&h.to_string());
    }
    hour_choice.set_value(init_time.hour() as i32);
    hm_row.fixed(&hour_choice, 80);

    let mut minute_choice = Choice::default();
    for m in 0..60 {
        minute_choice.add_choice(&m.to_string());
    }
    minute_choice.set_value(init_time.minute() as i32);
    hm_row.fixed(&minute_choice, 80);

    hm_row.end();

    // Button row
    let btn_w = (win_w - 32) / 3;
    let mut btn_row = Flex::default().row().with_pos(8, 52).with_size(win_w - 16, 28);
    btn_row.set_spacing(6);

    let mut now_btn = Button::default().with_label("Now");
    btn_row.fixed(&now_btn, btn_w);

    let mut ok_btn = Button::default().with_label("OK");
    ok_btn.set_color(Color::from_rgb(76, 175, 80));
    ok_btn.set_label_color(Color::White);
    btn_row.fixed(&ok_btn, btn_w);

    let mut cancel_btn = Button::default().with_label("Cancel");
    cancel_btn.set_color(Color::from_rgb(229, 115, 115));
    cancel_btn.set_label_color(Color::White);
    btn_row.fixed(&cancel_btn, btn_w);

    btn_row.end();
    win.end();
    win.show();

    // Now button action - only update dropdowns inside the picker window
    let mut hour_choice_now = hour_choice.clone();
    let mut minute_choice_now = minute_choice.clone();
    now_btn.set_callback(move |_| {
        let now = Local::now();
        hour_choice_now.set_value(now.hour() as i32);
        minute_choice_now.set_value(now.minute() as i32);
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
