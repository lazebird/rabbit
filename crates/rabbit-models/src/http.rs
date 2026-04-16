//! HTTP Server Module Models

use serde::{Deserialize, Serialize};

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

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8080,
            root_path: String::from("."),
            allow_upload: false,
            allow_delete: false,
            shell: false,
            auto_index: false,
            video_play: false,
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
