use rabbit_models::config::AppConfig;
use rabbit_models::ping::PingSummary;
use rabbit_platform::config::save_config;
use std::collections::HashMap;

/// Main application view model
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

    pub async fn save_settings(&mut self) -> anyhow::Result<()> {
        save_config(&self.config)?;
        Ok(())
    }

    // Ping
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

    // HTTP
    pub fn is_http_running(&self) -> bool {
        self.http_running
    }

    pub fn set_http_running(&mut self, running: bool) {
        self.http_running = running;
    }

    // TFTP
    pub fn is_tftp_server_running(&self) -> bool {
        self.tftp_server_running
    }

    pub fn set_tftp_server_running(&mut self, running: bool) {
        self.tftp_server_running = running;
    }

    // Chat
    pub fn get_chat_messages(&self) -> &[ChatMessageView] {
        &self.chat_messages
    }

    pub fn add_chat_message(&mut self, message: ChatMessageView) {
        self.chat_messages.push(message);
        // Keep only last 100 messages
        if self.chat_messages.len() > 100 {
            self.chat_messages.remove(0);
        }
    }

    // Scan
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

    // Chat
    pub fn is_chat_running(&self) -> bool {
        self.chat_running
    }

    pub fn set_chat_running(&mut self, running: bool) {
        self.chat_running = running;
    }
}

/// Chat message view model
#[derive(Clone, Debug)]
pub struct ChatMessageView {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    pub is_me: bool,
}

/// Scan result view model
#[derive(Clone, Debug)]
pub struct ScanResultView {
    pub ip: String,
    pub online: bool,
    pub hostname: Option<String>,
    pub response_time: Option<String>,
}
