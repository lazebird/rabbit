//! Centralized UI Refresh Manager
//!
//! Replaces per-tab `add_idle3` busy-loops with a single 100ms timeout callback.
//! This reduces CPU usage from ~100% (busy-waiting every frame) to near-idle.

use fltk::{prelude::*, text::TextDisplay};
use std::cell::RefCell;
use std::collections::HashMap;

type DisplayStore = RefCell<HashMap<&'static str, TextDisplay>>;

/// Global storage for text display widgets.
/// SAFETY: Only accessed from the main FLTK thread.
static mut DISPLAYS: Option<DisplayStore> = None;

/// Register a text display widget for centralized refresh management.
pub fn register_display(key: &'static str, display: TextDisplay) {
    unsafe {
        DISPLAYS.get_or_insert_with(|| RefCell::new(HashMap::new()))
            .borrow_mut()
            .insert(key, display);
    }
}

/// Register the HTTP toggle button for centralized state sync.
static mut HTTP_BTN: Option<fltk::button::Button> = None;
static mut HTTP_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);
const HTTP_STOP_COLOR: u32 = 0xE57373;

pub fn register_http_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        HTTP_BTN = Some(btn);
        HTTP_ACCENT = accent;
    }
}

/// Global flag to signal the refresh loop to stop.
static mut REFRESH_RUNNING: bool = true;

/// Start the centralized refresh loop. Fires every 100ms (10Hz).
pub fn start_refresh_loop() {
    fn tick() {
        unsafe {
            if !REFRESH_RUNNING {
                return; // Don't reschedule if stopped
            }
        }
        do_refresh();
        fltk::app::repeat_timeout2(0.1, tick);
    }
    tick();
}

/// Stop the refresh loop. Call before exiting.
pub fn stop_refresh_loop() {
    unsafe {
        REFRESH_RUNNING = false;
    }
}

fn do_refresh() {
    use crate::ui_state::UiState;

    // Phase 1: Collect updated keys + snapshot data while holding the lock
    let snapshot = {
        let Some(state) = UiState::global() else { return };
        let Ok(mut s) = state.lock() else { return };

        let mut keys: Vec<&str> = Vec::new();
        macro_rules! check {
            ($k:expr) => {
                if s.is_updated($k) {
                    keys.push($k);
                    s.clear_updated($k);
                }
            };
        }
        check!("ping_output");
        check!("ping_stats");
        check!("scan_output");
        check!("http_log");
        check!("http_running");
        check!("tftpd_log");
        check!("tftpd_dirs");
        check!("tftpc_log");
        check!("plan_list");
        check!("chat_messages");
        check!("chat_users");

        if keys.is_empty() {
            return;
        }

        let data: Vec<(&str, String)> = keys.iter().map(|&k| {
            let v = match k {
                "ping_output"   => s.ping_output.clone(),
                "ping_stats"    => s.ping_stats.clone(),
                "scan_output"   => s.scan_output.clone(),
                "http_log"      => s.http_log.clone(),
                "tftpd_log"     => s.tftpd_log.clone(),
                "tftpc_log"     => s.tftpc_log.clone(),
                "plan_list"     => s.plan_list.clone(),
                "chat_messages" => s.chat_messages.clone(),
                "chat_users"    => s.chat_users.clone(),
                "tftpd_dirs"    => {
                    if s.tftpd_dirs.is_empty() {
                        "(no directories added)\n".to_string()
                    } else {
                        s.tftpd_dirs.join("\n") + "\n"
                    }
                }
                _ => String::new(),
            };
            (k, v)
        }).collect();

        let http_updated = s.is_updated("http_running");
        if http_updated { s.clear_updated("http_running"); }
        let http_running = s.http_running;
        drop(s);
        (data, if http_updated { Some(http_running) } else { None })
    };

    // Phase 2: Update displays without holding the lock
    let Some(store) = (unsafe { DISPLAYS.as_ref() }) else { return };
    let mut map = store.borrow_mut();

    for (key, value) in snapshot.0 {
        if key == "http_running" { continue; }
        let Some(display) = map.get_mut(key) else { continue };
        if let Some(mut buf) = display.buffer() {
            buf.set_text(&value);
            // Auto-scroll for log-type displays
            if matches!(key, "ping_output" | "scan_output" | "http_log"
                              | "tftpd_log" | "tftpc_log" | "plan_list" | "chat_messages") {
                let lines = buf.count_lines(0, buf.length());
                display.scroll(lines, 0);
            }
        }
    }
    drop(map);

    // Phase 3: Sync HTTP button state (only when explicitly updated)
    if let Some(running) = snapshot.1 {
        if let Some(btn) = unsafe { HTTP_BTN.as_mut() } {
            if running {
                btn.set_label("Stop");
                btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
            } else {
                btn.set_label("Start");
                btn.set_color(unsafe { HTTP_ACCENT });
            }
            btn.redraw();
        }
    }
}
