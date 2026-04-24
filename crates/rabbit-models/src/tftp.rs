//! TFTP Module Models

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::config::ModuleConfigs;

/// TFTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TftpServerConfig {
    pub enabled: bool,
    pub bind_addr: String,
    pub root_path: String,
    pub block_size: usize,
    pub timeout_secs: u64,
    pub window_size: u16,
    pub allow_overwrite: bool,
}

impl From<&ModuleConfigs> for TftpServerConfig {
    fn from(modules: &ModuleConfigs) -> Self {
        Self {
            enabled: false,
            bind_addr: format!("0.0.0.0:{}", modules.get_integer("tftpd", "port").unwrap_or(69)),
            root_path: modules.get_array("tftpd", "work_dirs")
                .and_then(|dirs| dirs.first().cloned())
                .unwrap_or_default(),
            block_size: modules.get_integer("tftpd", "blksize").unwrap_or(512) as usize,
            timeout_secs: modules.get_integer("tftpd", "timeout").unwrap_or(200) as u64 / 1000,
            window_size: 1,
            allow_overwrite: modules.get_bool("tftpd", "override_conflicts").unwrap_or(false),
        }
    }
}

/// TFTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TftpClientConfig {
    pub server_addr: String,
    pub local_port: u16,
    pub block_size: usize,
    pub timeout_secs: u64,
}

impl From<&ModuleConfigs> for TftpClientConfig {
    fn from(modules: &ModuleConfigs) -> Self {
        let server_addr = modules.get_string("tftpc", "server_addr").unwrap_or_else(|| "127.0.0.1".into());
        let server_port = modules.get_integer("tftpc", "server_port").unwrap_or(69) as u16;
        Self {
            server_addr: format!("{}:{}", server_addr, server_port),
            local_port: 0,
            block_size: modules.get_integer("tftpc", "blksize").unwrap_or(1024) as usize,
            timeout_secs: modules.get_integer("tftpc", "timeout").unwrap_or(200) as u64 / 1000,
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
