//! IP Scanner Service

use crate::{Result, ServiceError};
use rabbit_models::scan::{ScanRange, ScanResult, ScannerConfig, ScannerState};
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use tracing::{error, info};

/// IP Scanner service
pub struct ScanService {
    config: Arc<RwLock<ScannerConfig>>,
    state: Arc<RwLock<ScannerState>>,
    results: Arc<RwLock<Vec<ScanResult>>>,
    cancel_tx: Option<mpsc::Sender<()>>,
    scan_handle: Option<JoinHandle<()>>,
}

impl ScanService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ScannerConfig::default())),
            state: Arc::new(RwLock::new(ScannerState::Idle)),
            results: Arc::new(RwLock::new(Vec::new())),
            cancel_tx: None,
            scan_handle: None,
        }
    }

    /// Initialize with configuration
    pub async fn init(&mut self, config: ScannerConfig) -> Result<()> {
        *self.config.write().await = config;
        info!("Scan service initialized");
        Ok(())
    }

    /// Start scanning a range
    pub async fn scan(&mut self, range: ScanRange) -> Result<()> {
        let mut state = self.state.write().await;
        if *state != ScannerState::Idle {
            return Err(ServiceError::AlreadyRunning);
        }
        *state = ScannerState::Scanning { progress: 0 };
        drop(state);

        // Clear previous results
        self.results.write().await.clear();

        let (cancel_tx, mut cancel_rx) = mpsc::channel(1);
        self.cancel_tx = Some(cancel_tx);

        let config = self.config.read().await.clone();
        let state = Arc::clone(&self.state);
        let results = Arc::clone(&self.results);

        let handle = tokio::spawn(async move {
            let ips = calculate_ip_range(range.start, range.end);
            let total = ips.len();
            let mut completed = 0;

            // Process in batches for concurrency control
            for chunk in ips.chunks(config.concurrent) {
                let mut tasks = Vec::new();

                for &ip in chunk {
                    let port = range.port;
                    let timeout_ms = config.timeout_ms;

                    let task = tokio::spawn(async move {
                        scan_host(ip, port, timeout_ms).await
                    });

                    tasks.push((ip, task));
                }

                // Wait for batch to complete
                for (ip, task) in tasks {
                    match task.await {
                        Ok(result) => {
                            results.write().await.push(result);
                        }
                        Err(e) => {
                            error!("Scan task error for {}: {}", ip, e);
                        }
                    }

                    completed += 1;
                    let progress = ((completed as f64 / total as f64) * 100.0) as u8;
                    *state.write().await = ScannerState::Scanning { progress };
                }

                // Check for cancellation
                if cancel_rx.try_recv().is_ok() {
                    *state.write().await = ScannerState::Cancelled;
                    return;
                }
            }

            *state.write().await = ScannerState::Completed;
        });

        self.scan_handle = Some(handle);
        Ok(())
    }

    /// Cancel ongoing scan
    pub async fn cancel(&mut self) -> Result<()> {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(handle) = self.scan_handle.take() {
            let _ = handle.await;
        }

        *self.state.write().await = ScannerState::Cancelled;
        info!("Scan cancelled");
        Ok(())
    }

    /// Get current state
    pub async fn get_state(&self) -> ScannerState {
        *self.state.read().await
    }

    /// Get scan results
    pub async fn get_results(&self) -> Vec<ScanResult> {
        self.results.read().await.clone()
    }

    /// Get only online hosts
    pub async fn get_online_hosts(&self) -> Vec<ScanResult> {
        self.results.read().await
            .iter()
            .filter(|r| r.online)
            .cloned()
            .collect()
    }

    /// Clear results
    pub async fn clear_results(&self) {
        self.results.write().await.clear();
        *self.state.write().await = ScannerState::Idle;
    }

    /// Get scan progress
    pub async fn get_progress(&self) -> Option<ScanProgress> {
        let state = *self.state.read().await;
        let results = self.results.read().await;
        let found_hosts = results.iter().filter(|r| r.online).count();

        match state {
            ScannerState::Scanning { progress } => Some(ScanProgress {
                percentage: progress,
                found_hosts,
                is_scanning: true,
            }),
            ScannerState::Completed => Some(ScanProgress {
                percentage: 100,
                found_hosts,
                is_scanning: false,
            }),
            _ => None,
        }
    }

    /// Get last scan results
    pub async fn get_last_results(&self) -> Option<Vec<ScanResult>> {
        let results = self.results.read().await;
        if results.is_empty() {
            None
        } else {
            Some(results.clone())
        }
    }
}

/// Scan progress information
#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub percentage: u8,
    pub found_hosts: usize,
    pub is_scanning: bool,
}

impl Default for ScanService {
    fn default() -> Self {
        Self::new()
    }
}

/// Scan a single host
async fn scan_host(ip: Ipv4Addr, port: u16, timeout_ms: u64) -> ScanResult {
    let start = tokio::time::Instant::now();

    // Try ICMP ping first (port 0 means ping)
    let online = if port == 0 {
        ping_host(ip, timeout_ms).await
    } else {
        // Try TCP connect
        let addr = format!("{}:{}", ip, port);
        matches!(
            timeout(Duration::from_millis(timeout_ms), tokio::net::TcpStream::connect(&addr)).await,
            Ok(Ok(_))
        )
    };

    let response_time_ms = if online {
        Some(start.elapsed().as_secs_f64() * 1000.0)
    } else {
        None
    };

    // Try to resolve hostname
    let hostname = if online {
        resolve_hostname(ip).await
    } else {
        None
    };

    ScanResult {
        ip,
        online,
        hostname,
        response_time_ms,
        open_ports: Vec::new(),
    }
}

/// Ping a host using ICMP
async fn ping_host(ip: Ipv4Addr, timeout_ms: u64) -> bool {
    // Use surge-ping or platform-specific ping
    // This is a placeholder implementation
    // In production, use rabbit_platform::ping::PlatformPing

    // Fallback: try TCP connect to common ports
    let common_ports = [80, 443, 22, 3389];

    for port in &common_ports {
        let addr = format!("{}:{}", ip, port);
        if let Ok(Ok(_)) = timeout(
            Duration::from_millis(timeout_ms / common_ports.len() as u64),
            tokio::net::TcpStream::connect(&addr)
        ).await {
            return true;
        }
    }

    false
}

/// Resolve hostname from IP
async fn resolve_hostname(_ip: Ipv4Addr) -> Option<String> {
    // Use DNS reverse lookup
    // This is a placeholder
    None
}

/// Calculate IP range
fn calculate_ip_range(start: Ipv4Addr, end: Ipv4Addr) -> Vec<Ipv4Addr> {
    let start_u32 = u32::from(start);
    let end_u32 = u32::from(end);

    (start_u32..=end_u32)
        .map(Ipv4Addr::from)
        .collect()
}
