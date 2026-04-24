//! TFTP Module Models

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

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

