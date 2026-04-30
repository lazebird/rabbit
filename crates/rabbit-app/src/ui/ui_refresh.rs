//! Centralized UI Refresh Manager
//!
//! 使用事件驱动模式：当 app.rs 收到 UiData 时，通过 fltk::app::awake_callback
//! 直接调用本模块的更新函数，完全无轮询。

use fltk::{browser::Browser, prelude::*, text::TextDisplay};
use parking_lot::Mutex;
use std::cell::RefCell;
use std::collections::HashMap;

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

/// 直接更新按钮状态（在主线程通过 awake_callback 调用）
pub fn update_button_state(module: rabbit_core::ui_channel::Module, running: bool) {
    use rabbit_core::ui_channel::Module;
    match module {
        Module::Ping => {
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
        Module::Http => {
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
        Module::Scan => {
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
        Module::Tftpd => {
            if let Some(btn) = TFTPD_BTN.lock().as_mut() {
                if running {
                    btn.set_label("Stop");
                    btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
                } else {
                    btn.set_label("Start");
                    btn.set_color(*TFTPD_ACCENT.lock());
                }
                btn.redraw();
            }
        }
        Module::Tftpc => {
            if let Some(btn) = TFTPC_BTN.lock().as_mut() {
                if running {
                    btn.set_label("Stop");
                    btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
                } else {
                    btn.set_label("Start");
                    btn.set_color(*TFTPC_ACCENT.lock());
                }
                btn.redraw();
            }
        }
        Module::Chat => {
            if let Some(btn) = CHAT_BTN.lock().as_mut() {
                if running {
                    btn.set_label("Stop");
                    btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
                } else {
                    btn.set_label("Start");
                    btn.set_color(*CHAT_ACCENT.lock());
                }
                btn.redraw();
            }
        }
        Module::Plan => {
            if let Some(btn) = PLAN_BTN.lock().as_mut() {
                if running {
                    btn.set_label("Stop");
                    btn.set_color(fltk::enums::Color::from_hex(HTTP_STOP_COLOR));
                } else {
                    btn.set_label("Start");
                    btn.set_color(*PLAN_ACCENT.lock());
                }
                btn.redraw();
            }
        }
    }
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

/// 刷新所有文本显示器和浏览器（在主线程通过 awake_callback 调用）
/// 由 app.rs 的 handle_ui_data 在处理需要更新 UI 的 UiData 时触发
pub fn refresh_displays() {
    use crate::ui_state::UiState;

    let Some(state) = UiState::global() else { return };
    let Ok(s) = state.lock() else { return };

    // 准备数据快照
    let data: Vec<(&str, String)> = [
        ("ping_output", s.ping_output.clone()),
        ("ping_stats", s.ping_stats.clone()),
        ("scan_output", s.scan_output.clone()),
        ("http_log", s.http_log.clone()),
        ("tftpd_log", s.tftpd_log.clone()),
        ("tftpc_log", s.tftpc_log.clone()),
        ("chat_messages", s.chat_messages.clone()),
        ("chat_users", s.chat_users.clone()),
        ("settings_output", s.settings_output.clone()),
    ]
    .into_iter()
    .collect();

    let tftpd_dirs = s.tftpd_dirs.clone();
    let http_items = s.http_items.clone();
    let http_selected_idx = s.http_selected_idx;
    let tftpd_selected_idx = s.tftpd_selected_idx;

    drop(s);

    // 更新文本显示器
    let displays_guard = DISPLAYS.lock();
    let Some(store) = displays_guard.as_ref() else { return };
    let mut map = store.borrow_mut();

    for (key, value) in &data {
        let Some(display) = map.get_mut(key) else { continue };
        if let Some(mut buf) = display.buffer() {
            buf.set_text(value);
            // Auto-scroll for all log-type displays when new content arrives
            if matches!(*key, "ping_output" | "scan_output" | "http_log" | "tftpd_log" | "tftpc_log" | "chat_messages" | "settings_output") {
                let lines = buf.count_lines(0, buf.length());
                display.scroll(lines, 0);
            }
        }
    }
    drop(map);

    // 更新 tftpd_dirs 浏览器
    if let Some(browser_store) = BROWSERS.lock().as_ref() {
        let mut browser_map = browser_store.borrow_mut();
        if let Some(browser) = browser_map.get_mut("tftpd_dirs") {
            browser.clear();
            if tftpd_dirs.is_empty() {
                browser.add("(no directories added)");
            } else {
                for line in &tftpd_dirs {
                    browser.add(line);
                }
            }
            if let Some(idx) = tftpd_selected_idx {
                browser.select(idx);
            }
        }
    }

    // 更新 http_items 浏览器
    if let Some(browser) = HTTP_BROWSER.lock().as_mut() {
        browser.clear();
        let selected_idx = http_selected_idx.unwrap_or(0);
        for (i, item) in http_items.iter().enumerate() {
            let line_num = (i + 1) as i32;
            if line_num == selected_idx {
                browser.add(&format!("▶ {}", item));
            } else {
                browser.add(item);
            }
        }
        if http_items.is_empty() {
            browser.add("(No files or directories configured)");
        } else if selected_idx > 0 {
            browser.select(selected_idx);
        }
    }
}
