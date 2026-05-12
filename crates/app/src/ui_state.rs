//! Global UI State for sharing between services and UI
//!
//! This module provides thread-safe access to UI text buffers
//! so services can update the display without direct FLTK access.

use chrono::Local;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};

/// Trim a string to at most `max_lines` lines, removing oldest lines from the front.
fn trim_lines(s: &mut String, max_lines: usize) {
    if s.is_empty() {
        return;
    }
    let count = s.lines().count();
    if count > max_lines {
        let to_remove = count - max_lines;
        let mut removed = 0usize;
        let mut pos = 0usize;
        for ch in s.chars() {
            if removed >= to_remove {
                break;
            }
            if ch == '\n' {
                removed += 1;
            }
            pos += ch.len_utf8();
        }
        *s = s[pos..].to_string();
    }
}

/// Global main window for updating title
static MAIN_WINDOW: Mutex<Option<fltk::window::Window>> = Mutex::new(None);

/// Global UI state singleton
static GLOBAL_UI_STATE: Mutex<Option<Arc<StdMutex<UiState>>>> = Mutex::new(None);

/// UI State containing all text buffers
#[derive(Debug, Default)]
pub struct UiState {
    pub ping_output: String,
    pub ping_stats: String,
    pub scan_output: String,
    pub http_log: String,
    pub http_items: Vec<String>,
    pub http_selected_idx: Option<i32>,
    pub tftpd_log: String,
    pub tftpd_dirs: Vec<String>,
    pub tftpd_selected_idx: Option<i32>,
    pub tftpc_log: String,
    pub chat_messages: String,
    pub chat_users: String,
    pub settings_output: String,
    pub plan_output: String,
    pub updated: HashMap<String, bool>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            ping_output: String::new(),
            ping_stats: String::from("Ready"),
            scan_output: String::new(),
            http_log: String::new(),
            http_items: Vec::new(),
            http_selected_idx: None,
            tftpd_log: String::new(),
            tftpd_dirs: Vec::new(),
            tftpd_selected_idx: None,
            tftpc_log: String::new(),
            chat_messages: String::new(),
            chat_users: String::new(),
            settings_output: String::new(),
            plan_output: String::new(),
            updated: HashMap::new(),
        }
    }

    pub fn set_main_window(window: fltk::window::Window) {
        *MAIN_WINDOW.lock() = Some(window);
    }

    pub fn get_main_window() -> Option<fltk::window::Window> {
        MAIN_WINDOW.lock().clone()
    }

    pub fn init() -> Arc<StdMutex<UiState>> {
        let state = Arc::new(StdMutex::new(UiState::new()));
        *GLOBAL_UI_STATE.lock() = Some(state.clone());
        state
    }

    pub fn global() -> Option<Arc<StdMutex<UiState>>> {
        GLOBAL_UI_STATE.lock().clone()
    }

    pub fn set_ping_stats(&mut self, stats: &str) {
        self.ping_stats = stats.to_string();
        self.updated.insert("ping_stats".to_string(), true);
    }

    pub fn append_chat(&mut self, sender: &str, message: &str) {
        self.chat_messages.push('[');
        self.chat_messages.push_str(sender);
        self.chat_messages.push_str("] ");
        self.chat_messages.push_str(message);
        self.chat_messages.push('\n');
        trim_lines(&mut self.chat_messages, 1000);
        self.updated.insert("chat_messages".to_string(), true);
    }

    /// Write content to a named output field, appending '\n' and trimming.
    pub fn write_to_field(&mut self, target: &str, content: &str) {
        let field: &mut String = match target {
            "ping_output" => &mut self.ping_output,
            "scan_output" => &mut self.scan_output,
            "http_log" => &mut self.http_log,
            "tftpd_log" => &mut self.tftpd_log,
            "tftpc_log" => &mut self.tftpc_log,
            "chat_messages" => &mut self.chat_messages,
            "settings_output" => &mut self.settings_output,
            "plan_output" => &mut self.plan_output,
            _ => return,
        };
        field.push_str(content);
        field.push('\n');
        trim_lines(field, 1000);
        self.updated.insert(target.to_string(), true);
    }

    pub fn is_updated(&self, key: &str) -> bool {
        self.updated.get(key).copied().unwrap_or(false)
    }

    pub fn clear_updated(&mut self, key: &str) {
        self.updated.remove(key);
    }
}

/// Format a message with a [HH:MM:SS] timestamp prefix.
pub fn fmt_log(msg: &str) -> String {
    format!("[{}] {}", Local::now().format("%H:%M:%S"), msg)
}

/// Pass-through for raw (unformatted) messages.
pub fn raw_log(msg: &str) -> String {
    msg.to_string()
}

/// Write a pre-formatted message to a named output field.
/// This is an internal helper; external code should use fmt_log/raw_log
/// to prepare the message before calling this.
pub fn write_to(target: &str, msg: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.write_to_field(target, msg);
        }
    }
}

/// Helper functions for services to update UI
pub fn set_ping_stats(stats: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.set_ping_stats(stats);
        }
    }
}

/// 通用模块状态更新：同时更新UI状态和配置
pub fn update_module_running(module: &str, running: bool) {
    // 更新UI状态
    match module {
        "ping" => set_ping_running(running),
        "scan" => set_scan_running(running),
        "http" => set_http_running(running),
        "tftpd" => set_tftpd_running(running),
        "chat" => set_chat_running(running),
        _ => {}
    }
    // 更新配置
    rabbit_config::update_config(|cfg| {
        cfg.modules.insert(module, "running", schema::config::ConfigValue::Boolean(running));
    })
    .ok();
}

pub fn set_scan_running(running: bool) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.updated.insert("scan_running".to_string(), running);
        }
    }
}

pub fn set_ping_running(running: bool) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.updated.insert("ping_running".to_string(), running);
        }
    }
}

pub fn set_http_running(running: bool) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.updated.insert("http_running".to_string(), running);
        }
    }
}

pub fn set_tftpd_running(running: bool) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.updated.insert("tftpd_running".to_string(), running);
        }
    }
}

pub fn set_chat_running(running: bool) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.updated.insert("chat_running".to_string(), running);
        }
    }
}

pub fn add_tftpd_dir(path: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            if !s.tftpd_dirs.contains(&path.to_string()) {
                s.tftpd_dirs.push(path.to_string());
                s.updated.insert("tftpd_dirs".to_string(), true);
            }
        }
    }
}

pub fn remove_tftpd_dir(path: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.tftpd_dirs.retain(|d| d != path);
            s.updated.insert("tftpd_dirs".to_string(), true);
        }
    }
}

pub fn get_tftpd_selected_dir() -> Option<String> {
    if let Some(state) = UiState::global() {
        if let Ok(s) = state.lock() {
            if let Some(idx) = s.tftpd_selected_idx {
                if idx > 0 && (idx as usize) <= s.tftpd_dirs.len() {
                    return Some(s.tftpd_dirs[(idx - 1) as usize].clone());
                }
            }
        }
    }
    None
}

pub fn set_tftpd_selected_idx(idx: i32) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.tftpd_selected_idx = if idx > 0 { Some(idx) } else { None };
        }
    }
}

// HTTP file/directory management
pub fn add_http_item(path: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            if !s.http_items.contains(&path.to_string()) {
                s.http_items.push(path.to_string());
                s.http_selected_idx = Some((s.http_items.len()) as i32);
                s.updated.insert("http_items".to_string(), true);
            }
        }
    }
}

pub fn remove_http_item(path: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            let old_len = s.http_items.len();
            s.http_items.retain(|d| d != path);
            if s.http_items.len() < old_len {
                // Update selection after removal
                if let Some(sel) = s.http_selected_idx {
                    let sel_usize = sel as usize;
                    if sel_usize > s.http_items.len() {
                        s.http_selected_idx = if s.http_items.is_empty() { None } else { Some(s.http_items.len() as i32) };
                    }
                }
                s.updated.insert("http_items".to_string(), true);
            }
        }
    }
}

pub fn set_http_selected(idx: i32) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.http_selected_idx = if idx <= 0 { None } else { Some(idx) };
        }
    }
}

/// Sync HTTP items from UI state to config and save
pub fn sync_http_config() {
    use schema::config::ConfigValue;
    use rabbit_config::{load_config, save_config};

    if let Some(state) = UiState::global() {
        if let Ok(s) = state.lock() {
            if let Ok(mut config) = load_config() {
                config
                    .modules
                    .http
                    .insert("dirs".into(), ConfigValue::Array(s.http_items.iter().map(|item| ConfigValue::String(item.clone())).collect()));
                if let Err(e) = save_config(&config) {
                    tracing::warn!("Failed to save HTTP config: {}", e);
                }
            }
        }
    }
}

/// Sync TFTP directory from UI state to config and save
pub fn sync_tftpd_config() {
    use schema::config::ConfigValue;
    use rabbit_config::{load_config, save_config};

    if let Some(state) = UiState::global() {
        if let Ok(s) = state.lock() {
            if let Ok(mut config) = load_config() {
                config
                    .modules
                    .tftpd
                    .insert("work_dirs".into(), ConfigValue::Array(s.tftpd_dirs.iter().map(|item| ConfigValue::String(item.clone())).collect()));
                if let Some(idx) = s.tftpd_selected_idx {
                    let config_idx = if idx <= 0 { 0 } else { (idx - 1) as i64 };
                    config.modules.tftpd.insert("working_dir_index".into(), ConfigValue::Integer(config_idx));
                }
                if let Err(e) = save_config(&config) {
                    tracing::warn!("Failed to save TFTP config: {}", e);
                }
            }
        }
    }
}

/// Set TFTP working directory by index (1-based browser index)
pub fn set_tftpd_selected(idx: i32) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.tftpd_selected_idx = if idx <= 0 { None } else { Some(idx) };
        }
    }
}

// plan_list 已移除，改用事件驱动模式

pub fn set_chat_users(users: &[String]) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            if users.is_empty() {
                s.chat_users = "Online Users:\n───────────\n(no users)\n".to_string();
            } else {
                let user_list = users.join("\n");
                s.chat_users = format!("Online Users:\n───────────\n{}\n", user_list);
            }
            s.updated.insert("chat_users".to_string(), true);
        }
    }
}

pub fn add_chat_user(user: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            if s.chat_users.contains("(no users)") {
                s.chat_users = "Online Users:\n───────────\n".to_string();
            }
            if !s.chat_users.contains(&format!("\n{}\n", user)) && !s.chat_users.ends_with(&user.to_string()) {
                s.chat_users.push_str(user);
                s.chat_users.push('\n');
                s.updated.insert("chat_users".to_string(), true);
            }
        }
    }
}

pub fn remove_chat_user(user: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.chat_users = s.chat_users.replace(&format!("{}\n", user), "");
            s.chat_users = s.chat_users.replace(&user.to_string(), "");
            if s.chat_users.trim() == "Online Users:\n───────────" {
                s.chat_users = "Online Users:\n───────────\n(no users)\n".to_string();
            }
            s.updated.insert("chat_users".to_string(), true);
        }
    }
}

// ============================================================
// Settings Output
// ============================================================

/// Update a single line in settings output (replaces instead of appending)
/// Useful for progress bars that need to refresh in place
pub fn update_settings_line(line_index: i32, text: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            let mut lines: Vec<&str> = s.settings_output.lines().collect();
            let len = lines.len();

            if line_index >= 0 && (line_index as usize) < len {
                lines[line_index as usize] = text;
            } else if line_index == -1 {
                // -1 means update the last line
                if len > 0 {
                    lines[len - 1] = text;
                }
            } else {
                // Line index out of bounds, append
                lines.push(text);
            }

            s.settings_output = lines.join("\n");
            s.updated.insert("settings_output".to_string(), true);
        }
    }
}

// Settings configuration updates
// These update the UI state and trigger config save via the SettingsSave event

pub fn set_systray(value: bool) {
    use schema::config::{self, ConfigValue};
    use rabbit_config::{load_config, save_config};

    if let Ok(mut config) = load_config() {
        config.modules.insert("global", config::keys::global::SYSTRAY, ConfigValue::Boolean(value));
        save_config(&config).ok();
    }
}

pub fn set_top(value: bool) {
    use schema::config::{self, ConfigValue};
    use rabbit_config::{load_config, save_config};

    if let Ok(mut config) = load_config() {
        config.modules.insert("global", config::keys::global::TOP, ConfigValue::Boolean(value));
        save_config(&config).ok();
    }
}

pub fn set_autostart(value: bool) {
    use schema::config::{self, ConfigValue};
    use rabbit_config::{load_config, save_config};

    if let Ok(mut config) = load_config() {
        config.modules.insert("global", config::keys::global::AUTOSTART, ConfigValue::Boolean(value));
        save_config(&config).ok();
    }
}

pub fn set_autoupdate(value: bool) {
    use schema::config::{self, ConfigValue};
    use rabbit_config::{load_config, save_config};

    if let Ok(mut config) = load_config() {
        config.modules.insert("global", config::keys::global::AUTOUPDATE, ConfigValue::Boolean(value));
        save_config(&config).ok();
    }
}

pub fn set_language(value: &str) {
    use schema::config::{self, ConfigValue};
    use rabbit_config::{load_config, save_config};

    if let Ok(mut config) = load_config() {
        config.modules.insert("global", config::keys::global::LANGUAGE, ConfigValue::String(value.to_string()));
        save_config(&config).ok();
    }
}

/// Save plan tasks to config (structured storage)
pub fn save_plan_tasks(tasks: &[schema::config::PlanTask]) {
    use schema::config::{self, ConfigValue};
    use rabbit_config::{load_config, save_config};

    if let Ok(mut config) = load_config() {
        let json = serde_json::to_string(tasks).unwrap_or_default();
        config.modules.insert("plan", config::keys::plan::TASKS, ConfigValue::String(json));
        if let Err(e) = save_config(&config) {
            tracing::warn!("Failed to save plan tasks: {}", e);
        }
    }
}

/// Load plan tasks from config (structured storage)
pub fn load_plan_tasks() -> Vec<schema::config::PlanTask> {
    use schema::config::PlanTask;
    use rabbit_config::load_config;

    if let Ok(config) = load_config() {
        if let Some(json_str) = config.modules.get_string("plan", schema::config::keys::plan::TASKS) {
            if let Ok(tasks) = serde_json::from_str::<Vec<PlanTask>>(&json_str) {
                return tasks;
            }
        }
    }
    Vec::new()
}

/// Sync scan configuration when starting a scan
pub fn sync_scan_config(start_ip: String, end_ip: String, filter: bool) {
    use schema::config::{self, ConfigValue};
    use rabbit_config::{load_config, save_config};

    if let Ok(mut config) = load_config() {
        config.modules.scan.insert(config::keys::scan::START_IP.into(), ConfigValue::String(start_ip));

        // Save only single number (last octet) for end_ip
        let end_suffix: u8 = end_ip.parse().unwrap_or(254);
        config.modules.scan.insert(config::keys::scan::END_IP.into(), ConfigValue::String(end_suffix.to_string()));

        config.modules.scan.insert(config::keys::scan::FILTER.into(), ConfigValue::Boolean(filter));
        if let Err(e) = save_config(&config) {
            tracing::warn!("Failed to save scan config: {}", e);
        }
    }
}

/// Sync HTTP configuration when starting the server
pub fn sync_http_start_config(port: u16, shell: bool, autoindex: bool, videoplay: bool) {
    use schema::config::{self, ConfigValue};
    use rabbit_config::{load_config, save_config};

    tracing::info!("sync_http_start_config: port={}, shell={}, autoindex={}, videoplay={}", port, shell, autoindex, videoplay);

    if let Ok(mut config) = load_config() {
        let old_port = config.modules.get_integer("http", config::keys::http::PORT);
        tracing::info!("sync_http_start_config: old port from cache = {:?}", old_port);

        config.modules.http.insert(config::keys::http::PORT.into(), ConfigValue::Integer(port as i64));
        config.modules.http.insert(config::keys::http::SHELL.into(), ConfigValue::Boolean(shell));
        config.modules.http.insert(config::keys::http::AUTO_INDEX.into(), ConfigValue::Boolean(autoindex));
        config.modules.http.insert(config::keys::http::VIDEO_PLAY.into(), ConfigValue::Boolean(videoplay));
        match save_config(&config) {
            Ok(()) => tracing::info!("sync_http_start_config: config saved successfully"),
            Err(e) => tracing::warn!("sync_http_start_config: Failed to save HTTP config: {}", e),
        }
    } else {
        tracing::warn!("sync_http_start_config: load_config failed");
    }
}
