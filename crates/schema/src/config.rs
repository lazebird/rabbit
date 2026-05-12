use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// 计划任务结构化数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTask {
    pub msg: String,
    pub date: String,
    pub time: String,
    pub cycle: i32,
    pub unit: String,
}

/// 窗口位置和大小配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        self.as_i64()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_string_array(&self) -> Option<Vec<String>> {
        self.as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub modules: ModuleConfigs,
}

/// 配置键常量定义
///
/// 所有模块配置键统一在此定义，避免魔术字符串在代码中散落。
/// 使用方式：`config.modules.get_bool("ping", keys::ping::RUNNING)`
pub mod keys {
    pub mod global {
        pub const LANGUAGE: &str = "language";
        pub const THEME: &str = "theme";
        pub const SYSTRAY: &str = "systray";
        pub const TOP: &str = "top";
        pub const AUTOSTART: &str = "autostart";
        pub const AUTOUPDATE: &str = "autoupdate";
        pub const LAST_ACTIVE_TAB: &str = "last_active_tab";
        pub const WINDOW: &str = "window";
    }

    pub mod ping {
        pub const TARGET: &str = "target";
        pub const INTERVAL: &str = "interval";
        pub const COUNT: &str = "count";
        pub const STOP_ON_LOSS: &str = "stoponloss";
        pub const TASKBAR: &str = "taskbar";
        pub const LOG: &str = "log";
        pub const RUNNING: &str = "running";
    }

    pub mod scan {
        pub const START_IP: &str = "start_ip";
        pub const END_IP: &str = "end_ip";
        pub const FILTER: &str = "filter";
        pub const RUNNING: &str = "running";
    }

    pub mod http {
        pub const PORT: &str = "port";
        pub const SHELL: &str = "shell";
        pub const AUTO_INDEX: &str = "autoindex";
        pub const VIDEO_PLAY: &str = "videoplay";
        pub const DIRS: &str = "dirs";
        pub const LOG: &str = "log";
        pub const RUNNING: &str = "running";
    }

    pub mod tftpd {
        pub const PORT: &str = "port";
        pub const TIMEOUT: &str = "timeout";
        pub const MAX_RETRY: &str = "maxretry";
        pub const BLK_SIZE: &str = "blksize";
        pub const QSIZE: &str = "qsize";
        pub const QTOUT: &str = "qtout";
        pub const OVERRIDE: &str = "override";
        pub const FSLOG: &str = "fslog";
        pub const WORK_DIRS: &str = "work_dirs";
        pub const WORKING_DIR_INDEX: &str = "working_dir_index";
        pub const RUNNING: &str = "running";
    }

    pub mod tftpc {
        pub const SERVER_ADDR: &str = "server_addr";
        pub const SERVER_PORT: &str = "server_port";
        pub const LOCAL_PATH: &str = "local_path";
        pub const REMOTE_FILE: &str = "remote_file";
        pub const TIMEOUT: &str = "timeout";
        pub const MAX_RETRY: &str = "maxretry";
        pub const BLK_SIZE: &str = "blksize";
    }

    pub mod plan {
        pub const TASKS: &str = "tasks";
        pub const OVERRIDE: &str = "override";
    }

    pub mod chat {
        pub const USERNAME: &str = "username";
        pub const PORT: &str = "port";
        pub const BROADCAST_ADDR: &str = "broadcast_addr";
        pub const RUNNING: &str = "running";
    }
}

impl AppConfig {
    pub fn merge_defaults(&mut self) {
        self.modules.merge_defaults();
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfigs {
    pub global: HashMap<String, ConfigValue>,
    pub ping: HashMap<String, ConfigValue>,
    pub scan: HashMap<String, ConfigValue>,
    pub http: HashMap<String, ConfigValue>,
    pub tftpd: HashMap<String, ConfigValue>,
    pub tftpc: HashMap<String, ConfigValue>,
    pub plan: HashMap<String, ConfigValue>,
    pub chat: HashMap<String, ConfigValue>,
}

impl Default for ModuleConfigs {
    fn default() -> Self {
        Self {
            global: Self::default_global(),
            ping: Self::default_ping(),
            scan: Self::default_scan(),
            http: Self::default_http(),
            tftpd: Self::default_tftpd(),
            tftpc: Self::default_tftpc(),
            plan: Self::default_plan(),
            chat: Self::default_chat(),
        }
    }
}

impl ModuleConfigs {
    pub fn merge_defaults(&mut self) {
        let defaults = Self::default();
        for (k, v) in defaults.global {
            self.global.entry(k).or_insert(v);
        }
        for (k, v) in defaults.ping {
            self.ping.entry(k).or_insert(v);
        }
        for (k, v) in defaults.scan {
            self.scan.entry(k).or_insert(v);
        }
        for (k, v) in defaults.http {
            self.http.entry(k).or_insert(v);
        }
        for (k, v) in defaults.tftpd {
            self.tftpd.entry(k).or_insert(v);
        }
        for (k, v) in defaults.tftpc {
            self.tftpc.entry(k).or_insert(v);
        }
        for (k, v) in defaults.plan {
            self.plan.entry(k).or_insert(v);
        }
        for (k, v) in defaults.chat {
            self.chat.entry(k).or_insert(v);
        }
    }

    pub fn get_string(&self, module: &str, key: &str) -> Option<String> {
        self.get_map(module)?.get(key)?.as_str().map(|s| s.to_string())
    }

    pub fn get_integer(&self, module: &str, key: &str) -> Option<i64> {
        self.get_map(module)?.get(key)?.as_integer()
    }

    pub fn get_bool(&self, module: &str, key: &str) -> Option<bool> {
        self.get_map(module)?.get(key)?.as_bool()
    }

    pub fn get_array(&self, module: &str, key: &str) -> Option<Vec<String>> {
        self.get_map(module)?.get(key)?.as_string_array()
    }

    pub fn insert(&mut self, module: &str, key: &str, value: ConfigValue) {
        if let Some(map) = self.get_map_mut(module) {
            map.insert(key.to_string(), value);
        }
    }

    fn get_map(&self, module: &str) -> Option<&HashMap<String, ConfigValue>> {
        match module {
            "global" => Some(&self.global),
            "ping" => Some(&self.ping),
            "scan" => Some(&self.scan),
            "http" => Some(&self.http),
            "tftpd" => Some(&self.tftpd),
            "tftpc" => Some(&self.tftpc),
            "plan" => Some(&self.plan),
            "chat" => Some(&self.chat),
            _ => None,
        }
    }

    fn get_map_mut(&mut self, module: &str) -> Option<&mut HashMap<String, ConfigValue>> {
        match module {
            "global" => Some(&mut self.global),
            "ping" => Some(&mut self.ping),
            "scan" => Some(&mut self.scan),
            "http" => Some(&mut self.http),
            "tftpd" => Some(&mut self.tftpd),
            "tftpc" => Some(&mut self.tftpc),
            "plan" => Some(&mut self.plan),
            "chat" => Some(&mut self.chat),
            _ => None,
        }
    }

    fn default_global() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("language".into(), ConfigValue::String("System".into())),
            ("theme".into(), ConfigValue::String("System".into())),
            ("systray".into(), ConfigValue::Boolean(true)),
            ("top".into(), ConfigValue::Boolean(false)),
            ("autostart".into(), ConfigValue::Boolean(false)),
            ("autoupdate".into(), ConfigValue::Boolean(true)),
            ("last_active_tab".into(), ConfigValue::Integer(0)),
            ("window".into(), ConfigValue::String(r#"{"x":100,"y":100,"width":800,"height":600}"#.into())),
        ])
    }

    fn default_ping() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("target".into(), ConfigValue::String("1.1.1.1".into())),
            ("interval".into(), ConfigValue::Integer(1000)),
            ("count".into(), ConfigValue::Integer(-1)),
            ("stoponloss".into(), ConfigValue::Boolean(false)),
            ("taskbar".into(), ConfigValue::Boolean(true)),
            ("log".into(), ConfigValue::String(String::new())),
            ("running".into(), ConfigValue::Boolean(false)),
        ])
    }

    fn default_scan() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("start_ip".into(), ConfigValue::String("192.168.1.1".into())),
            ("end_ip".into(), ConfigValue::String("254".into())),
            ("filter".into(), ConfigValue::Boolean(true)),
        ])
    }

    fn default_http() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("port".into(), ConfigValue::Integer(8000)),
            ("shell".into(), ConfigValue::Boolean(false)),
            ("autoindex".into(), ConfigValue::Boolean(true)),
            ("videoplay".into(), ConfigValue::Boolean(true)),
            ("dirs".into(), ConfigValue::Array(Vec::new())),
            ("running".into(), ConfigValue::Boolean(false)),
        ])
    }

    fn default_tftpd() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("port".into(), ConfigValue::Integer(69)),
            ("timeout".into(), ConfigValue::Integer(200)),
            ("maxretry".into(), ConfigValue::Integer(10)),
            ("blksize".into(), ConfigValue::Integer(512)),
            ("qsize".into(), ConfigValue::Integer(2000)),
            ("qtout".into(), ConfigValue::Integer(1000)),
            ("override".into(), ConfigValue::Boolean(false)),
            ("fslog".into(), ConfigValue::Boolean(false)),
            ("work_dirs".into(), ConfigValue::Array(Vec::new())),
            ("working_dir_index".into(), ConfigValue::Integer(0)),
            ("running".into(), ConfigValue::Boolean(false)),
        ])
    }

    fn default_tftpc() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("server_addr".into(), ConfigValue::String("127.0.0.1".into())),
            ("server_port".into(), ConfigValue::Integer(69)),
            ("local_path".into(), ConfigValue::String(String::new())),
            ("remote_file".into(), ConfigValue::String(String::new())),
            ("timeout".into(), ConfigValue::Integer(200)),
            ("maxretry".into(), ConfigValue::Integer(10)),
            ("blksize".into(), ConfigValue::Integer(1024)),
        ])
    }

    fn default_plan() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("tasks".into(), ConfigValue::Array(Vec::new())),
            ("override".into(), ConfigValue::Boolean(false)),
        ])
    }

    fn default_chat() -> HashMap<String, ConfigValue> {
        HashMap::from([
            ("username".into(), ConfigValue::String("User@PC".into())),
            ("port".into(), ConfigValue::Integer(1314)),
            ("broadcast_addr".into(), ConfigValue::String("255.255.255.255".into())),
            ("running".into(), ConfigValue::Boolean(false)),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.modules.get_string("global", keys::global::LANGUAGE), Some("System".into()));
        assert_eq!(config.modules.get_string("global", keys::global::THEME), Some("System".into()));
        assert_eq!(config.modules.get_bool("global", keys::global::SYSTRAY), Some(true));
        assert_eq!(config.modules.get_bool("global", keys::global::TOP), Some(false));
        assert_eq!(config.modules.get_bool("global", keys::global::AUTOSTART), Some(false));
        assert_eq!(config.modules.get_bool("global", keys::global::AUTOUPDATE), Some(true));
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.modules.get_string("global", keys::global::LANGUAGE), parsed.modules.get_string("global", keys::global::LANGUAGE));
        assert_eq!(config.modules.get_string("global", keys::global::THEME), parsed.modules.get_string("global", keys::global::THEME));
    }

    #[test]
    fn test_module_configs_map() {
        let config = ModuleConfigs::default();
        assert_eq!(config.get_string("ping", keys::ping::TARGET), Some("1.1.1.1".into()));
        assert_eq!(config.get_integer("ping", keys::ping::INTERVAL), Some(1000));
        assert_eq!(config.get_string("scan", keys::scan::START_IP), Some("192.168.1.1".into()));
        assert_eq!(config.get_integer("http", keys::http::PORT), Some(8000));
        assert_eq!(config.get_integer("tftpd", keys::tftpd::PORT), Some(69));
        assert_eq!(config.get_string("chat", keys::chat::USERNAME), Some("User@PC".into()));
    }

    #[test]
    fn test_config_value_accessors() {
        let val = ConfigValue::String("test".into());
        assert_eq!(val.as_str(), Some("test"));
        assert_eq!(val.as_integer(), None);
        assert_eq!(val.as_bool(), None);

        let val = ConfigValue::Integer(42);
        assert_eq!(val.as_str(), None);
        assert_eq!(val.as_integer(), Some(42));

        let val = ConfigValue::Boolean(true);
        assert_eq!(val.as_bool(), Some(true));
    }
}
