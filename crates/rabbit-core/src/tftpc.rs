//! TFTP Client Service

use crate::{Result, ServiceError, ui_channel::{UiData, Module}};
use rabbit_models::tftp::TftpTransfer;
use rabbit_platform::config::{get_integer, get_string};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{error, info};

const TFTP_OPCODE_RRQ: u16 = 1;
const TFTP_OPCODE_WRQ: u16 = 2;
const TFTP_OPCODE_DATA: u16 = 3;
const TFTP_OPCODE_ACK: u16 = 4;
const TFTP_OPCODE_ERROR: u16 = 5;
const TFTP_BLOCK_SIZE: usize = 512;
const TFTP_TIMEOUT_SECS: u64 = 5;

/// TFTP client internal configuration
#[derive(Debug, Clone, Default)]
struct ClientConfig {
    pub server_addr: String,
    pub block_size: usize,
    pub timeout_secs: u64,
}

impl ClientConfig {
    fn from_platform() -> Self {
        let server_addr = get_string("tftpc", "server_addr").unwrap_or_else(|| "127.0.0.1".into());
        let server_port = get_integer("tftpc", "server_port").unwrap_or(69) as u16;
        Self {
            server_addr: format!("{}:{}", server_addr, server_port),
            block_size: get_integer("tftpc", "blksize").unwrap_or(1024) as usize,
            timeout_secs: get_integer("tftpc", "timeout").unwrap_or(200) as u64 / 1000,
        }
    }
}

/// TFTP Client Service
pub struct TftpcService {
    transfers: Arc<RwLock<HashMap<String, TftpTransfer>>>,
    tx: Option<tokio::sync::mpsc::Sender<UiData>>,
}

impl TftpcService {
    pub fn new() -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            tx: None,
        }
    }

    pub fn with_channel(tx: tokio::sync::mpsc::Sender<UiData>) -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            tx: Some(tx),
        }
    }

    pub async fn send(&self, data: UiData) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(data).await;
        }
    }

    /// Initialize - now a no-op as config is pulled on operations
    pub async fn init(&mut self) -> Result<()> {
        info!("TFTP client service initialized");
        Ok(())
    }

    pub async fn put(&self, local_path: &str, remote_filename: &str) -> Result<String> {
        let config = ClientConfig::from_platform();
        self.upload_to(&config.server_addr, local_path, remote_filename).await
    }

    pub async fn get(&self, remote_filename: &str, local_path: &str) -> Result<String> {
        let config = ClientConfig::from_platform();
        self.download_from(&config.server_addr, remote_filename, local_path).await
    }


    async fn upload_to(&self, server_addr: &str, local_path: &str, remote_filename: &str) -> Result<String> {
        let transfer_id = format!("upload_{}_{}", remote_filename, chrono::Local::now().timestamp());

        let transfer = TftpTransfer {
            id: transfer_id.clone(),
            operation: "upload".to_string(),
            filename: remote_filename.to_string(),
            remote_addr: server_addr.to_string(),
            state: "transferring".to_string(),
            progress: 0,
            bytes_transferred: 0,
            total_bytes: None,
            error: None,
        };

        self.transfers.write().await.insert(transfer_id.clone(), transfer.clone());

        let transfers = Arc::clone(&self.transfers);
        let tid = transfer_id.clone();
        let server = server_addr.to_string();
        let local = local_path.to_string();
        let remote = remote_filename.to_string();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            match Self::do_upload(&server, &local, &remote).await {
                Ok(bytes) => {
                    info!("Upload completed: {} bytes", bytes);
                    if let Some(t) = transfers.write().await.get_mut(&tid) {
                        t.state = "completed".to_string();
                        t.bytes_transferred = bytes as u64;
                        t.progress = 100;
                    }
                    if let Some(ref tx) = tx {
                        let _ = tx.send(UiData::Log(Module::Tftpc, format!("Uploaded {} bytes", bytes)));
                    }
                }
                Err(e) => {
                    error!("Upload failed: {}", e);
                    if let Some(t) = transfers.write().await.get_mut(&tid) {
                        t.state = "error".to_string();
                        t.error = Some(e.to_string());
                    }
                }
            }
        });

        info!("Starting upload: {} -> {}@{}", local_path, remote_filename, server_addr);
        Ok(transfer_id)
    }

    async fn download_from(&self, server_addr: &str, remote_filename: &str, local_path: &str) -> Result<String> {
        let transfer_id = format!("download_{}_{}", remote_filename, chrono::Local::now().timestamp());

        let transfer = TftpTransfer {
            id: transfer_id.clone(),
            operation: "download".to_string(),
            filename: remote_filename.to_string(),
            remote_addr: server_addr.to_string(),
            state: "transferring".to_string(),
            progress: 0,
            bytes_transferred: 0,
            total_bytes: None,
            error: None,
        };

        self.transfers.write().await.insert(transfer_id.clone(), transfer.clone());

        let transfers = Arc::clone(&self.transfers);
        let tid = transfer_id.clone();
        let server = server_addr.to_string();
        let local = local_path.to_string();
        let remote = remote_filename.to_string();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            match Self::do_download(&server, &remote, &local).await {
                Ok(bytes) => {
                    info!("Download completed: {} bytes", bytes);
                    if let Some(t) = transfers.write().await.get_mut(&tid) {
                        t.state = "completed".to_string();
                        t.bytes_transferred = bytes as u64;
                        t.progress = 100;
                    }
                    if let Some(ref tx) = tx {
                        let _ = tx.send(UiData::Log(Module::Tftpc, format!("Downloaded {} bytes", bytes)));
                    }
                }
                Err(e) => {
                    error!("Download failed: {}", e);
                    if let Some(t) = transfers.write().await.get_mut(&tid) {
                        t.state = "error".to_string();
                        t.error = Some(e.to_string());
                    }
                }
            }
        });

        info!("Starting download: {}@{} -> {}", remote_filename, server_addr, local_path);
        Ok(transfer_id)
    }

    async fn do_upload(server: &str, local: &str, remote: &str) -> Result<u64> {
        let local_file = PathBuf::from(local);
        if !local_file.exists() {
            return Err(ServiceError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Local file not found",
            )));
        }

        let mut file = tokio::fs::File::open(local_file).await?;
        let metadata = file.metadata().await?;
        let file_size = metadata.len();

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(server).await?;

        let mut filename = remote.as_bytes().to_vec();
        filename.push(0);
        let mode = b"octet";
        let mode_len = mode.len();

        let mut packet = Vec::new();
        packet.extend_from_slice(&TFTP_OPCODE_WRQ.to_be_bytes());
        packet.extend_from_slice(&filename);
        packet.extend_from_slice(mode);

        socket.send(&packet).await?;

        let mut buf = [0u8; 516];
        let (bytes_read, _) = socket.recv_from(&mut buf).await?;
        if bytes_read < 4 {
            return Err(ServiceError::Other("Invalid response".to_string()));
        }

        let opcode = u16::from_be_bytes([buf[0], buf[1]]);
        if opcode != TFTP_OPCODE_ACK {
            return Err(ServiceError::Other("Expected ACK packet".to_string()));
        }

        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content).await?;

        let mut block_num = 1u16;
        let mut offset = 0;

        while offset < file_content.len() {
            let chunk = &file_content[offset..std::cmp::min(offset + TFTP_BLOCK_SIZE, file_content.len())];
            
            let mut packet = Vec::new();
            packet.extend_from_slice(&TFTP_OPCODE_DATA.to_be_bytes());
            packet.extend_from_slice(&block_num.to_be_bytes());
            packet.extend_from_slice(chunk);

            socket.send(&packet).await?;

            let mut ack_buf = [0u8; 4];
            socket.recv_from(&mut ack_buf).await?;

            let ack_block = u16::from_be_bytes([ack_buf[2], ack_buf[3]]);
            if ack_block != block_num {
                return Err(ServiceError::Other("Block number mismatch".to_string()));
            }

            block_num = block_num.saturating_add(1);
            offset += TFTP_BLOCK_SIZE;
        }

        Ok(file_size)
    }

    async fn do_download(server: &str, remote: &str, local: &str) -> Result<u64> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(server).await?;

        let mut filename = remote.as_bytes().to_vec();
        filename.push(0);
        let mode = b"octet";

        let mut packet = Vec::new();
        packet.extend_from_slice(&TFTP_OPCODE_RRQ.to_be_bytes());
        packet.extend_from_slice(&filename);
        packet.extend_from_slice(mode);

        socket.send(&packet).await?;

        let mut file_data = Vec::new();
        let mut block_num = 0u16;
        let mut retries = 3;

        while retries > 0 {
            let mut buf = [0u8; 516];
            match socket.recv_from(&mut buf).await {
                Ok((bytes_read, _)) if bytes_read >= 4 => {
                    let opcode = u16::from_be_bytes([buf[0], buf[1]]);
                    if opcode == TFTP_OPCODE_DATA {
                        let recv_block = u16::from_be_bytes([buf[2], buf[3]]);
                        if recv_block == block_num.saturating_add(1) {
                            block_num = recv_block;
                            file_data.extend_from_slice(&buf[4..bytes_read]);

                            let mut ack = Vec::new();
                            ack.extend_from_slice(&TFTP_OPCODE_ACK.to_be_bytes());
                            ack.extend_from_slice(&block_num.to_be_bytes());
                            socket.send(&ack).await?;

                            if bytes_read < 516 {
                                break;
                            }
                            retries = 3;
                        }
                    } else if opcode == TFTP_OPCODE_ERROR {
                        let error_msg = String::from_utf8_lossy(&buf[4..bytes_read]).to_string();
                        return Err(ServiceError::Other(error_msg));
                    }
                }
                Ok(_) => {
                    retries -= 1;
                }
                Err(e) => {
                    retries -= 1;
                    if retries == 0 {
                        return Err(ServiceError::Io(e));
                    }
                }
            }
        }

        let local_path = PathBuf::from(local);
        let mut file = tokio::fs::File::create(local_path).await?;
        file.write_all(&file_data).await?;

        Ok(file_data.len() as u64)
    }

    pub async fn get_transfer(&self, id: &str) -> Option<TftpTransfer> {
        self.transfers.read().await.get(id).cloned()
    }

    pub async fn get_active_transfers(&self) -> Vec<TftpTransfer> {
        self.transfers.read().await
            .values()
            .filter(|t| t.state == "transferring")
            .cloned()
            .collect()
    }
}

impl Default for TftpcService {
    fn default() -> Self {
        Self::new()
    }
}