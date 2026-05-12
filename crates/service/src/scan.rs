//! IP Scanner Service — ICMP-based host discovery

use crate::{
    ui_channel::{Module, UiData},
    Result, ServiceError, ServiceUpdateResult,
};
pub use schema::scan::ScanRange;
use schema::NetworkProvider;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Instant;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use socket2::Type as SockType;
use rand::random;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::Duration;
use tracing::{error, info};

/// Internal Scanner configuration
#[derive(Debug, Clone)]
struct ScannerConfig {
    pub timeout_ms: u64,
    pub concurrent: usize,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self { timeout_ms: 500, concurrent: 256 }
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
    pub hostname: Option<String>,
    pub mac_address: Option<String>,
    /// Diagnostic reason when mac_address is None (shown in scan output)
    pub mac_unavailable_reason: Option<String>,
}

/// IP Scanner service
pub struct ScanService {
    config: Arc<RwLock<ScannerConfig>>,
    state: Arc<RwLock<ScannerState>>,
    results: Arc<RwLock<Vec<ScanResult>>>,
    cancel_tx: Option<mpsc::Sender<()>>,
    scan_handle: Option<JoinHandle<()>>,
    tx: Option<mpsc::Sender<UiData>>,
    network_provider: Option<Arc<dyn NetworkProvider>>,
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
            network_provider: None,
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
            network_provider: None,
        }
    }

    /// Attach a [`NetworkProvider`] for MAC address resolution.
    pub fn with_network(mut self, provider: Arc<dyn NetworkProvider>) -> Self {
        self.network_provider = Some(provider);
        self
    }

    pub async fn send(&self, data: UiData) {
        crate::send_ui(&self.tx, data).await;
    }

    fn load_range_from_config(&self) -> Option<ScanRange> {
        let config = rabbit_config::load_config().ok()?;

        let start_ip = config.modules.get_string("scan", schema::config::keys::scan::START_IP)?;
        let end_ip = config.modules.get_string("scan", schema::config::keys::scan::END_IP)?;

        if start_ip.is_empty() || end_ip.is_empty() {
            return None;
        }

        let start: Ipv4Addr = start_ip.parse().ok()?;

        // 支持简写格式: 1 表示 .1
        let end: Ipv4Addr = if let Ok(num) = end_ip.parse::<u8>() {
            let parts: Vec<&str> = start_ip.splitn(5, '.').collect();
            if parts.len() == 4 {
                let a: u8 = parts[0].parse().unwrap_or(192);
                let b: u8 = parts[1].parse().unwrap_or(168);
                let c: u8 = parts[2].parse().unwrap_or(1);
                Ipv4Addr::new(a, b, c, num)
            } else {
                Ipv4Addr::new(192, 168, 1, num)
            }
        } else {
            end_ip.parse().ok()?
        };

        Some(ScanRange::new(start, end))
    }

    /// 内部启动 Scan 服务
    async fn start(&mut self) -> Result<()> {
        let range = match self.load_range_from_config() {
            Some(r) => r,
            None => {
                return Err(ServiceError::Other("Invalid scan configuration".to_string()));
            }
        };

        let mut state = self.state.write().await;
        if let ScannerState::Scanning { .. } = *state {
            return Err(ServiceError::AlreadyRunning);
        }
        *state = ScannerState::Scanning { progress: 0 };
        drop(state);

        if let Some(ref tx) = self.tx {
            let _ = tx.send(UiData::ServiceStatus(Module::Scan, true, None)).await;
            let _ = tx.send(UiData::ScanProgress("Starting scan...".to_string())).await;
        }

        self.results.write().await.clear();

        // Create multiple ICMP clients (raw sockets) to parallelize
        // kernel-level ICMP packet processing.  A single client/socket
        // serialises all sends through one kernel queue, creating a
        // bottleneck on large subnets.  Each additional socket gives the
        // kernel another parallel path to process route-lookup + ARP +
        // transmit, making results appear truly concurrent.
        // (requires CAP_NET_RAW or root on Linux)
        const POOL_SIZE: usize = 8;
        let mut icmp_clients = Vec::with_capacity(POOL_SIZE);
        for _ in 0..POOL_SIZE {
            match Client::new(
                &Config::builder().sock_type_hint(SockType::RAW).build()
            ) {
                Ok(c) => icmp_clients.push(Arc::new(c)),
                Err(e) => {
                    return Err(ServiceError::Other(format!(
                        "Failed to create ICMP client (need root or CAP_NET_RAW): {}", e
                    )));
                }
            }
        }

        let (cancel_tx, mut cancel_rx) = mpsc::channel(1);
        self.cancel_tx = Some(cancel_tx);

        let config = self.config.read().await.clone();
        let state = Arc::clone(&self.state);
        let results = Arc::clone(&self.results);
        let tx = self.tx.clone();
        let network_provider = self.network_provider.clone();

        let handle = tokio::spawn(async move {
            let ips = calculate_ip_range(range.start, range.end);
            let total = ips.len();
            let semaphore = Arc::new(Semaphore::new(config.concurrent));
            let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut join_set = tokio::task::JoinSet::new();

            // Build IP→MAC table for local interfaces so that the
            // scanner's own IP(s) resolve without ARP table queries.
            let local_macs = match &network_provider {
                Some(p) => {
                    let p = Arc::clone(p);
                    tokio::task::spawn_blocking(move || p.get_local_mac_table())
                        .await
                        .unwrap_or_default()
                }
                None => HashMap::new(),
            };
            let local_macs = Arc::new(local_macs);

            for (i, ip) in ips.into_iter().enumerate() {
                let timeout_ms = config.timeout_ms;
                let permit = semaphore.clone();
                let completed_count = completed.clone();
                let results_clone = results.clone();
                let state_clone = state.clone();
                let tx_clone = tx.clone();
                let provider = network_provider.clone();
                let client = Arc::clone(&icmp_clients[i % POOL_SIZE]);
                let local_macs = Arc::clone(&local_macs);

                join_set.spawn(async move {
                    let _permit = permit.acquire().await.unwrap();

                    let online = icmp_ping_host(&client, ip, timeout_ms).await;
                    let result = ScanResult { ip, hostname: None, mac_address: None, mac_unavailable_reason: None };

                    results_clone.write().await.push(result);
                    let done = completed_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    let progress = ((done as f64 / total as f64) * 100.0) as u8;
                    *state_clone.write().await = ScannerState::Scanning { progress };

                    if online {
                        let bg_results = results_clone.clone();
                        let bg_tx = tx_clone.clone();
                        let bg_provider = provider.clone();
                        let bg_local_macs = local_macs.clone();
                        tokio::spawn(async move {
                            let t0 = Instant::now();

                            // ── MAC (near-instant: local table or ioctl) ──
                            let (mac, reason) = if let Some(m) = bg_local_macs.get(&ip) {
                                (Some(m.clone()), None)
                            } else {
                                match &bg_provider {
                                    Some(p) => {
                                        let mac = tokio::task::spawn_blocking({
                                            let p = Arc::clone(p);
                                            move || p.get_mac_from_arp(ip)
                                        })
                                        .await
                                        .unwrap_or_default();
                                        let r = mac.as_ref().map(|_| None)
                                            .unwrap_or_else(|| Some("MAC not found (no ARP entry)".into()));
                                        (mac, r)
                                    }
                                    None => (None, Some("no ARP provider available".into())),
                                }
                            };
                            let t1 = Instant::now();

                            // ── DNS with 200ms timeout ─────────────────
                            //
                            // glibc's getnameinfo() uses the system
                            // resolver with default 5s per-NS timeout × 2
                            // retries – can take 10s+ for IPs without PTR.
                            // We cap it here so scan output is never
                            // blocked for more than 200ms per host.
                            let hostname = tokio::time::timeout(
                                Duration::from_millis(200),
                                resolve_hostname(ip),
                            )
                            .await
                            .ok()
                            .flatten();
                            let t2 = Instant::now();

                            // ── Output ─────────────────────────────────
                            let host_info = hostname.as_ref()
                                .map(|h| format!(" ({})", h))
                                .unwrap_or_default();
                            let mac_info = mac.as_ref()
                                .map(|m| format!(" [MAC: {}]", m))
                                .unwrap_or_default();
                            let diag = reason.as_ref()
                                .map(|r| format!(" — MAC: {}", r))
                                .unwrap_or_default();
                            let full_msg = format!(
                                "Found online host: {}{}{}{}",
                                ip, host_info, mac_info, diag,
                            );
                            if let Some(ref tx) = bg_tx {
                                let _ = tx.send(UiData::Log(Module::Scan, full_msg)).await;
                            }

                            info!(
                                "scan_timing: {} mac={}us dns={}us total={}us hostname={}",
                                ip,
                                t1.duration_since(t0).as_micros(),
                                t2.duration_since(t1).as_micros(),
                                t2.duration_since(t0).as_micros(),
                                hostname.as_deref().unwrap_or("(none)"),
                            );

                            // ── Update result entry ────────────────────
                            let mut r = bg_results.write().await;
                            if let Some(entry) = r.iter_mut().find(|e| e.ip == ip) {
                                entry.mac_address = mac;
                                entry.mac_unavailable_reason = reason;
                                entry.hostname = hostname;
                            }
                        });
                    }
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
                // 停止原因：扫描完成
                let _ = tx.send(UiData::ServiceStatus(Module::Scan, false, Some("completed".into()))).await;
                let _ = tx.send(UiData::Log(Module::Scan, "Scan finished.".to_string())).await;
            }
        });

        self.scan_handle = Some(handle);
        Ok(())
    }

    pub async fn update(&mut self) -> ServiceUpdateResult {
        let state = *self.state.read().await;
        match state {
            ScannerState::Idle | ScannerState::Completed | ScannerState::Cancelled => {
                match self.start().await {
                    Ok(()) => ServiceUpdateResult::Started("Scan started".to_string()),
                    Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
                }
            }
            ScannerState::Scanning { .. } => match self.cancel().await {
                Ok(()) => ServiceUpdateResult::Stopped("Scan cancelled".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to cancel: {}", e)),
            },
        }
    }

    /// 程序退出时调用，销毁资源，不发状态通告
    pub async fn destroy(&mut self) -> Result<()> {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(()).await;
        }
        if let Some(handle) = self.scan_handle.take() {
            let _ = handle.await;
        }
        *self.state.write().await = ScannerState::Idle;
        info!("Scan service destroyed");
        Ok(())
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

#[async_trait::async_trait]
impl crate::Service for ScanService {
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

impl Default for ScanService {
    fn default() -> Self {
        Self::new()
    }
}

/// ICMP ping a single host. Returns `true` if echo reply received.
async fn icmp_ping_host(client: &Client, ip: Ipv4Addr, timeout_ms: u64) -> bool {
    let mut pinger = client.pinger(IpAddr::V4(ip), PingIdentifier(random())).await;
    pinger.timeout(Duration::from_millis(timeout_ms));
    pinger.ping(PingSequence(0), &[]).await.is_ok()
}

/// Resolve hostname from IP
async fn resolve_hostname(ip: Ipv4Addr) -> Option<String> {
    let ip_addr = IpAddr::V4(ip);

    // Perform reverse DNS lookup (blocking operation)
    tokio::task::spawn_blocking(move || match dns_lookup::lookup_addr(&ip_addr) {
        Ok(name) if !name.is_empty() => Some(name),
        _ => None,
    })
    .await
    .ok()
    .flatten()
}

/// Calculate IP range
fn calculate_ip_range(start: Ipv4Addr, end: Ipv4Addr) -> Vec<Ipv4Addr> {
    let start_u32 = u32::from(start);
    let end_u32 = u32::from(end);

    (start_u32..=end_u32).map(Ipv4Addr::from).collect()
}
