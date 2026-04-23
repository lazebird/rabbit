use serde::{Deserialize, Serialize};

/// 通用的配置值类型，支持 String、Integer、Boolean、Array 四种变体
/// 用于 section+map 方式更新配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Language,
    pub theme: Theme,
    pub systray: bool,
    pub top: bool,
    pub autostart: bool,
    pub autoupdate: bool,
    pub last_active_tab: usize,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub window_width: Option<i32>,
    pub window_height: Option<i32>,
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
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            modules: ModuleConfigs::default(),
        }
    }
}

impl AppConfig {
    pub fn merge_defaults(&mut self) {
        self.modules.ping.merge_defaults();
        self.modules.scan.merge_defaults();
        self.modules.http.merge_defaults();
        self.modules.tftpd.merge_defaults();
        self.modules.tftpc.merge_defaults();
        self.modules.plan.merge_defaults();
        self.modules.chat.merge_defaults();
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

impl ModuleConfigs {
    pub fn merge_defaults(&mut self) {
        self.ping.merge_defaults();
        self.scan.merge_defaults();
        self.http.merge_defaults();
        self.tftpd.merge_defaults();
        self.tftpc.merge_defaults();
        self.plan.merge_defaults();
        self.chat.merge_defaults();
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
    /// Whether ping was running when app was closed
    pub running: bool,
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
            running: false,
        }
    }
}

impl PingConfig {
    pub fn merge_defaults(&mut self) {
        let defaults = Self::default();
        if self.target.is_empty() { self.target = defaults.target; }
        if self.interval == 0 { self.interval = defaults.interval; }
        if self.count == 0 { self.count = defaults.count; }
    }

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
    pub fn merge_defaults(&mut self) {
        let defaults = Self::default();
        if self.start_ip.is_empty() { self.start_ip = defaults.start_ip; }
        if self.end_ip.is_empty() { self.end_ip = defaults.end_ip; }
    }

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
    /// Whether HTTP server was running when app was closed
    pub running: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            shell: false,
            autoindex: true,
            videoplay: true,
            dirs: Vec::new(),
            running: false,
        }
    }
}

impl HttpConfig {
    pub fn merge_defaults(&mut self) {
        if self.port == 0 { self.port = Self::default().port; }
    }

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
    /// Index of the currently selected working directory (0-based, None = first item)
    pub working_dir_index: Option<usize>,
    /// Whether TFTP server was running when app was closed
    pub running: bool,
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
            working_dir_index: None,
            running: false,
        }
    }
}

impl TftpdConfig {
    pub fn merge_defaults(&mut self) {
        let defaults = Self::default();
        if self.port == 0 { self.port = defaults.port; }
        if self.timeout == 0 { self.timeout = defaults.timeout; }
        if self.maxretry == 0 { self.maxretry = defaults.maxretry; }
        if self.blksize == 0 { self.blksize = defaults.blksize; }
        if self.qsize == 0 { self.qsize = defaults.qsize; }
        if self.qtout == 0 { self.qtout = defaults.qtout; }
    }

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
    pub fn merge_defaults(&mut self) {
        let defaults = Self::default();
        if self.server_addr.is_empty() { self.server_addr = defaults.server_addr; }
        if self.server_port == 0 { self.server_port = defaults.server_port; }
        if self.timeout == 0 { self.timeout = defaults.timeout; }
        if self.maxretry == 0 { self.maxretry = defaults.maxretry; }
        if self.blksize == 0 { self.blksize = defaults.blksize; }
    }

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

impl PlanConfig {
    pub fn merge_defaults(&mut self) {
        let defaults = Self::default();
        if self.unit.is_empty() { self.unit = defaults.unit; }
    }
}

/// Chat module configuration (UI defaults)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatModuleConfig {
    pub username: String,
    pub port: u16,
    pub broadcast_addr: String,
    /// Whether chat was running when app was closed
    pub running: bool,
}

impl Default for ChatModuleConfig {
    fn default() -> Self {
        Self {
            username: String::from("User@PC"),
            port: 1314,
            broadcast_addr: String::from("255.255.255.255"),
            running: false,
        }
    }
}

impl ChatModuleConfig {
    pub fn merge_defaults(&mut self) {
        let defaults = Self::default();
        if self.username.is_empty() { self.username = defaults.username; }
        if self.port == 0 { self.port = defaults.port; }
        if self.broadcast_addr.is_empty() { self.broadcast_addr = defaults.broadcast_addr; }
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
