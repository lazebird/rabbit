//! HTTP Server Module Models

use serde::{Deserialize, Serialize};

use crate::config::HttpConfig;

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

impl From<&HttpConfig> for HttpServerConfig {
    fn from(config: &HttpConfig) -> Self {
        Self {
            enabled: false,
            port: config.port,
            root_path: config.dirs.first().cloned().unwrap_or_default(),
            allow_upload: false,
            allow_delete: false,
            shell: config.shell,
            auto_index: config.autoindex,
            video_play: config.videoplay,
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
