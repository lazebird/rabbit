//! HTTP Server Module Models

use serde::{Deserialize, Serialize};

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

