//! TFTP Server Service

use crate::{Result, ServiceError, ServiceUpdateResult};
use rabbit_models::tftp::{TftpLogEntry, TftpServerConfig, TftpTransfer};
use rabbit_platform::config::load_config;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// TFTP Server Service
pub struct TftpdService {
    config: Arc<RwLock<TftpServerConfig>>,
    transfers: Arc<RwLock<HashMap<String, TftpTransfer>>>,
    logs: Arc<RwLock<Vec<TftpLogEntry>>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    server_handle: Option<JoinHandle<()>>,
}

impl TftpdService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(TftpServerConfig::default())),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            server_handle: None,
            shutdown_tx: None,
        }
    }

    /// Initialize from ModuleConfigs
    pub async fn init(&mut self) -> Result<()> {
        let config = load_config()?;
        let server_config = TftpServerConfig::from(&config.modules);
        *self.config.write().await = server_config;
        info!("TFTP server service initialized");
        Ok(())
    }

    /// Start TFTP server
    pub async fn start(&mut self) -> Result<()> {
        self.start_server().await
    }

    /// Start TFTP server
    pub async fn start_server(&mut self) -> Result<()> {
        if self.server_handle.is_some() {
            return Err(ServiceError::AlreadyRunning);
        }

        let config = self.config.read().await.clone();
        if !config.enabled {
            return Ok(());
        }

        let root = PathBuf::from(&config.root_path);
        if !root.exists() {
            std::fs::create_dir_all(&root)?;
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let transfers = Arc::clone(&self.transfers);
        let logs = Arc::clone(&self.logs);
        let bind_addr = config.bind_addr.clone();
        let root_path = config.root_path.clone();
        let timeout_secs = config.timeout_secs;
        let block_size = config.block_size;
        let window_size = config.window_size;

        let handle = tokio::task::spawn_blocking(move || {
            info!("Starting TFTP server on {} with root: {}", bind_addr, root_path);
            
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime");
            
            rt.block_on(async {
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
    pub async fn stop(&mut self) -> Result<()> {
        self.stop_server().await
    }

    /// Stop TFTP server
    pub async fn stop_server(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }

        info!("TFTP server stopped");
        Ok(())
    }

    /// Update service (toggle start/stop)
    pub async fn update(&mut self) -> ServiceUpdateResult {
        if self.is_running() {
            match self.stop().await {
                Ok(()) => ServiceUpdateResult::Stopped("TFTP server stopped".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to stop: {}", e)),
            }
        } else {
            match self.start().await {
                Ok(()) => ServiceUpdateResult::Started("TFTP server started".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
            }
        }
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.server_handle.is_some()
    }

    /// Update config
    pub async fn update_config(&mut self, config: TftpServerConfig) -> Result<()> {
        let was_running = self.server_handle.is_some();
        if was_running {
            self.stop().await?;
        }
        *self.config.write().await = config;
        if was_running {
            self.start().await?;
        }
        Ok(())
    }

    /// Get logs
    pub async fn get_logs(&self) -> Vec<TftpLogEntry> {
        self.logs.read().await.clone()
    }
}

impl Default for TftpdService {
    fn default() -> Self {
        Self::new()
    }
}