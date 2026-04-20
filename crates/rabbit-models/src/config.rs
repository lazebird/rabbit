//! Configuration Models

use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Language,
    pub theme: Theme,
    pub systray: bool,
    pub top: bool,
    pub autostart: bool,
    pub autoupdate: bool,
    pub last_active_tab: usize,
    pub modules: ModuleConfigs,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::System,
            theme: Theme::System,
            systray: true,
            top: false,
            autostart: false,
            autoupdate: true,
            last_active_tab: 0,
            modules: ModuleConfigs::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    System,
    English,
    Chinese,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Module-specific configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfigs {
    pub ping: PingConfig,
    pub scan: ScanConfig,
    pub http: HttpConfig,
    pub tftpd: TftpdConfig,
    pub tftpc: TftpcConfig,
    pub plan: PlanConfig,
    pub chat: ChatModuleConfig,
}

impl Default for ModuleConfigs {
    fn default() -> Self {
        Self {
            ping: PingConfig::default(),
            scan: ScanConfig::default(),
            http: HttpConfig::default(),
            tftpd: TftpdConfig::default(),
            tftpc: TftpcConfig::default(),
            plan: PlanConfig::default(),
            chat: ChatModuleConfig::default(),
        }
    }
}

/// Ping module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingConfig {
    pub target: String,
    pub interval: i32,
    pub count: i32,
    pub stoponloss: bool,
    pub taskbar: bool,
    pub log: String,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            target: String::from("1.1.1.1"),
            interval: 1000,
            count: -1,
            stoponloss: false,
            taskbar: true,
            log: String::new(),
        }
    }
}

impl PingConfig {
    /// Get options string in key=value format
    pub fn opts_string(&self) -> String {
        format!(
            "interval={};count={};stoponloss={}",
            self.interval, self.count, self.stoponloss
        )
    }
}

/// Scan module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub start_ip: String,
    pub end_ip: String,
    pub filter: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            start_ip: String::from("192.168.1.1"),
            end_ip: String::from("254"),
            filter: true,
        }
    }
}

impl ScanConfig {
    /// Get options string in key=value format
    pub fn opts_string(&self) -> String {
        format!("filter={}", self.filter)
    }
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub port: u16,
    pub shell: bool,
    pub autoindex: bool,
    pub videoplay: bool,
    pub dirs: Vec<String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            shell: false,
            autoindex: true,
            videoplay: true,
            dirs: Vec::new(),
        }
    }
}

impl HttpConfig {
    /// Get options string in key=value format
    pub fn opts_string(&self) -> String {
        format!(
            "autoindex={};videoplay={};",
            self.autoindex, self.videoplay
        )
    }
}

/// TFTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TftpdConfig {
    pub port: u16,
    pub timeout: i32,
    pub maxretry: i32,
    pub blksize: i32,
    pub qsize: i32,
    pub qtout: i32,
    pub override_conflicts: bool,
    pub fslog: bool,
    pub work_dirs: Vec<String>,
}

impl Default for TftpdConfig {
    fn default() -> Self {
        Self {
            port: 69,
            timeout: 200,
            maxretry: 10,
            blksize: 512,
            qsize: 2000,
            qtout: 1000,
            override_conflicts: false,
            fslog: false,
            work_dirs: Vec::new(),
        }
    }
}

impl TftpdConfig {
    /// Get options string in key=value format
    pub fn opts_string(&self) -> String {
        format!(
            "timeout={};retry={};blksize={};override={};qsize={};qtout={};fslog={};",
            self.timeout,
            self.maxretry,
            self.blksize,
            self.override_conflicts,
            self.qsize,
            self.qtout,
            self.fslog
        )
    }
}

/// TFTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TftpcConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub local_path: String,
    pub remote_file: String,
    pub timeout: i32,
    pub maxretry: i32,
    pub blksize: i32,
}

impl Default for TftpcConfig {
    fn default() -> Self {
        Self {
            server_addr: String::from("127.0.0.1"),
            server_port: 69,
            local_path: String::new(),  // Empty - UI should set appropriate default
            remote_file: String::new(), // Empty - UI should set appropriate default
            timeout: 200,
            maxretry: 10,
            blksize: 1024,
        }
    }
}

impl TftpcConfig {
    /// Get options string in key=value format
    pub fn opts_string(&self) -> String {
        format!(
            "timeout={};retry={};blksize={};",
            self.timeout, self.maxretry, self.blksize
        )
    }
}

/// Plan/Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanConfig {
    pub date: String,
    pub time: String,
    pub cycle: i32,
    pub unit: String,
    pub msg: String,
    pub override_conflicts: bool,
}

impl Default for PlanConfig {
    fn default() -> Self {
        Self {
            date: String::new(),  // Empty - UI should set current date
            time: String::new(),  // Empty - UI should set current time
            cycle: 0,
            unit: String::from("minutes"),
            msg: String::new(),   // Empty - user should enter
            override_conflicts: false,
        }
    }
}

/// Chat module configuration (UI defaults)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatModuleConfig {
    pub username: String,
    pub port: u16,
    pub broadcast_addr: String,
}

impl Default for ChatModuleConfig {
    fn default() -> Self {
        Self {
            username: String::from("User@PC"),
            port: 1314,
            broadcast_addr: String::from("255.255.255.255"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.language, Language::System);
        assert_eq!(config.theme, Theme::System);
        assert!(config.systray);
        assert!(!config.top);
        assert!(!config.autostart);
        assert!(config.autoupdate);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.language, parsed.language);
        assert_eq!(config.theme, parsed.theme);
    }

    #[test]
    fn test_ping_config_default() {
        let config = PingConfig::default();
        assert_eq!(config.target, "1.1.1.1");
        assert_eq!(config.interval, 1000);
        assert_eq!(config.count, -1);
        assert!(!config.stoponloss);
        assert!(config.taskbar);
    }

    #[test]
    fn test_ping_opts_string() {
        let config = PingConfig::default();
        let opts = config.opts_string();
        assert!(opts.contains("interval=1000"));
        assert!(opts.contains("count=-1"));
        assert!(opts.contains("stoponloss=false"));
    }

    #[test]
    fn test_scan_config_default() {
        let config = ScanConfig::default();
        assert_eq!(config.start_ip, "192.168.1.1");
        assert_eq!(config.end_ip, "254");
        assert!(config.filter);
    }

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.port, 8000);
        assert!(!config.shell);
        assert!(config.autoindex);  // default is true
        assert!(config.videoplay);  // default is true
    }

    #[test]
    fn test_tftpd_config_default() {
        let config = TftpdConfig::default();
        assert_eq!(config.port, 69);
        assert_eq!(config.timeout, 200);
        assert_eq!(config.maxretry, 10);
        assert_eq!(config.blksize, 512);
        assert_eq!(config.qsize, 2000);
        assert_eq!(config.qtout, 1000);
        assert!(!config.override_conflicts);
        assert!(!config.fslog);
    }

    #[test]
    fn test_tftpc_config_default() {
        let config = TftpcConfig::default();
        assert_eq!(config.server_addr, "127.0.0.1");
        assert_eq!(config.server_port, 69);
        assert_eq!(config.blksize, 1024);
        assert!(config.local_path.is_empty()); // UI should set platform-appropriate default
        assert!(config.remote_file.is_empty());
    }

    #[test]
    fn test_plan_config_default() {
        let config = PlanConfig::default();
        assert!(config.date.is_empty()); // UI should set current date
        assert!(config.time.is_empty()); // UI should set current time
        assert_eq!(config.unit, "minutes");
        assert!(config.msg.is_empty()); // User should enter
        assert!(!config.override_conflicts);
    }

    #[test]
    fn test_chat_config_default() {
        let config = ChatModuleConfig::default();
        assert_eq!(config.username, "User@PC");
        assert_eq!(config.port, 1314);
        assert_eq!(config.broadcast_addr, "255.255.255.255");
    }
}
