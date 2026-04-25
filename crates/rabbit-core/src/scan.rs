//! IP Scanner Service

use crate::{Result, ServiceError, ServiceUpdateResult, ui_channel::{UiData, Module}};
pub use rabbit_models::scan::ScanRange;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};
use tracing::{error, info};

/// Internal Scanner configuration
#[derive(Debug, Clone)]
struct ScannerConfig {
    pub timeout_ms: u64,
    pub concurrent: usize,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 1500,
            concurrent: 256,
        }
    }
}

/// Internal Scanner state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScannerState {
    Idle,
    Scanning { progress: u8 },
    Completed,
    Cancelled,
}

/// Internal Scan result
#[derive(Debug, Clone)]
struct ScanResult {
    pub ip: Ipv4Addr,
    pub online: bool,
    pub hostname: Option<String>,
    pub mac_address: Option<String>,
    pub response_time_ms: Option<f64>,
    pub open_ports: Vec<u16>,
}

/// IP Scanner service
pub struct ScanService {
    config: Arc<RwLock<ScannerConfig>>,
    state: Arc<RwLock<ScannerState>>,
    results: Arc<RwLock<Vec<ScanResult>>>,
    cancel_tx: Option<mpsc::Sender<()>>,
    scan_handle: Option<JoinHandle<()>>,
    tx: Option<mpsc::Sender<UiData>>,
}

impl ScanService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ScannerConfig::default())),
            state: Arc::new(RwLock::new(ScannerState::Idle)),
            results: Arc::new(RwLock::new(Vec::new())),
            cancel_tx: None,
            scan_handle: None,
            tx: None,
        }
    }

    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self {
        Self {
            config: Arc::new(RwLock::new(ScannerConfig::default())),
            state: Arc::new(RwLock::new(ScannerState::Idle)),
            results: Arc::new(RwLock::new(Vec::new())),
            cancel_tx: None,
            scan_handle: None,
            tx: Some(tx),
        }
    }

    pub async fn send(&self, data: UiData) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(data).await;
        }
    }

    /// Initialize the service
    pub async fn init(&mut self) -> Result<()> {
        info!("Scan service initialized");
        Ok(())
    }

    /// Start scanning a range
    pub async fn scan(&mut self, range: ScanRange) -> Result<()> {
        let mut state = self.state.write().await;
        if let ScannerState::Scanning { .. } = *state {
            return Err(ServiceError::AlreadyRunning);
        }
        *state = ScannerState::Scanning { progress: 0 };
        drop(state);

        if let Some(ref tx) = self.tx {
            let _ = tx.send(UiData::ScanProgress("Starting scan...".to_string())).await;
        }

        self.results.write().await.clear();

        let (cancel_tx, mut cancel_rx) = mpsc::channel(1);
        self.cancel_tx = Some(cancel_tx);

        let config = self.config.read().await.clone();
        let state = Arc::clone(&self.state);
        let results = Arc::clone(&self.results);
        let tx = self.tx.clone();

        let handle = tokio::spawn(async move {
            let ips = calculate_ip_range(range.start, range.end);
            let total = ips.len();
            let semaphore = Arc::new(Semaphore::new(config.concurrent));
            let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut join_set = tokio::task::JoinSet::new();

            for ip in ips {
                let port = range.port;
                let timeout_ms = config.timeout_ms;
                let permit = semaphore.clone();
                let completed_count = completed.clone();
                let results_clone = results.clone();
                let state_clone = state.clone();
                let tx_clone = tx.clone();

                join_set.spawn(async move {
                    let _permit = permit.acquire().await.unwrap();
                    let result = scan_host(ip, port, timeout_ms).await;

                    if result.online {
                        if let Some(ref tx) = tx_clone {
                            let msg = format!("Found online host: {}{}", 
                                result.ip, 
                                result.hostname.as_ref().map(|h| format!(" ({})", h)).unwrap_or_default()
                            );
                            let _ = tx.send(UiData::Log(Module::Scan, msg)).await;
                        }
                    }

                    results_clone.write().await.push(result);
                    let done = completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    let progress = ((done as f64 / total as f64) * 100.0) as u8;
                    *state_clone.write().await = ScannerState::Scanning { progress };
                });
            }

            while let Some(res) = join_set.join_next().await {
                if cancel_rx.try_recv().is_ok() {
                    join_set.abort_all();
                    *state.write().await = ScannerState::Cancelled;
                    if let Some(ref tx) = tx {
                        let _ = tx.send(UiData::ScanProgress("Scan cancelled".to_string())).await;
                    }
                    return;
                }
                if let Err(e) = res {
                    error!("Scan task error: {}", e);
                }
            }

            *state.write().await = ScannerState::Completed;
            if let Some(ref tx) = tx {
                let _ = tx.send(UiData::ServiceStatus(Module::Scan, false)).await;
                let _ = tx.send(UiData::Log(Module::Scan, "Scan finished.".to_string())).await;
            }
        });

        self.scan_handle = Some(handle);
        Ok(())
    }

    pub async fn update(&mut self) -> ServiceUpdateResult {
        let state = *self.state.read().await;
        match state {
            ScannerState::Idle => ServiceUpdateResult::NoChange,
            _ => match self.cancel().await {
                Ok(()) => ServiceUpdateResult::Stopped("Scan cancelled".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to cancel: {}", e)),
            }
        }
    }

    pub fn is_running(&self) -> bool {
        if let Ok(s) = self.state.try_read() {
            matches!(*s, ScannerState::Scanning { .. })
        } else {
            false
        }
    }

    pub async fn cancel(&mut self) -> Result<()> {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(()).await;
        }
        if let Some(handle) = self.scan_handle.take() {
            let _ = handle.await;
        }
        *self.state.write().await = ScannerState::Idle;
        info!("Scan cancelled");
        Ok(())
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

/// Scan a single host with parallel port probing
async fn scan_host(ip: Ipv4Addr, port: u16, timeout_ms: u64) -> ScanResult {
    let start = tokio::time::Instant::now();

    // If specific port given, try TCP connect
    // If port == 0, do parallel ping (ICMP-like via multi-port TCP)
    let (online, response_time) = if port == 0 {
        ping_host_parallel(ip, timeout_ms).await
    } else {
        let addr = format!("{}:{}", ip, port);
        let result = timeout(
            Duration::from_millis(timeout_ms),
            tokio::net::TcpStream::connect(&addr)
        ).await;
        let is_online = matches!(&result, Ok(Ok(_)));
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        (is_online, if is_online { Some(elapsed) } else { None })
    };

    // Try to resolve hostname
    let hostname = if online {
        resolve_hostname(ip).await
    } else {
        None
    };

    // Try to get MAC address from ARP table
    let mac_address = if online {
        get_mac_from_arp(ip)
    } else {
        None
    };

    ScanResult {
        ip,
        online,
        hostname,
        mac_address,
        response_time_ms: response_time,
        open_ports: Vec::new(),
    }
}

/// Ping a host using parallel TCP connect attempts to multiple common ports
/// Returns (online, response_time_ms) - response_time is from the fastest successful connection
async fn ping_host_parallel(ip: Ipv4Addr, timeout_ms: u64) -> (bool, Option<f64>) {
    let start = tokio::time::Instant::now();
    let common_ports = [80, 443, 22, 3389, 8080, 8443];
    let overall_timeout = Duration::from_millis(timeout_ms);

    // Create a future for each port connection
    let mut join_set = tokio::task::JoinSet::new();
    for port in common_ports {
        let addr = format!("{}:{}", ip, port);
        join_set.spawn(async move {
            tokio::net::TcpStream::connect(&addr).await
        });
    }

    // Wait for first success or timeout
    let result = tokio::time::timeout(overall_timeout, async {
        while let Some(res) = join_set.join_next().await {
            if let Ok(Ok(_)) = res {
                return true;
            }
        }
        false
    }).await;

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    match result {
        Ok(true) => (true, Some(elapsed)),
        _ => (false, None),
    }
}

/// Resolve hostname from IP
async fn resolve_hostname(ip: Ipv4Addr) -> Option<String> {
    // Use DNS reverse lookup via dns-lookup crate
    use std::net::IpAddr;
    
    let ip_addr = IpAddr::V4(ip);
    
    // Perform reverse DNS lookup (blocking operation)
    tokio::task::spawn_blocking(move || {
        match dns_lookup::lookup_addr(&ip_addr) {
            Ok(name) if !name.is_empty() => Some(name),
            _ => None,
        }
    }).await.ok().flatten()
}

/// Get MAC address from ARP table (Linux: /proc/net/arp)
fn get_mac_from_arp(ip: Ipv4Addr) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let ip_str = ip.to_string();
        if let Ok(content) = std::fs::read_to_string("/proc/net/arp") {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 && parts[0] == ip_str {
                    // parts[3] is the HW type, parts[4] is flags, parts[5] is MAC
                    // Actually on Linux /proc/net/arp: IP address HW type Flags HW address Device
                    if parts.len() >= 4 {
                        let mac = parts[3].to_string();
                        if mac != "00:00:00:00:00:00" {
                            return Some(mac);
                        }
                    }
                }
            }
        }
    }
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
