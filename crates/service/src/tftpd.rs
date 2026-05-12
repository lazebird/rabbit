//! TFTP Server Service

use crate::{
    ui_channel::{Module, UiData},
    Result, ServiceError, ServiceUpdateResult,
};
use rabbit_config::get_integer;
use schema::config::keys;
use std::path::PathBuf;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// TFTP server internal configuration
#[derive(Debug, Clone, Default)]
struct ServerConfig {
    pub bind_addr: String,
    pub root_path: String,
    pub block_size: usize,
    pub timeout_secs: u64,
    pub window_size: u16,
}

impl ServerConfig {
    fn from_platform() -> Self {
        let port = get_integer("tftpd", keys::tftpd::PORT).unwrap_or(69);
        Self {
            bind_addr: format!("0.0.0.0:{}", port),
            root_path: get_array_first("tftpd", keys::tftpd::WORK_DIRS).unwrap_or_else(|| ".".to_string()),
            block_size: get_integer("tftpd", keys::tftpd::BLK_SIZE).unwrap_or(512) as usize,
            timeout_secs: get_integer("tftpd", keys::tftpd::TIMEOUT).unwrap_or(200) as u64 / 1000,
            window_size: 1,
        }
    }
}

fn get_array_first(module: &str, key: &str) -> Option<String> {
    rabbit_config::get_array(module, key).and_then(|arr| arr.first().cloned())
}

/// TFTP Server Service
pub struct TftpdService {
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    server_handle: Option<JoinHandle<()>>,
    tx: Option<tokio::sync::mpsc::Sender<UiData>>,
}

impl TftpdService {
    pub fn new() -> Self {
        Self {
            server_handle: None,
            shutdown_tx: None,
            tx: None,
        }
    }

    pub fn with_channel(tx: tokio::sync::mpsc::Sender<UiData>) -> Self {
        Self {
            server_handle: None,
            shutdown_tx: None,
            tx: Some(tx),
        }
    }

    pub async fn send(&self, data: UiData) {
        crate::send_ui(&self.tx, data).await;
    }

    /// 内部启动 TFTP 服务器
    async fn start(&mut self) -> Result<()> {
        self.start_server().await
    }

    /// 内部启动 TFTP 服务器
    async fn start_server(&mut self) -> Result<()> {
        if self.server_handle.is_some() {
            return Err(ServiceError::AlreadyRunning);
        }

        // Pull configuration directly from platform cache
        let config = ServerConfig::from_platform();

        let root = PathBuf::from(&config.root_path);
        if !root.exists() {
            std::fs::create_dir_all(&root)?;
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let bind_addr = config.bind_addr.clone();
        let root_path = config.root_path.clone();
        let timeout_secs = config.timeout_secs;
        let block_size = config.block_size;
        let window_size = config.window_size;
        let tx_ui = self.tx.clone();

        let handle = tokio::task::spawn_blocking(move || {
            info!("Starting TFTP server on {} with root: {}", bind_addr, root_path);

            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("Failed to create runtime");

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
                            .window_size_limit(window_size)
                            .max_send_retries(100);

                        match builder.build().await {
                            Ok(server) => {
                                info!("TFTP server started successfully");
                                if let Some(ref tx) = tx_ui {
                                    let _ = tx.send(UiData::Log(Module::Tftpd, "TFTP server started".to_string())).await;
                                }

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
        info!("TFTP server handle created");
        Ok(())
    }

    /// Stop TFTP server
    async fn stop(&mut self) -> Result<()> {
        self.stop_server().await
    }

    /// 内部停止 TFTP 服务器
    async fn stop_server(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }

        if let Some(ref tx) = self.tx {
            let _ = tx.send(UiData::Log(Module::Tftpd, "TFTP server stopped".to_string())).await;
        }

        info!("TFTP server stopped");
        Ok(())
    }

    /// 公开接口：启停切换，会发状态通告
    pub async fn update(&mut self) -> ServiceUpdateResult {
        if self.server_handle.is_some() {
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

    /// 程序退出时调用，销毁资源，不发状态通告
    pub async fn destroy(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }
        info!("TFTP server destroyed");
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::Service for TftpdService {
    async fn update(&mut self) -> crate::ServiceUpdateResult {
        self.update().await
    }

    async fn destroy(&mut self) -> crate::Result<()> {
        self.destroy().await
    }

    async fn send(&self, data: crate::UiData) {
        self.send(data).await;
    }
}

impl Default for TftpdService {
    fn default() -> Self {
        Self::new()
    }
}
