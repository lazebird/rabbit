//! HTTP Server Module Models

use serde::{Deserialize, Serialize};

use crate::config::{ConfigValue, ModuleConfigs};

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    pub enabled: bool,
    pub port: u16,
    pub root_path: String,
    pub allow_upload: bool,
    pub allow_delete: bool,
    pub shell: bool,
    pub auto_index: bool,
    pub video_play: bool,
}

impl From<&ModuleConfigs> for HttpServerConfig {
    fn from(modules: &ModuleConfigs) -> Self {
        Self {
            enabled: false,
            port: modules.get_integer("http", "port").unwrap_or(8000) as u16,
            root_path: modules.get_array("http", "dirs")
                .and_then(|dirs| dirs.first().cloned())
                .unwrap_or_default(),
            allow_upload: false,
            allow_delete: false,
            shell: modules.get_bool("http", "shell").unwrap_or(false),
            auto_index: modules.get_bool("http", "autoindex").unwrap_or(true),
            video_play: modules.get_bool("http", "videoplay").unwrap_or(true),
        }
    }
}

/// HTTP server state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpServerState {
    Stopped,
    Starting,
    Running,
    Error,
}

/// HTTP request log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAccessLog {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub bytes_sent: u64,
    pub client_addr: String,
}
