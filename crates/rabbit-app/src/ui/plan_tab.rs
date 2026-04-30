//! Plan Tab UI Component
//!
//! Layout matching old version:
//! - Single row: Date [input] Time [input] Repeat/ [input] [unit dropdown] Opt. [input] [+] [-]
//! - Event list fills remaining space

use fltk::{
    button::Button,
    enums::{Align, Color, Event},
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
use crate::ui_state::UiState;
use chrono::{Datelike, Local, NaiveDate, Timelike};

/// Plan Tab Component
pub struct PlanTab;

impl TabComponent for PlanTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "PLAN").column();
        grp.set_margin(0); // Remove margin to match old version
        grp.set_spacing(4);

        // Control row - matching old version compact layout
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // Date label
        let _date_label = Frame::default().with_label("Date");
        ctrl_row.fixed(&_date_label, 30);

        // Date input - click to show date picker
        let mut date_input = Input::default();
        let plan_date_val = defaults::plan_date();
        if plan_date_val.is_empty() {
            let now = Local::now();
            date_input.set_value(&now.format("%Y/%m/%d").to_string());
        } else {
            date_input.set_value(&plan_date_val);
        }
        ctrl_row.fixed(&date_input, 85);

        // Time label
        let _time_label = Frame::default().with_label("Time");
        ctrl_row.fixed(&_time_label, 30);

        // Time input - click to show time picker
        let mut time_input = Input::default();
        let plan_time_val = defaults::plan_time();
        if plan_time_val.is_empty() {
            let now = Local::now();
            time_input.set_value(&now.format("%H:%M").to_string());
        } else {
            time_input.set_value(&plan_time_val);
        }
        ctrl_row.fixed(&time_input, 45);

        // Date picker - show calendar dialog when user clicks
        let mut date_input_clone = date_input.clone();
        date_input.handle(move |_inp, ev| {
            if ev == Event::Push {
                show_date_picker(&mut date_input_clone);
                true
            } else {
                false
            }
        });

        // Time picker - show spinner dialog when user clicks
        let mut time_input_clone = time_input.clone();
        time_input.handle(move |_inp, ev| {
            if ev == Event::Push {
                show_time_picker(&mut time_input_clone);
                true
            } else {
                false
            }
        });

        // Now button - fills date/time with current time
        let mut now_btn = Button::default().with_label("Now");
        ctrl_row.fixed(&now_btn, 35);

        // Repeat label
        let _repeat_label = Frame::default().with_label("Repeat/");
        ctrl_row.fixed(&_repeat_label, 45);

        // Cycle input (small)
        let mut cycle_input = Input::default();
        cycle_input.set_value(&defaults::plan_cycle());
        ctrl_row.fixed(&cycle_input, 35);

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

        // Options input (takes remaining space)
        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::plan_options());

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
        event_display.set_frame(fltk::enums::FrameType::FlatBox); // Remove border

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
            let now = Local::now();
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
            }
            .to_string();
            let msg = opt_input_clone.value();

            // Save plan configuration before sending event (which moves values)
            crate::ui_state::sync_plan_config(date.clone(), time.clone(), cycle, &unit, msg.clone());

            if date.is_empty() || time.is_empty() {
                fltk::dialog::alert_default("Please enter date and time!");
                return;
            }

            // Generate event ID and update UI state
            let event_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    let cycle_str = if cycle > 0 { format!(" (Repeat every {} {})", cycle, unit) } else { " (One-time)".to_string() };
                    s.plan_list.push_str(&format!("[{}] {} {} - {}{}\n", event_id, date, time, msg, cycle_str));
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

        // Register add button for Enter key support (primary action)
        super::ui_refresh::register_plan_button(add_btn.clone(), colors.accent);

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

/// Show a professional date picker dialog with dropdown choices and Today button.
fn show_date_picker(date_input: &mut Input) {
    let current_val = date_input.value();
    let init_date = NaiveDate::parse_from_str(&current_val, "%Y/%m/%d").unwrap_or_else(|_| Local::now().naive_local().date());

    let today = Local::now().naive_local().date();
    let current_year = today.year();
    let init_year = init_date.year();
    let days_in_month = get_days_in_month(init_year, init_date.month());

    // Compact window - buttons at bottom
    let win_w = 280;
    let win_h = 90;
    let mut win = Window::new(0, 0, win_w, win_h, "Select Date");
    win.make_modal(true);

    // Dropdown row at top
    let mut ymd_row = Flex::default().row().with_pos(8, 6).with_size(win_w - 16, 28);
    ymd_row.set_spacing(6);

    let mut year_choice = Choice::default();
    for y in (current_year - 5)..=(current_year + 5) {
        year_choice.add_choice(&y.to_string());
        if y == init_year {
            year_choice.set_value((y - (current_year - 5)).max(0));
        }
    }
    ymd_row.fixed(&year_choice, 80);

    let mut month_choice = Choice::default();
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    for (i, m) in months.iter().enumerate() {
        month_choice.add_choice(m);
        if (i as u32) == init_date.month() - 1 {
            month_choice.set_value(i as i32);
        }
    }
    ymd_row.fixed(&month_choice, 70);

    let mut day_choice = Choice::default();
    for d in 1..=days_in_month {
        day_choice.add_choice(&d.to_string());
        if d == init_date.day() {
            day_choice.set_value((d - 1) as i32);
        }
    }
    ymd_row.fixed(&day_choice, 60);

    ymd_row.end();

    // Button row at bottom
    let mut btn_row = Flex::default().row().with_pos(8, 56).with_size(win_w - 16, 28);
    btn_row.set_spacing(6);

    let mut today_btn = Button::default().with_label("Today");
    today_btn.set_color(Color::from_hex(0x4A90D9));
    today_btn.set_label_color(Color::White);
    btn_row.fixed(&today_btn, 70);

    let mut cancel_btn = Button::default().with_label("Cancel");
    btn_row.fixed(&cancel_btn, 70);

    let mut ok_btn = Button::default().with_label("OK");
    ok_btn.set_color(Color::from_hex(0x4A90D9));
    ok_btn.set_label_color(Color::White);
    btn_row.fixed(&ok_btn, 70);

    btn_row.end();
    win.end();

    // Helper to update day dropdown when month/year changes
    let update_days = move |choice: &mut Choice, year: i32, month: u32, selected_day: Option<u32>| {
        let days = get_days_in_month(year, month);
        let current_val = choice.value();
        let current_day = if current_val >= 0 { (current_val + 1) as u32 } else { 1 };
        choice.clear();
        for d in 1..=days {
            choice.add_choice(&d.to_string());
        }
        let day_to_select = selected_day.unwrap_or(current_day.min(days));
        choice.set_value((day_to_select as i32 - 1).max(0).min(days as i32 - 1));
    };

    // Today button
    let mut year_choice_today = year_choice.clone();
    let mut month_choice_today = month_choice.clone();
    let mut day_choice_today = day_choice.clone();
    let update_days_today = update_days;
    today_btn.set_callback(move |_| {
        let today = Local::now().naive_local().date();
        let base_year = today.year() - 5;
        year_choice_today.set_value((today.year() - base_year).clamp(0, 10));
        month_choice_today.set_value((today.month() - 1) as i32);
        update_days_today(&mut day_choice_today, today.year(), today.month(), Some(today.day()));
    });

    // Year/Month change - update day range
    let mut day_choice_update = day_choice.clone();
    let month_choice_ref = month_choice.clone();
    let update_days_ref = update_days;
    year_choice.set_callback(move |c: &mut Choice| {
        let base_year = today.year() - 5;
        let yr = base_year + c.value();
        let mo = (month_choice_ref.value() + 1) as u32;
        update_days_ref(&mut day_choice_update, yr, mo, None);
    });

    let mut day_choice_update2 = day_choice.clone();
    let year_choice_ref = year_choice.clone();
    let update_days_ref2 = update_days;
    month_choice.set_callback(move |c: &mut Choice| {
        let base_year = today.year() - 5;
        let yr = base_year + year_choice_ref.value();
        let mo = (c.value() + 1) as u32;
        update_days_ref2(&mut day_choice_update2, yr, mo, None);
    });

    // Cancel
    let mut win_cancel = win.clone();
    cancel_btn.set_callback(move |_| {
        win_cancel.hide();
    });

    // OK
    let mut win_ok = win.clone();
    let mut date_inp = date_input.clone();
    ok_btn.set_callback(move |_| {
        let base_year = today.year() - 5;
        let yr = base_year + year_choice.value();
        let mo = (month_choice.value() + 1) as u32;
        let day = (day_choice.value() + 1) as u32;
        let date_str = format!("{}/{:02}/{:02}", yr, mo, day);
        date_inp.set_value(&date_str);
        win_ok.hide();
    });

    win.show();

    while win.shown() {
        if !fltk::app::wait() {
            break;
        }
    }

    win.hide();
}

/// Get the number of days in a given month and year.
fn get_days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_yr = if month == 12 { year + 1 } else { year };
    let first_of_next = NaiveDate::from_ymd_opt(next_yr, next_month, 1).unwrap_or_else(|| Local::now().naive_local().date());
    (first_of_next - chrono::Days::new(1)).day()
}

/// Show a time picker dialog with dropdown choices for hour/minute.
fn show_time_picker(time_input: &mut Input) {
    let current_val = time_input.value();
    let current_time = chrono::NaiveTime::parse_from_str(&current_val, "%H:%M").unwrap_or_else(|_| chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());

    let init_hour = current_time.hour() as i32;
    let init_minute = current_time.minute() as i32;

    // Compact window - buttons at bottom
    let win_w = 240;
    let win_h = 90;
    let mut win = Window::new(0, 0, win_w, win_h, "Select Time");
    win.make_modal(true);

    // Dropdown row at top
    let mut hm_row = Flex::default().row().with_pos(8, 6).with_size(win_w - 16, 28);
    hm_row.set_spacing(6);

    let mut hour_choice = Choice::default();
    for h in 0..24 {
        hour_choice.add_choice(&format!("{:02}", h));
    }
    hour_choice.set_value(init_hour);
    hm_row.fixed(&hour_choice, 80);

    let mut colon = Frame::default().with_label(":");
    colon.set_label_size(18);
    colon.set_align(Align::Center | Align::Inside);
    hm_row.fixed(&colon, 15);

    let mut minute_choice = Choice::default();
    for m in 0..60 {
        minute_choice.add_choice(&format!("{:02}", m));
    }
    minute_choice.set_value(init_minute);
    hm_row.fixed(&minute_choice, 80);

    hm_row.end();

    // Button row at bottom
    let mut btn_row = Flex::default().row().with_pos(8, 56).with_size(win_w - 16, 28);
    btn_row.set_spacing(6);

    let mut now_btn = Button::default().with_label("Now");
    btn_row.fixed(&now_btn, 70);

    let mut cancel_btn = Button::default().with_label("Cancel");
    btn_row.fixed(&cancel_btn, 70);

    let mut ok_btn = Button::default().with_label("OK");
    ok_btn.set_color(Color::from_hex(0x4A90D9));
    ok_btn.set_label_color(Color::White);
    btn_row.fixed(&ok_btn, 70);

    btn_row.end();
    win.end();

    // Now button
    let mut hour_choice_now = hour_choice.clone();
    let mut minute_choice_now = minute_choice.clone();
    now_btn.set_callback(move |_| {
        let now = Local::now();
        hour_choice_now.set_value(now.hour() as i32);
        minute_choice_now.set_value(now.minute() as i32);
    });

    // Cancel
    let mut win_cancel = win.clone();
    cancel_btn.set_callback(move |_| {
        win_cancel.hide();
    });

    // OK
    let mut win_ok = win.clone();
    let mut time_inp = time_input.clone();
    ok_btn.set_callback(move |_| {
        let h = hour_choice.value() as u32;
        let m = minute_choice.value() as u32;
        let time_str = format!("{:02}:{:02}", h, m);
        time_inp.set_value(&time_str);
        win_ok.hide();
    });

    win.show();

    while win.shown() {
        if !fltk::app::wait() {
            break;
        }
    }

    win.hide();
}
