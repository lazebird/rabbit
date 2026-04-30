//! Centralized UI Refresh Manager
//!
//! Replaces per-tab `add_idle3` busy-loops with a single 100ms timeout callback.
//! This reduces CPU usage from ~100% (busy-waiting every frame) to near-idle.

use fltk::{browser::Browser, prelude::*, text::TextDisplay};
use parking_lot::Mutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

type DisplayStore = RefCell<HashMap<&'static str, TextDisplay>>;
type BrowserStore = RefCell<HashMap<&'static str, Browser>>;

/// Global storage for text display widgets.
static DISPLAYS: Mutex<Option<DisplayStore>> = Mutex::new(None);

/// Global storage for browser widgets.
static BROWSERS: Mutex<Option<BrowserStore>> = Mutex::new(None);

/// Register a text display widget for centralized refresh management.
pub fn register_display(key: &'static str, display: TextDisplay) {
    DISPLAYS.lock().get_or_insert_with(|| RefCell::new(HashMap::new())).borrow_mut().insert(key, display);
}

/// Register a browser widget for centralized refresh management.
pub fn register_browser(key: &'static str, browser: Browser) {
    BROWSERS.lock().get_or_insert_with(|| RefCell::new(HashMap::new())).borrow_mut().insert(key, browser);
}

/// Register the HTTP toggle button for centralized state sync.
static HTTP_BTN: Mutex<Option<fltk::button::Button>> = Mutex::new(None);
static HTTP_ACCENT: Mutex<fltk::enums::Color> = Mutex::new(fltk::enums::Color::from_rgb(0, 0, 0));
pub const HTTP_STOP_COLOR: u32 = 0xE57373;

pub fn register_http_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    *HTTP_BTN.lock() = Some(btn);
    *HTTP_ACCENT.lock() = accent;
}

pub fn get_http_btn() -> Option<fltk::button::Button> {
    HTTP_BTN.lock().clone()
}

/// Register the HTTP browser widget for centralized item refresh.
static HTTP_BROWSER: Mutex<Option<Browser>> = Mutex::new(None);

pub fn register_http_browser(browser: Browser) {
    *HTTP_BROWSER.lock() = Some(browser);
}

/// Register the Ping toggle button for centralized state sync.
static PING_BTN: Mutex<Option<fltk::button::Button>> = Mutex::new(None);
static PING_ACCENT: Mutex<fltk::enums::Color> = Mutex::new(fltk::enums::Color::from_rgb(0, 0, 0));

pub fn register_ping_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    *PING_BTN.lock() = Some(btn);
    *PING_ACCENT.lock() = accent;
}

pub fn get_ping_btn() -> Option<fltk::button::Button> {
    PING_BTN.lock().clone()
}

/// Register the Scan toggle button for centralized state sync.
static SCAN_BTN: Mutex<Option<fltk::button::Button>> = Mutex::new(None);
static SCAN_ACCENT: Mutex<fltk::enums::Color> = Mutex::new(fltk::enums::Color::from_rgb(0, 0, 0));

pub fn register_scan_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    *SCAN_BTN.lock() = Some(btn);
    *SCAN_ACCENT.lock() = accent;
}

pub fn get_scan_btn() -> Option<fltk::button::Button> {
    SCAN_BTN.lock().clone()
}

/// Register the TFTP Server toggle button for centralized state sync.
static TFTPD_BTN: Mutex<Option<fltk::button::Button>> = Mutex::new(None);
static TFTPD_ACCENT: Mutex<fltk::enums::Color> = Mutex::new(fltk::enums::Color::from_rgb(0, 0, 0));

pub fn register_tftpd_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    *TFTPD_BTN.lock() = Some(btn);
    *TFTPD_ACCENT.lock() = accent;
}

pub fn get_tftpd_btn() -> Option<fltk::button::Button> {
    TFTPD_BTN.lock().clone()
}

/// Register the TFTP Client action button for centralized state sync.
static TFTPC_BTN: Mutex<Option<fltk::button::Button>> = Mutex::new(None);
static TFTPC_ACCENT: Mutex<fltk::enums::Color> = Mutex::new(fltk::enums::Color::from_rgb(0, 0, 0));

pub fn register_tftpc_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    *TFTPC_BTN.lock() = Some(btn);
    *TFTPC_ACCENT.lock() = accent;
}

pub fn get_tftpc_btn() -> Option<fltk::button::Button> {
    TFTPC_BTN.lock().clone()
}

/// Register the Plan action button for centralized state sync.
static PLAN_BTN: Mutex<Option<fltk::button::Button>> = Mutex::new(None);
static PLAN_ACCENT: Mutex<fltk::enums::Color> = Mutex::new(fltk::enums::Color::from_rgb(0, 0, 0));

pub fn register_plan_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    *PLAN_BTN.lock() = Some(btn);
    *PLAN_ACCENT.lock() = accent;
}

pub fn get_plan_btn() -> Option<fltk::button::Button> {
    PLAN_BTN.lock().clone()
}

/// Register the Chat action button for centralized state sync.
static CHAT_BTN: Mutex<Option<fltk::button::Button>> = Mutex::new(None);
static CHAT_ACCENT: Mutex<fltk::enums::Color> = Mutex::new(fltk::enums::Color::from_rgb(0, 0, 0));

pub fn register_chat_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    *CHAT_BTN.lock() = Some(btn);
    *CHAT_ACCENT.lock() = accent;
}

pub fn get_chat_btn() -> Option<fltk::button::Button> {
    CHAT_BTN.lock().clone()
}

/// Register the Settings action button for centralized state sync.
static SETTINGS_BTN: Mutex<Option<fltk::button::Button>> = Mutex::new(None);
static SETTINGS_ACCENT: Mutex<fltk::enums::Color> = Mutex::new(fltk::enums::Color::from_rgb(0, 0, 0));

pub fn register_settings_button(btn: fltk::button::Button, accent: fltk::enums::Color) {
    *SETTINGS_BTN.lock() = Some(btn);
    *SETTINGS_ACCENT.lock() = accent;
}

pub fn get_settings_btn() -> Option<fltk::button::Button> {
    SETTINGS_BTN.lock().clone()
}

/// Trigger the main button of a specific tab by index (0=Ping, 1=Scan, 2=HTTP, 3=TFTPD, 4=TFTPC, 5=PLAN, 6=CHAT, 7=Settings)
pub fn trigger_tab_button(tab_index: usize) {
    let btn = match tab_index {
        0 => PING_BTN.lock().clone(),
        1 => SCAN_BTN.lock().clone(),
        2 => HTTP_BTN.lock().clone(),
        3 => TFTPD_BTN.lock().clone(),
        4 => TFTPC_BTN.lock().clone(),
        5 => PLAN_BTN.lock().clone(),
        6 => CHAT_BTN.lock().clone(),
        7 => SETTINGS_BTN.lock().clone(),
        _ => None,
    };
    if let Some(mut btn) = btn {
        btn.do_callback();
    }
}

static REFRESH_RUNNING: AtomicBool = AtomicBool::new(true);

/// Start the centralized refresh loop. Fires every 100ms (10Hz).
pub fn start_refresh_loop() {
    fn tick() {
        if !REFRESH_RUNNING.load(Ordering::SeqCst) {
            return;
        }
        do_refresh();
        fltk::app::repeat_timeout2(0.1, tick);
    }
    tick();
}

/// Stop the refresh loop. Call before exiting.
pub fn stop_refresh_loop() {
    REFRESH_RUNNING.store(false, Ordering::SeqCst);
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
        check!("http_items");
        check!("tftpd_log");
        check!("tftpd_dirs");
        check!("tftpc_log");
        check!("plan_list");
        check!("chat_messages");
        check!("chat_users");
        check!("settings_output");
        check!("ping_running");
        check!("scan_running");
        check!("http_running");

        if keys.is_empty() {
            return;
        }

        let data: Vec<(&str, String)> = keys
            .iter()
            .map(|&k| {
                let v = match k {
                    "ping_output" => s.ping_output.clone(),
                    "ping_stats" => s.ping_stats.clone(),
                    "scan_output" => s.scan_output.clone(),
                    "http_log" => s.http_log.clone(),
                    "tftpd_log" => s.tftpd_log.clone(),
                    "tftpc_log" => s.tftpc_log.clone(),
                    "plan_list" => s.plan_list.clone(),
                    "chat_messages" => s.chat_messages.clone(),
                    "chat_users" => s.chat_users.clone(),
                    "settings_output" => s.settings_output.clone(),
                    "tftpd_dirs" => {
                        if s.tftpd_dirs.is_empty() {
                            "(no directories added)\n".to_string()
                        } else {
                            s.tftpd_dirs.join("\n") + "\n"
                        }
                    }
                    "ping_running" | "scan_running" | "http_running" => {
                        let val = s.updated.get(k).copied().unwrap_or(false);
                        val.to_string()
                    }
                    _ => String::new(),
                };
                (k, v)
            })
            .collect();

        let ping_updated = keys.contains(&"ping_running");
        let http_updated = keys.contains(&"http_running");
        let scan_updated = keys.contains(&"scan_running");

        // 使用 updated 标志作为运行状态
        let ping_running = ping_updated;
        let http_running = http_updated;
        let scan_running = scan_updated;

        drop(s);
        (
            data,
            if ping_updated { Some(ping_running) } else { None },
            if http_updated { Some(http_running) } else { None },
            if scan_updated { Some(scan_running) } else { None },
        )
    };

    // Phase 2: Update displays without holding the lock
    let displays_guard = DISPLAYS.lock();
    let Some(store) = displays_guard.as_ref() else { return };
    let mut map = store.borrow_mut();

    for (key, value) in &snapshot.0 {
        if *key == "http_running" {
            continue;
        }
        if *key == "tftpd_dirs" {
            continue;
        } // Handle browsers separately
        let Some(display) = map.get_mut(key) else { continue };
        if let Some(mut buf) = display.buffer() {
            buf.set_text(value);
            // Auto-scroll for all log-type displays when new content arrives
            if matches!(
                *key,
                "ping_output" | "scan_output" | "http_log" | "tftpd_log" | "tftpc_log" | "plan_list" | "chat_messages" | "settings_output"
            ) {
                let lines = buf.count_lines(0, buf.length());
                display.scroll(lines, 0);
            }
        }
    }
    drop(map);

    // Update browsers (tftpd_dirs and http_items)
    if let Some(browser_store) = BROWSERS.lock().as_ref() {
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
            if let Some(browser) = HTTP_BROWSER.lock().as_mut() {
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
        if let Some(btn) = PING_BTN.lock().as_mut() {
            if running {
                btn.set_label("Stop");
                btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
            } else {
                btn.set_label("Start");
                btn.set_color(*PING_ACCENT.lock());
            }
            btn.redraw();
        }
    }
    // HTTP button
    if let Some(running) = snapshot.2 {
        if let Some(btn) = HTTP_BTN.lock().as_mut() {
            if running {
                btn.set_label("Stop");
                btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
            } else {
                btn.set_label("Start");
                btn.set_color(*HTTP_ACCENT.lock());
            }
            btn.redraw();
        }
    }
    // Scan button
    if let Some(running) = snapshot.3 {
        if let Some(btn) = SCAN_BTN.lock().as_mut() {
            if running {
                btn.set_label("Stop");
                btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
            } else {
                btn.set_label("Start");
                btn.set_color(*SCAN_ACCENT.lock());
            }
            btn.redraw();
        }
    }
}
