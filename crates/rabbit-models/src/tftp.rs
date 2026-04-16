//! TFTP Module Models

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// TFTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TftpServerConfig {
    pub enabled: bool,
    pub bind_addr: String,
    pub root_path: String,
    pub block_size: usize,
    pub timeout_secs: u64,
    pub window_size: u16,
    pub allow_overwrite: bool,
}

impl Default for TftpServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_addr: String::from("0.0.0.0:69"),
            root_path: String::from("."),
            block_size: 512,
            timeout_secs: 5,
            window_size: 1,
            allow_overwrite: false,
        }
    }
}

/// TFTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TftpClientConfig {
    pub server_addr: String,
    pub local_port: u16,
    pub block_size: usize,
    pub timeout_secs: u64,
}

impl Default for TftpClientConfig {
    fn default() -> Self {
        Self {
            server_addr: String::from("127.0.0.1:69"),
            local_port: 0,
            block_size: 512,
            timeout_secs: 5,
        }
    }
}

/// TFTP transfer operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TftpOperation {
    Upload,
    Download,
}

/// TFTP transfer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TftpTransferState {
    Idle,
    Connecting,
    Transferring { progress: u8 },
    Completed,
    Error,
}

/// TFTP transfer info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TftpTransfer {
    pub id: String,
    pub operation: String,
    pub filename: String,
    pub remote_addr: String,
    pub state: String,
    pub progress: u8,
    pub bytes_transferred: u64,
    pub total_bytes: Option<u64>,
    pub error: Option<String>,
}

/// TFTP log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TftpLogEntry {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub remote_addr: SocketAddr,
    pub operation: String,
    pub filename: String,
    pub success: bool,
    pub bytes_transferred: u64,
}
