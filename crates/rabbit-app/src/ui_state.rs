//! Global UI State for sharing between services and UI
//!
//! This module provides thread-safe access to UI text buffers
//! so services can update the display without direct FLTK access.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Trim a string to at most `max_lines` lines, removing oldest lines from the front.
fn trim_lines(s: &mut String, max_lines: usize) {
    let count = s.lines().count();
    if count > max_lines {
        let to_remove = count - max_lines;
        let mut removed = 0usize;
        let mut pos = 0usize;
        for ch in s.chars() {
            if removed >= to_remove { break; }
            if ch == '\n' { removed += 1; }
            pos += ch.len_utf8();
        }
        *s = s[pos..].to_string();
    }
}

/// Global UI state singleton
static mut GLOBAL_UI_STATE: Option<Arc<Mutex<UiState>>> = None;

/// UI State containing all text buffers
#[derive(Debug, Default)]
pub struct UiState {
    pub ping_output: String,
    pub ping_stats: String,
    pub scan_output: String,
    pub scan_running: bool,
    pub http_log: String,
    pub http_running: bool,
    pub tftpd_log: String,
    pub tftpd_dirs: Vec<String>,
    pub tftpc_log: String,
    pub plan_list: String,
    pub chat_messages: String,
    pub chat_users: String,
    // Flags to track which buffers have been updated
    pub updated: HashMap<String, bool>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            ping_output: String::new(),
            ping_stats: String::from("Ready"),
            scan_output: String::new(),
            scan_running: false,
            http_log: String::new(),
            http_running: false,
            tftpd_log: String::new(),
            tftpd_dirs: Vec::new(),
            tftpc_log: String::new(),
            plan_list: String::new(),
            chat_messages: String::new(),
            chat_users: String::new(),
            updated: HashMap::new(),
        }
    }

    pub fn init() -> Arc<Mutex<UiState>> {
        let state = Arc::new(Mutex::new(UiState::new()));
        unsafe {
            GLOBAL_UI_STATE = Some(state.clone());
        }
        state
    }

    pub fn global() -> Option<Arc<Mutex<UiState>>> {
        unsafe { GLOBAL_UI_STATE.clone() }
    }

    pub fn append_ping(&mut self, line: &str) {
        self.ping_output.push_str(line);
        self.ping_output.push('\n');
        trim_lines(&mut self.ping_output, 1000);
        self.updated.insert("ping_output".to_string(), true);
    }

    pub fn set_ping_stats(&mut self, stats: &str) {
        self.ping_stats = stats.to_string();
        self.updated.insert("ping_stats".to_string(), true);
    }

    pub fn append_scan(&mut self, line: &str) {
        self.scan_output.push_str(line);
        self.scan_output.push('\n');
        trim_lines(&mut self.scan_output, 1000);
        self.updated.insert("scan_output".to_string(), true);
    }

    pub fn append_http_log(&mut self, line: &str) {
        self.http_log.push_str(line);
        self.http_log.push('\n');
        trim_lines(&mut self.http_log, 1000);
        self.updated.insert("http_log".to_string(), true);
    }

    pub fn append_tftpd_log(&mut self, line: &str) {
        self.tftpd_log.push_str(line);
        self.tftpd_log.push('\n');
        trim_lines(&mut self.tftpd_log, 1000);
        self.updated.insert("tftpd_log".to_string(), true);
    }

    pub fn append_tftpc_log(&mut self, line: &str) {
        self.tftpc_log.push_str(line);
        self.tftpc_log.push('\n');
        trim_lines(&mut self.tftpc_log, 1000);
        self.updated.insert("tftpc_log".to_string(), true);
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

    pub fn is_updated(&self, key: &str) -> bool {
        self.updated.get(key).copied().unwrap_or(false)
    }

    pub fn clear_updated(&mut self, key: &str) {
        self.updated.remove(key);
    }
}

/// Helper functions for services to update UI
pub fn append_ping_output(line: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.append_ping(line);
        }
    }
}

pub fn set_ping_output(text: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.ping_output = text.to_string();
            s.updated.insert("ping_output".to_string(), true);
        }
    }
}

pub fn set_ping_stats(stats: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.set_ping_stats(stats);
        }
    }
}

pub fn append_scan_output(line: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.append_scan(line);
        }
    }
}

pub fn set_scan_output(text: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.scan_output = text.to_string();
            s.updated.insert("scan_output".to_string(), true);
        }
    }
}

pub fn set_scan_running(running: bool) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.scan_running = running;
            s.updated.insert("scan_running".to_string(), true);
        }
    }
}

pub fn append_http_log(line: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.append_http_log(line);
        }
    }
}

pub fn set_http_running(running: bool) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.http_running = running;
            s.updated.insert("http_running".to_string(), true);
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

pub fn append_tftpd_log(line: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.append_tftpd_log(line);
        }
    }
}

pub fn append_chat_message(sender: &str, message: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.append_chat(sender, message);
        }
    }
}

pub fn append_tftpc_log(line: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.append_tftpc_log(line);
        }
    }
}

pub fn set_plan_list(list: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            s.plan_list = format!("Scheduled Events:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n{}", list);
            s.updated.insert("plan_list".to_string(), true);
        }
    }
}

pub fn append_plan_event(id: &str, date: &str, time: &str, cycle: i32, unit: &str, msg: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            let cycle_str = if cycle > 0 {
                format!(" (Repeat every {} {})", cycle, unit)
            } else {
                " (One-time)".to_string()
            };
            if s.plan_list.contains("No scheduled events") {
                s.plan_list = "Scheduled Events:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n".to_string();
            }
            s.plan_list.push_str(&format!(
                "[{}] {} {} - {}{}\n",
                id, date, time, msg, cycle_str
            ));
            s.updated.insert("plan_list".to_string(), true);
        }
    }
}

pub fn remove_plan_event(id: &str) {
    if let Some(state) = UiState::global() {
        if let Ok(mut s) = state.lock() {
            let lines: Vec<&str> = s.plan_list.lines().collect();
            let mut new_list = String::from("Scheduled Events:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            let mut has_events = false;
            for line in lines {
                if line.starts_with("Scheduled") || line.starts_with("━") {
                    continue;
                }
                if !line.starts_with(&format!("[{}]", id)) {
                    new_list.push_str(line);
                    new_list.push('\n');
                    has_events = true;
                }
            }
            if !has_events {
                new_list.push_str("No scheduled events.\n\nUse + button to add a new reminder.\n");
            }
            s.plan_list = new_list;
            s.updated.insert("plan_list".to_string(), true);
        }
    }
}

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
            if !s.chat_users.contains(&format!("\n{}\n", user)) && !s.chat_users.ends_with(&format!("{}", user)) {
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
            s.chat_users = s.chat_users.replace(&format!("{}", user), "");
            if s.chat_users.trim() == "Online Users:\n───────────" {
                s.chat_users = "Online Users:\n───────────\n(no users)\n".to_string();
            }
            s.updated.insert("chat_users".to_string(), true);
        }
    }
}
