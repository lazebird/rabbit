//! Fullscreen Black Reminder Window
//!
//! 1. Black fullscreen with message centered
//! 2. Countdown below message ("03:00" format, 3 minutes)
//! 3. Double-click MESSAGE to close (other areas ignore)
//! 4. Auto-close when countdown ends.

use fltk::{
    app, enums::*,
    frame::Frame,
    prelude::*,
    window::Window,
};
use std::rc::Rc;
use std::cell::RefCell;

use crate::ui_state;

struct RemState {
    remaining: i32,
    window: Window,
    cd_frame: Frame,
    message: String,
    closed: bool,
    close_reason: String,
}

pub fn show_reminder(message: &str) {
    let (sx, sy, sw, sh) = app::screen_xywh(0);

    let mut win = Window::new(sx, sy, sw, sh, "Reminder");
    win.set_color(Color::Black);
    win.set_border(false);
    win.set_override();

    // Message area (centered)
    let msg_h = 60;
    let cd_h = 40;
    let gap = 10;
    let total_h = msg_h + gap + cd_h;
    let start_y = (sh - total_h) / 2;

    let mut msg_frame = Frame::new(0, start_y, sw, msg_h, message);
    msg_frame.set_color(Color::Black);
    msg_frame.set_label_color(Color::White);
    msg_frame.set_label_font(Font::HelveticaBold);
    msg_frame.set_label_size(48);

    // Countdown frame (below message)
    let mut cd_frame = Frame::new(0, start_y + msg_h + gap, sw, cd_h, "03:00");
    cd_frame.set_color(Color::Black);
    cd_frame.set_label_color(Color::White);
    cd_frame.set_label_font(Font::Helvetica);
    cd_frame.set_label_size(36);

    // Shared state - Rc<RefCell>
    let msg_owned = message.to_string();
    let state = Rc::new(RefCell::new(RemState {
        remaining: 180,
        window: win.clone(),
        cd_frame: cd_frame.clone(),
        message: msg_owned,
        closed: false,
        close_reason: String::new(),
    }));

    // Double-click on MESSAGE FRAME ONLY to close
    let state_close = Rc::clone(&state);
    msg_frame.handle(move |_, ev| match ev {
        Event::Push => {
            if app::event_clicks() {
                let mut s = state_close.borrow_mut();
                s.close_reason = "closed by user double-click".to_string();
                drop(s);
                close_reminder(&state_close);
                return true;
            }
            false
        }
        _ => false,
    });

    win.end();
    win.show();
    // X11 (override-redirect): WM ignores this, no-op
    // Windows (WS_POPUP):      SetWindowPos HWND_TOPMOST, above taskbar
    win.set_on_top();

    ui_state::write_to("plan_output", &ui_state::fmt_log(&format!("Reminder triggered: {}", message)));
    crate::ui::ui_refresh::refresh_displays();

    // Countdown function - accepts Rc by reference
    fn tick(state: &Rc<RefCell<RemState>>) {
        let mut s = state.borrow_mut();
        if s.closed {
            return;
        }
        s.remaining -= 1;

        if s.remaining <= 0 {
            s.close_reason = "auto-closed after countdown".to_string();
            drop(s);
            close_reminder(state);
            return;
        }

        let minutes = s.remaining / 60;
        let seconds = s.remaining % 60;
        s.cd_frame.set_label(&format!("{:02}:{:02}", minutes, seconds));
        drop(s); // Release borrow before scheduling.

        // Schedule next tick - clone Rc for next call
        let next_state = Rc::clone(state);
        app::add_timeout3(1.0, move |_| {
            tick(&next_state);
        });
    }

    fn close_reminder(state: &Rc<RefCell<RemState>>) {
        let mut s = state.borrow_mut();
        if s.closed {
            return;
        }
        s.closed = true;
        s.window.hide();
        let reason = s.close_reason.clone();
        let msg = s.message.clone();
        drop(s);
        ui_state::write_to("plan_output", &ui_state::fmt_log(&format!("Reminder {}: {}", reason, msg)));
        crate::ui::ui_refresh::refresh_displays();
    }

    // Start countdown - clone Rc for first call
    let start_state = Rc::clone(&state);
    app::add_timeout3(1.0, move |_| {
        tick(&start_state);
    });

    // Fallback: force close after 3 minutes
    let mut win_fallback = win.clone();
    app::add_timeout3(180.0, move |_| {
        win_fallback.hide();
    });
}
