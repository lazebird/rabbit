//! Configuration Models

use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Language,
    pub theme: Theme,
    pub modules: ModuleConfigs,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::English,
            theme: Theme::System,
            modules: ModuleConfigs::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
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
    pub http: HttpConfig,
    pub tftp: TftpConfig,
    pub plan: PlanConfig,
    pub chat: ChatModuleConfig,
}

impl Default for ModuleConfigs {
    fn default() -> Self {
        Self {
            ping: PingConfig::default(),
            http: HttpConfig::default(),
            tftp: TftpConfig::default(),
            plan: PlanConfig::default(),
            chat: ChatModuleConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingConfig {
    pub default_count: u32,
    pub default_interval_ms: u64,
    pub default_timeout_ms: u64,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            default_count: 4,
            default_interval_ms: 1000,
            default_timeout_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub default_port: u16,
    pub default_root: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            default_port: 8080,
            default_root: String::from("."),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TftpConfig {
    pub server_port: u16,
    pub client_port: u16,
    pub default_root: String,
    pub block_size: usize,
    pub timeout_secs: u64,
}

impl Default for TftpConfig {
    fn default() -> Self {
        Self {
            server_port: 69,
            client_port: 0,
            default_root: String::from("."),
            block_size: 512,
            timeout_secs: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanConfig {
    pub reminder_enabled: bool,
}

impl Default for PlanConfig {
    fn default() -> Self {
        Self {
            reminder_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatModuleConfig {
    pub default_port: u16,
    pub default_username: String,
}

impl Default for ChatModuleConfig {
    fn default() -> Self {
        Self {
            default_port: 5000,
            default_username: String::from("User"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.language, Language::English);
        assert_eq!(config.theme, Theme::System);
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
        assert_eq!(config.default_count, 4);
        assert_eq!(config.default_interval_ms, 1000);
        assert_eq!(config.default_timeout_ms, 2000);
    }

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.default_port, 8080);
        assert_eq!(config.default_root, ".");
    }

    #[test]
    fn test_tftp_config_default() {
        let config = TftpConfig::default();
        assert_eq!(config.server_port, 69);
        assert_eq!(config.block_size, 512);
    }
}
