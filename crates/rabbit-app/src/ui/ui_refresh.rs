//! Centralized UI Refresh Manager
//!
//! Replaces per-tab `add_idle3` busy-loops with a single 100ms timeout callback.
//! This reduces CPU usage from ~100% (busy-waiting every frame) to near-idle.

use fltk::{prelude::*, text::TextDisplay, browser::Browser};
use std::cell::RefCell;
use std::collections::HashMap;

type DisplayStore = RefCell<HashMap<&'static str, TextDisplay>>;
type BrowserStore = RefCell<HashMap<&'static str, Browser>>;

/// Global storage for text display widgets.
/// SAFETY: Only accessed from the main FLTK thread.
static mut DISPLAYS: Option<DisplayStore> = None;

/// Global storage for browser widgets.
/// SAFETY: Only accessed from the main FLTK thread.
static mut BROWSERS: Option<BrowserStore> = None;

/// Register a text display widget for centralized refresh management.
pub fn register_display(key: &'static str, display: TextDisplay) {
    unsafe {
        DISPLAYS.get_or_insert_with(|| RefCell::new(HashMap::new()))
            .borrow_mut()
            .insert(key, display);
    }
}

/// Register a browser widget for centralized refresh management.
pub fn register_browser(key: &'static str, browser: Browser) {
    unsafe {
        BROWSERS.get_or_insert_with(|| RefCell::new(HashMap::new()))
            .borrow_mut()
            .insert(key, browser);
    }
}

/// Register the HTTP toggle button for centralized state sync.
pub static mut HTTP_BTN: Option<fltk::button::Button> = None;
pub static mut HTTP_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);
pub const HTTP_STOP_COLOR: u32 = 0xE57373;

pub fn register_http_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        HTTP_BTN = Some(btn);
        HTTP_ACCENT = accent;
    }
}

/// Register the HTTP browser widget for centralized item refresh.
static mut HTTP_BROWSER: Option<Browser> = None;

pub fn register_http_browser(browser: Browser) {
    unsafe {
        HTTP_BROWSER = Some(browser);
    }
}

/// Register the Ping toggle button for centralized state sync.
static mut PING_BTN: Option<fltk::button::Button> = None;
static mut PING_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);

pub fn register_ping_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        PING_BTN = Some(btn);
        PING_ACCENT = accent;
    }
}

/// Register the Scan toggle button for centralized state sync.
static mut SCAN_BTN: Option<fltk::button::Button> = None;
static mut SCAN_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);

pub fn register_scan_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        SCAN_BTN = Some(btn);
        SCAN_ACCENT = accent;
    }
}

/// Register the TFTP Server toggle button for centralized state sync.
static mut TFTPD_BTN: Option<fltk::button::Button> = None;
static mut TFTPD_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);

pub fn register_tftpd_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        TFTPD_BTN = Some(btn);
        TFTPD_ACCENT = accent;
    }
}

/// Register the TFTP Client action button for centralized state sync.
static mut TFTPC_BTN: Option<fltk::button::Button> = None;
static mut TFTPC_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);

pub fn register_tftpc_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        TFTPC_BTN = Some(btn);
        TFTPC_ACCENT = accent;
    }
}

/// Register the Plan action button for centralized state sync.
static mut PLAN_BTN: Option<fltk::button::Button> = None;
static mut PLAN_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);

pub fn register_plan_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        PLAN_BTN = Some(btn);
        PLAN_ACCENT = accent;
    }
}

/// Register the Chat action button for centralized state sync.
static mut CHAT_BTN: Option<fltk::button::Button> = None;
static mut CHAT_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);

pub fn register_chat_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        CHAT_BTN = Some(btn);
        CHAT_ACCENT = accent;
    }
}

/// Register the Settings action button for centralized state sync.
static mut SETTINGS_BTN: Option<fltk::button::Button> = None;
static mut SETTINGS_ACCENT: fltk::enums::Color = fltk::enums::Color::from_rgb(0, 0, 0);

pub fn register_settings_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    unsafe {
        SETTINGS_BTN = Some(btn);
        SETTINGS_ACCENT = accent;
    }
}

/// Trigger the main button of a specific tab by index (0=Ping, 1=Scan, 2=HTTP, 3=TFTPD, 4=TFTPC, 5=PLAN, 6=CHAT, 7=Settings)
pub fn trigger_tab_button(tab_index: usize) {
    unsafe {
        let btn = match tab_index {
            0 => PING_BTN.as_mut(),
            1 => SCAN_BTN.as_mut(),
            2 => HTTP_BTN.as_mut(),
            3 => TFTPD_BTN.as_mut(),
            4 => TFTPC_BTN.as_mut(),
            5 => PLAN_BTN.as_mut(),
            6 => CHAT_BTN.as_mut(),
            7 => SETTINGS_BTN.as_mut(),
            _ => None,
        };
        if let Some(btn) = btn {
            btn.do_callback();
        }
    }
}
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
        check!("scan_running");
        check!("http_log");
        // http_running handled separately in Phase 1 end
        check!("http_items");
        check!("tftpd_log");
        check!("tftpd_dirs");
        check!("tftpc_log");
        check!("plan_list");
        check!("chat_messages");
        check!("chat_users");
        check!("settings_output");

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
                "settings_output" => s.settings_output.clone(),
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

        let scan_updated = s.is_updated("scan_running");
        if scan_updated { s.clear_updated("scan_running"); }
        let scan_running = s.scan_running;

        let ping_updated = s.is_updated("ping_running");
        if ping_updated { s.clear_updated("ping_running"); }
        let ping_running = s.ping_running;

        drop(s);
        let http_state = if http_updated { Some(http_running) } else { None };
        (data, if ping_updated { Some(ping_running) } else { None }, http_state, if scan_updated { Some(scan_running) } else { None })
    };

    // Phase 2: Update displays without holding the lock
    let Some(store) = (unsafe { DISPLAYS.as_ref() }) else { return };
    let mut map = store.borrow_mut();

    for (key, value) in &snapshot.0 {
        if *key == "http_running" { continue; }
        if *key == "tftpd_dirs" { continue; } // Handle browsers separately
        let Some(display) = map.get_mut(key) else { continue };
        if let Some(mut buf) = display.buffer() {
            buf.set_text(value);
            // Auto-scroll for all log-type displays when new content arrives
            if matches!(*key, "ping_output" | "scan_output" | "http_log"
                              | "tftpd_log" | "tftpc_log" | "plan_list"
                              | "chat_messages" | "settings_output") {
                let lines = buf.count_lines(0, buf.length());
                display.scroll(lines, 0);
            }
        }
    }
    drop(map);

    // Update browsers (tftpd_dirs and http_items)
    if let Some(browser_store) = unsafe { BROWSERS.as_ref() } {
        let mut browser_map = browser_store.borrow_mut();
        for (key, value) in &snapshot.0 {
            if *key == "tftpd_dirs" {
                if let Some(browser) = browser_map.get_mut("tftpd_dirs") {
                    browser.clear();
                    for line in value.lines() {
                        if !line.is_empty() {
                            browser.add(line);
                        }
                    }
                    if browser.size() == 0 {
                        browser.add("(no directories added)");
                    }
                }
            }
        }
    }

    // Update HTTP browser with items and selection
    if let Some(state) = crate::ui_state::UiState::global() {
        if let Ok(s) = state.lock() {
            if let Some(mut browser) = unsafe { HTTP_BROWSER.as_mut() } {
                browser.clear();
                let selected_idx = s.http_selected_idx.unwrap_or(0);
                for (i, item) in s.http_items.iter().enumerate() {
                    let line_num = (i + 1) as i32;
                    // Add visual indicator for selected item
                    if line_num == selected_idx {
                        browser.add(&format!("▶ {}", item));
                    } else {
                        browser.add(item);
                    }
                }
                if s.http_items.is_empty() {
                    browser.add("(No files or directories configured)");
                } else {
                    // Restore selection
                    if selected_idx > 0 {
                        browser.select(selected_idx);
                    }
                }
            }
        }
    }

    // Phase 3: Sync button states (only when explicitly updated)
    // Ping button
    if let Some(running) = snapshot.1 {
        if let Some(btn) = unsafe { PING_BTN.as_mut() } {
            if running {
                btn.set_label("Stop");
                btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
            } else {
                btn.set_label("Start");
                btn.set_color(unsafe { PING_ACCENT });
            }
            btn.redraw();
        }
    }
    // HTTP button
    if let Some(running) = snapshot.2 {
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
    // Scan button
    if let Some(running) = snapshot.3 {
        if let Some(btn) = unsafe { SCAN_BTN.as_mut() } {
            if running {
                btn.set_label("Stop");
                btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
            } else {
                btn.set_label("Start");
                btn.set_color(unsafe { SCAN_ACCENT });
            }
            btn.redraw();
        }
    }
}
