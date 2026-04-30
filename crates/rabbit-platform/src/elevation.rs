//! Privilege elevation using the `xelevate` crate (v0.1.0)
//!
//! Cross-platform elevation with fltk error dialogs.

pub fn ensure_elevated() {
    // Debug/Android/iOS: skip elevation
    #[cfg(any(debug_assertions, target_os = "android", target_os = "ios"))]
    {
        // skip
    }

    #[cfg(not(any(debug_assertions, target_os = "android", target_os = "ios")))]
    {
        use xelevate::{elevate, is_elevated};

        // Check if already elevated via env var or API
        if std::env::var("XELEVATE_ELEVATED").is_ok() || is_elevated() {
            return;
        }

        // Get current exe path
        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from(std::env::args().next().unwrap_or_default()))
            .to_string_lossy()
            .into_owned();

        // Attempt elevation
        match elevate(&exe_path) {
            Ok(()) => {
                // New elevated process started, exit current
                std::process::exit(0);
            }
            Err(e) => {
                show_elevation_error(&format!("{}", e));
                std::process::exit(1);
            }
        }
    }
}

#[cfg(not(any(debug_assertions, target_os = "android", target_os = "ios")))]
fn show_elevation_error(msg: &str) {
    use fltk::prelude::*;

    // Use fltk for cross-platform error dialog
    let app = fltk::app::App::default();
    let mut wind = fltk::window::Window::default().with_size(400, 180).with_label("Elevation Failed");
    wind.make_modal(true);

    let mut msg_box = fltk::text::TextDisplay::default().with_pos(20, 20).with_size(360, 100);
    msg_box.set_buffer(Some(fltk::text::TextBuffer::default()));
    msg_box.buffer().unwrap().set_text(msg);

    let mut btn = fltk::button::Button::default().with_pos(160, 140).with_size(80, 30).with_label("OK");
    btn.set_callback({
        let mut w = wind.clone();
        move |_| w.hide()
    });

    wind.end();
    wind.show();
    app.run().unwrap();
}
