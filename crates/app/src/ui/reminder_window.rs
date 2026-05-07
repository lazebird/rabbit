//! Fullscreen Black Reminder Window
//!
//! Displays a fullscreen black window with the plan task message.
//! Double-click to close, auto-closes after 3 minutes.

use fltk::{app, enums::*, frame::Frame, prelude::*, window::Window};

/// Show a fullscreen black reminder with the given message
/// Double-click to close, auto-closes after 3 minutes (180 seconds)
pub fn show_reminder(message: &str) {
    let (sw, sh) = app::screen_size();
    let mut win = Window::new(0, 0, sw as i32, sh as i32, "Reminder");
    win.set_color(Color::Black);
    win.set_border(false);
    win.fullscreen(true);

    let mut frame = Frame::new(0, 0, sw as i32, sh as i32, "");
    frame.set_color(Color::Black);
    frame.set_label_color(Color::White);
    frame.set_label_font(Font::HelveticaBold);
    frame.set_label_size(48);
    frame.set_label(message);

    win.end();
    win.show();

    // Double-click to close
    win.handle(move |w, ev| match ev {
        Event::Push => {
            if app::event_clicks() {
                w.hide();
                true
            } else {
                false
            }
        }
        _ => false,
    });

    // Auto-close after 3 minutes (180 seconds)
    let mut win_clone = win.clone();
    app::add_timeout3(180.0, move |_| {
        win_clone.hide();
    });
}
