//! TFTP Service

use crate::{Result, ServiceError};
use rabbit_models::tftp::{TftpClientConfig, TftpLogEntry, TftpServerConfig, TftpTransfer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tracing::{error, info};

const TFTP_OPCODE_RRQ: u16 = 1;
const TFTP_OPCODE_WRQ: u16 = 2;
const TFTP_OPCODE_DATA: u16 = 3;
const TFTP_OPCODE_ACK: u16 = 4;
const TFTP_OPCODE_ERROR: u16 = 5;
const TFTP_BLOCK_SIZE: usize = 512;
const TFTP_TIMEOUT_SECS: u64 = 5;

/// TFTP service managing both server and client
pub struct TftpService {
    server_config: Arc<RwLock<TftpServerConfig>>,
    client_config: Arc<RwLock<TftpClientConfig>>,
    transfers: Arc<RwLock<HashMap<String, TftpTransfer>>>,
    logs: Arc<RwLock<Vec<TftpLogEntry>>>,
    server_shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl TftpService {
    pub fn new() -> Self {
        Self {
            server_config: Arc::new(RwLock::new(Self::placeholder_server_config())),
            client_config: Arc::new(RwLock::new(Self::placeholder_client_config())),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            server_shutdown: None,
            server_handle: None,
        }
    }

    fn placeholder_server_config() -> TftpServerConfig {
        TftpServerConfig {
            enabled: false,
            bind_addr: String::new(),
            root_path: String::new(),
            block_size: 512,
            timeout_secs: 5,
            window_size: 1,
            allow_overwrite: false,
        }
    }

    fn placeholder_client_config() -> TftpClientConfig {
        TftpClientConfig {
            server_addr: String::new(),
            local_port: 0,
            block_size: 1024,
            timeout_secs: 5,
        }
    }

    /// Initialize with configurations
    pub async fn init(
        &mut self,
        server_config: TftpServerConfig,
        client_config: TftpClientConfig,
    ) -> Result<()> {
        *self.server_config.write().await = server_config;
        *self.client_config.write().await = client_config;
        info!("TFTP service initialized");
        Ok(())
    }

    /// Start TFTP server
    pub async fn start_server(&mut self) -> Result<()> {
        if self.server_handle.is_some() {
            return Err(ServiceError::AlreadyRunning);
        }

        let config = self.server_config.read().await.clone();
        if !config.enabled {
            return Ok(());
        }

        let root = PathBuf::from(&config.root_path);
        if !root.exists() {
            std::fs::create_dir_all(&root)?;
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.server_shutdown = Some(shutdown_tx);

        let transfers = Arc::clone(&self.transfers);
        let logs = Arc::clone(&self.logs);
        let bind_addr = config.bind_addr.clone();
        let root_path = config.root_path.clone();
        let timeout_secs = config.timeout_secs;
        let block_size = config.block_size;
        let window_size = config.window_size;
        let allow_overwrite = config.allow_overwrite;

        let handle = tokio::task::spawn_blocking(move || {
            info!("Starting TFTP server on {} with root: {}", bind_addr, root_path);
            
            // Use async-tftp library to create server with directory handler
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime");
            
            rt.block_on(async {
                // Create the TFTP server with the root directory
                match async_tftp::server::TftpServerBuilder::with_dir_rw(&root_path) {
                    Ok(builder) => {
                        let addr: std::net::SocketAddr = match bind_addr.parse() {
                            Ok(a) => a,
                            Err(e) => {
                                error!("Invalid bind address {}: {}", bind_addr, e);
                                return;
                            }
                        };
                        
                        let builder = builder
                            .bind(addr)
                            .timeout(std::time::Duration::from_secs(timeout_secs))
                            .block_size_limit(block_size as u16)
                            .window_size_limit(window_size as u16)
                            .max_send_retries(100);
                        
                        match builder.build().await {
                            Ok(server) => {
                                info!("TFTP server started successfully");
                                // Run server until shutdown
                                let server_fut = server.serve();
                                tokio::select! {
                                    _ = server_fut => {
                                        info!("TFTP server finished");
                                    }
                                    _ = shutdown_rx => {
                                        info!("TFTP server shutting down");
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to start TFTP server: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to create TFTP server builder: {}", e);
                    }
                }
            });
        });

        self.server_handle = Some(handle);
        info!("TFTP server started");
        Ok(())
    }

    /// Stop TFTP server
    pub async fn stop_server(&mut self) -> Result<()> {
        if let Some(tx) = self.server_shutdown.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }

        info!("TFTP server stopped");
        Ok(())
    }

    /// Upload file (client) to specified server
    pub async fn upload_to(&self, server_addr: &str, local_path: &str, remote_filename: &str) -> Result<String> {
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

        // Spawn actual upload task
        let transfers = Arc::clone(&self.transfers);
        let tid = transfer_id.clone();
        let server = server_addr.to_string();
        let local = local_path.to_string();
        let remote = remote_filename.to_string();

        tokio::spawn(async move {
            match Self::do_upload(&server, &local, &remote).await {
                Ok(bytes) => {
                    info!("Upload completed: {} bytes", bytes);
                    if let Some(t) = transfers.write().await.get_mut(&tid) {
                        t.state = "completed".to_string();
                        t.bytes_transferred = bytes as u64;
                        t.progress = 100;
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

    /// Download file (client) from specified server
    pub async fn download_from(&self, server_addr: &str, remote_filename: &str, local_path: &str) -> Result<String> {
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

        // Spawn actual download task
        let transfers = Arc::clone(&self.transfers);
        let tid = transfer_id.clone();
        let server = server_addr.to_string();
        let remote = remote_filename.to_string();
        let local = local_path.to_string();

        tokio::spawn(async move {
            match Self::do_download(&server, &remote, &local).await {
                Ok(bytes) => {
                    info!("Download completed: {} bytes", bytes);
                    if let Some(t) = transfers.write().await.get_mut(&tid) {
                        t.state = "completed".to_string();
                        t.bytes_transferred = bytes as u64;
                        t.progress = 100;
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

    /// Internal: Perform TFTP upload
    async fn do_upload(server_addr: &str, local_path: &str, remote_filename: &str) -> Result<usize> {
        let addr: SocketAddr = server_addr.parse()
            .map_err(|e| ServiceError::Other(format!("Invalid server address: {}", e)))?;

        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| ServiceError::Io(e))?;

        // Read file
        let mut file = File::open(local_path).await
            .map_err(|e| ServiceError::Io(e))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await
            .map_err(|e| ServiceError::Io(e))?;

        // Send WRQ (Write Request)
        let wrq = Self::create_wrq_packet(remote_filename, "octet");
        socket.send_to(&wrq, addr).await
            .map_err(|e| ServiceError::Io(e))?;

        let mut buf = vec![0u8; 1024];
        let mut block_num: u16 = 0;
        let mut bytes_sent = 0;

        loop {
            // Wait for ACK
            let (len, from) = match timeout(Duration::from_secs(TFTP_TIMEOUT_SECS), socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => (len, from),
                Ok(Err(e)) => return Err(ServiceError::Io(e)),
                Err(_) => return Err(ServiceError::Other("TFTP timeout".to_string())),
            };

            if len < 4 {
                continue;
            }

            let opcode = u16::from_be_bytes([buf[0], buf[1]]);
            let ack_block = u16::from_be_bytes([buf[2], buf[3]]);

            if opcode == TFTP_OPCODE_ERROR {
                let msg = String::from_utf8_lossy(&buf[4..len]);
                return Err(ServiceError::Other(format!("TFTP error: {}", msg)));
            }

            if opcode == TFTP_OPCODE_ACK && ack_block == block_num {
                block_num += 1;

                // Send data block
                let start = (block_num as usize - 1) * TFTP_BLOCK_SIZE;
                let end = (start + TFTP_BLOCK_SIZE).min(buffer.len());
                let data_block = &buffer[start..end];

                let mut data_packet = vec![0u8, 0, (block_num >> 8) as u8, (block_num & 0xff) as u8];
                data_packet.extend_from_slice(data_block);

                socket.send_to(&data_packet, from).await
                    .map_err(|e| ServiceError::Io(e))?;

                bytes_sent += data_block.len();

                // Last block
                if data_block.len() < TFTP_BLOCK_SIZE {
                    break;
                }
            }
        }

        Ok(bytes_sent)
    }

    /// Internal: Perform TFTP download
    async fn do_download(server_addr: &str, remote_filename: &str, local_path: &str) -> Result<usize> {
        let addr: SocketAddr = server_addr.parse()
            .map_err(|e| ServiceError::Other(format!("Invalid server address: {}", e)))?;

        let socket = UdpSocket::bind("0.0.0.0:0").await
            .map_err(|e| ServiceError::Io(e))?;

        // Send RRQ (Read Request)
        let rrq = Self::create_rrq_packet(remote_filename, "octet");
        socket.send_to(&rrq, addr).await
            .map_err(|e| ServiceError::Io(e))?;

        let mut file = File::create(local_path).await
            .map_err(|e| ServiceError::Io(e))?;

        let mut buf = vec![0u8; 1024];
        let mut expected_block: u16 = 1;
        let mut bytes_received = 0;

        loop {
            // Wait for DATA
            let (len, from) = match timeout(Duration::from_secs(TFTP_TIMEOUT_SECS), socket.recv_from(&mut buf)).await {
                Ok(Ok((len, from))) => (len, from),
                Ok(Err(e)) => return Err(ServiceError::Io(e)),
                Err(_) => return Err(ServiceError::Other("TFTP timeout".to_string())),
            };

            if len < 4 {
                continue;
            }

            let opcode = u16::from_be_bytes([buf[0], buf[1]]);
            let block_num = u16::from_be_bytes([buf[2], buf[3]]);

            if opcode == TFTP_OPCODE_ERROR {
                let msg = String::from_utf8_lossy(&buf[4..len]);
                return Err(ServiceError::Other(format!("TFTP error: {}", msg)));
            }

            if opcode == TFTP_OPCODE_DATA && block_num == expected_block {
                let data = &buf[4..len];
                file.write_all(data).await
                    .map_err(|e| ServiceError::Io(e))?;
                bytes_received += data.len();

                // Send ACK
                let ack = vec![0u8, 4, (block_num >> 8) as u8, (block_num & 0xff) as u8];
                socket.send_to(&ack, from).await
                    .map_err(|e| ServiceError::Io(e))?;

                expected_block += 1;

                // Last block
                if data.len() < TFTP_BLOCK_SIZE {
                    break;
                }
            } else {
                // Resend ACK for previous block
                let ack_block = if block_num < expected_block { block_num } else { expected_block - 1 };
                let ack = vec![0u8, 4, (ack_block >> 8) as u8, (ack_block & 0xff) as u8];
                socket.send_to(&ack, from).await.ok();
            }
        }

        Ok(bytes_received)
    }

    /// Create RRQ (Read Request) packet
    fn create_rrq_packet(filename: &str, mode: &str) -> Vec<u8> {
        let mut packet = vec![0u8, 1]; // Opcode 1 = RRQ
        packet.extend_from_slice(filename.as_bytes());
        packet.push(0);
        packet.extend_from_slice(mode.as_bytes());
        packet.push(0);
        packet
    }

    /// Create WRQ (Write Request) packet
    fn create_wrq_packet(filename: &str, mode: &str) -> Vec<u8> {
        let mut packet = vec![0u8, 2]; // Opcode 2 = WRQ
        packet.extend_from_slice(filename.as_bytes());
        packet.push(0);
        packet.extend_from_slice(mode.as_bytes());
        packet.push(0);
        packet
    }

    /// Legacy upload method (uses configured server)
    pub async fn upload(&self, local_path: &str, remote_filename: &str) -> Result<String> {
        let config = self.client_config.read().await.clone();
        self.upload_to(&config.server_addr, local_path, remote_filename).await
    }

    /// Legacy download method (uses configured server)
    pub async fn download(&self, remote_filename: &str, local_path: &str) -> Result<String> {
        let config = self.client_config.read().await.clone();
        self.download_from(&config.server_addr, remote_filename, local_path).await
    }

    /// Get transfer status
    pub async fn get_transfer(&self, id: &str) -> Option<TftpTransfer> {
        self.transfers.read().await.get(id).cloned()
    }

    /// Get all active transfers
    pub async fn get_active_transfers(&self) -> Vec<TftpTransfer> {
        self.transfers.read().await
            .values()
            .filter(|t| t.state == "transferring")
            .cloned()
            .collect()
    }

    /// Get server logs
    pub async fn get_logs(&self) -> Vec<TftpLogEntry> {
        self.logs.read().await.clone()
    }

    /// Check if server is running
    pub fn is_server_running(&self) -> bool {
        self.server_handle.is_some()
    }

    /// Update server configuration
    pub async fn update_server_config(&mut self, config: TftpServerConfig) -> Result<()> {
        let was_running = self.server_handle.is_some();

        if was_running {
            self.stop_server().await?;
        }

        *self.server_config.write().await = config;

        if was_running {
            self.start_server().await?;
        }

        Ok(())
    }

    /// Update client configuration
    pub async fn update_client_config(&mut self, config: TftpClientConfig) -> Result<()> {
        *self.client_config.write().await = config;
        Ok(())
    }
}

impl Default for TftpService {
    fn default() -> Self {
        Self::new()
    }
}
