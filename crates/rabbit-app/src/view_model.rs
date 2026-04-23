use rabbit_models::config::AppConfig;
use rabbit_models::ping::PingSummary;
use rabbit_platform::config::save_config;
use rabbit_platform::{Result, PlatformError};
use std::collections::HashMap;

pub struct AppViewModel {
    config: AppConfig,
    ping_results: HashMap<String, PingSummary>,
    ping_running: bool,
    http_running: bool,
    tftp_server_running: bool,
    chat_running: bool,
    scan_running: bool,
    chat_messages: Vec<ChatMessageView>,
    scan_results: Vec<ScanResultView>,
}

impl AppViewModel {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            ping_results: HashMap::new(),
            ping_running: false,
            http_running: false,
            tftp_server_running: false,
            chat_running: false,
            scan_running: false,
            chat_messages: Vec::new(),
            scan_results: Vec::new(),
        }
    }

    pub fn get_config(&self) -> AppConfig {
        self.config.clone()
    }

    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }

    pub fn save_settings(&mut self) -> Result<()> {
        save_config(&self.config)
    }

    pub fn update_and_save(&mut self, config: AppConfig) -> Result<()> {
        self.config = config;
        save_config(&self.config)
    }

    pub fn update_global(&mut self, language: rabbit_models::config::Language, theme: rabbit_models::config::Theme, systray: bool, top: bool, autostart: bool, autoupdate: bool) -> Result<()> {
        self.config.language = language;
        self.config.theme = theme;
        self.config.systray = systray;
        self.config.top = top;
        self.config.autostart = autostart;
        self.config.autoupdate = autoupdate;
        save_config(&self.config)
    }

    pub fn update_last_tab(&mut self, tab: usize) -> Result<()> {
        self.config.last_active_tab = tab;
        save_config(&self.config)
    }

    pub fn is_ping_running(&self) -> bool {
        self.ping_running
    }

    pub fn set_ping_running(&mut self, running: bool) {
        self.ping_running = running;
    }

    pub fn get_ping_result(&self, target: &str) -> Option<&PingSummary> {
        self.ping_results.get(target)
    }

    pub fn update_ping_result(&mut self, result: PingSummary) {
        self.ping_results.insert(result.target.clone(), result);
    }

    pub fn is_http_running(&self) -> bool {
        self.http_running
    }

    pub fn set_http_running(&mut self, running: bool) {
        self.http_running = running;
    }

    pub fn is_tftp_server_running(&self) -> bool {
        self.tftp_server_running
    }

    pub fn set_tftp_server_running(&mut self, running: bool) {
        self.tftp_server_running = running;
    }

    pub fn get_chat_messages(&self) -> &[ChatMessageView] {
        &self.chat_messages
    }

    pub fn add_chat_message(&mut self, message: ChatMessageView) {
        self.chat_messages.push(message);
        if self.chat_messages.len() > 100 {
            self.chat_messages.remove(0);
        }
    }

    pub fn is_scan_running(&self) -> bool {
        self.scan_running
    }

    pub fn set_scan_running(&mut self, running: bool) {
        self.scan_running = running;
    }

    pub fn get_scan_results(&self) -> &[ScanResultView] {
        &self.scan_results
    }

    pub fn update_scan_results(&mut self, results: Vec<ScanResultView>) {
        self.scan_results = results;
    }

    pub fn is_chat_running(&self) -> bool {
        self.chat_running
    }

    pub fn set_chat_running(&mut self, running: bool) {
        self.chat_running = running;
    }
}

#[derive(Clone, Debug)]
pub struct ChatMessageView {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    pub is_me: bool,
}

#[derive(Clone, Debug)]
pub struct ScanResultView {
    pub ip: String,
    pub online: bool,
    pub hostname: Option<String>,
    pub response_time: Option<String>,
}