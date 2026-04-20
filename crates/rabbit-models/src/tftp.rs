//! TFTP Module Models

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::config::{TftpdConfig, TftpcConfig};

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

impl From<&TftpdConfig> for TftpServerConfig {
    fn from(config: &TftpdConfig) -> Self {
        Self {
            enabled: false,
            bind_addr: format!("0.0.0.0:{}", config.port),
            root_path: config.work_dirs.first().cloned().unwrap_or_default(),
            block_size: config.blksize as usize,
            timeout_secs: config.timeout as u64 / 1000,
            window_size: 1,
            allow_overwrite: config.override_conflicts,
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

impl From<&TftpcConfig> for TftpClientConfig {
    fn from(config: &TftpcConfig) -> Self {
        Self {
            server_addr: format!("{}:{}", config.server_addr, config.server_port),
            local_port: 0,
            block_size: config.blksize as usize,
            timeout_secs: config.timeout as u64 / 1000,
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
