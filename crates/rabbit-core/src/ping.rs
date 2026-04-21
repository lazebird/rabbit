//! Ping Service

use crate::{Result, ServiceError};
use rabbit_models::ping::{PingResult, PingState, PingSummary, PingTarget};
use rand::random;
use socket2::Type;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::info;

/// Ping service for managing ping operations
pub struct PingService {
    state: Arc<RwLock<PingState>>,
    targets: Arc<RwLock<Vec<PingTarget>>>,
    results: Arc<RwLock<HashMap<String, Vec<PingResult>>>>,
    consumed: Arc<RwLock<HashMap<String, usize>>>,
    command_tx: Option<mpsc::Sender<PingCommand>>,
    sequence: Arc<AtomicU16>,
    client: Option<Arc<Client>>,
    /// Log file path for ping results (empty string means no logging)
    log_file: String,
}

#[derive(Debug)]
#[allow(dead_code)]
enum PingCommand {
    Start,
    Stop,
    AddTarget(PingTarget),
    RemoveTarget(String),
}

impl PingService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PingState::Idle)),
            targets: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            consumed: Arc::new(RwLock::new(HashMap::new())),
            command_tx: None,
            sequence: Arc::new(AtomicU16::new(0)),
            client: None,
            log_file: String::new(),
        }
    }

    /// Initialize the service
    pub async fn init(&mut self, log_file: String) -> Result<()> {
        self.log_file = log_file;
        
        // Try to create ping client with RAW socket type to get TTL on Linux
        // RAW socket requires root/CAP_NET_RAW, so fallback to DGRAM if it fails
        let config = Config::builder()
            .sock_type_hint(Type::RAW)
            .build();
        let client = match Client::new(&config) {
            Ok(client) => {
                info!("Ping service initialized with RAW socket (TTL available)");
                if !self.log_file.is_empty() {
                    info!("Ping log file enabled: {}", self.log_file);
                }
                client
            }
            Err(e) => {
                info!("RAW socket failed ({}), falling back to DGRAM socket", e);
                Client::new(&config)
                    .map_err(|e| ServiceError::Other(format!("Failed to create ping client: {}", e)))?
            }
        };
        self.client = Some(Arc::new(client));
        Ok(())
    }

    /// Start the ping service
    pub async fn start(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        if *state != PingState::Idle {
            return Err(ServiceError::AlreadyRunning);
        }
        *state = PingState::Running;
        drop(state);

        let (tx, mut rx) = mpsc::channel(32);
        self.command_tx = Some(tx);

        let state = Arc::clone(&self.state);
        let targets = Arc::clone(&self.targets);
        let results = Arc::clone(&self.results);
        let sequence = Arc::clone(&self.sequence);
        let client = self.client.clone();
        let log_file = self.log_file.clone();

        tokio::spawn(async move {
            let mut ping_interval = interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = ping_interval.tick() => {
                        if *state.read().await == PingState::Running {
                            if let Some(ref client) = client {
                                Self::ping_all(client, &targets, &results, &sequence, &log_file).await;
                            }
                        }
                    }
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            PingCommand::Stop => {
                                *state.write().await = PingState::Idle;
                                break;
                            }
                            PingCommand::AddTarget(target) => {
                                targets.write().await.push(target);
                            }
                            PingCommand::RemoveTarget(addr) => {
                                targets.write().await.retain(|t| t.address != addr);
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        info!("Ping service started");
        Ok(())
    }

    /// Stop the ping service
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(PingCommand::Stop).await;
        }
        *self.state.write().await = PingState::Idle;
        self.command_tx = None;
        // Clear accumulated results and consumed positions
        self.results.write().await.clear();
        self.consumed.write().await.clear();
        self.targets.write().await.clear();
        info!("Ping service stopped");
        Ok(())
    }

    /// Add a ping target
    pub async fn add_target(&self, target: PingTarget) -> Result<()> {
        if let Some(tx) = &self.command_tx {
            tx.send(PingCommand::AddTarget(target)).await
                .map_err(|_| ServiceError::Other("Command channel closed".into()))?;
        } else {
            self.targets.write().await.push(target);
        }
        Ok(())
    }

    /// Remove a ping target
    pub async fn remove_target(&self, address: &str) -> Result<()> {
        if let Some(tx) = &self.command_tx {
            tx.send(PingCommand::RemoveTarget(address.to_string())).await
                .map_err(|_| ServiceError::Other("Command channel closed".into()))?;
        } else {
            self.targets.write().await.retain(|t| t.address != address);
        }
        Ok(())
    }

    /// Get current state
    pub async fn get_state(&self) -> PingState {
        *self.state.read().await
    }

    /// Get all targets
    pub async fn get_targets(&self) -> Vec<PingTarget> {
        self.targets.read().await.clone()
    }

    /// Get new (unconsumed) results for a target since last call
    pub async fn get_results(&self, address: &str) -> Vec<PingResult> {
        let results_guard = self.results.read().await;
        let all = match results_guard.get(address) {
            Some(v) => v,
            None => return vec![],
        };
        let mut consumed_guard = self.consumed.write().await;
        let consumed_pos = consumed_guard.entry(address.to_string()).or_insert(0);

        // After trimming, the actual start index of `all` shifted.
        // We track count of total ever-pushed items via results length + any trimmed.
        // Simpler: just take all items not yet seen (from consumed_pos into `all`)
        let new_results = if *consumed_pos < all.len() {
            all[*consumed_pos..].to_vec()
        } else {
            vec![]
        };
        *consumed_pos = all.len();
        drop(consumed_guard);
        new_results
    }

    /// Get summary for a target (based on all retained results)
    pub async fn get_summary(&self, address: &str) -> Option<PingSummary> {
        let results_guard = self.results.read().await;
        let results = match results_guard.get(address) {
            Some(v) if !v.is_empty() => v,
            _ => return None,
        };

        let sent = results.len() as u32;
        let received = results.iter().filter(|r| r.success).count() as u32;
        let lost = sent - received;
        let loss_rate = if sent > 0 { (lost as f64 / sent as f64) * 100.0 } else { 0.0 };

        let mut min_ms = f64::INFINITY;
        let mut max_ms = f64::NEG_INFINITY;
        let mut sum_ms = 0.0f64;
        let mut time_count = 0u32;
        for r in results.iter() {
            if let Some(ms) = r.duration_ms {
                if ms < min_ms { min_ms = ms; }
                if ms > max_ms { max_ms = ms; }
                sum_ms += ms;
                time_count += 1;
            }
        }

        let (min_opt, max_opt, avg_opt) = if time_count > 0 {
            (Some(min_ms), Some(max_ms), Some(sum_ms / time_count as f64))
        } else {
            (None, None, None)
        };

        Some(PingSummary {
            target: address.to_string(),
            sent,
            received,
            lost,
            loss_rate,
            min_ms: min_opt,
            max_ms: max_opt,
            avg_ms: avg_opt,
        })
    }

    /// Ping all targets
    async fn ping_all(
        client: &Arc<Client>,
        targets: &Arc<RwLock<Vec<PingTarget>>>,
        results: &Arc<RwLock<HashMap<String, Vec<PingResult>>>>,
        sequence: &Arc<AtomicU16>,
        log_file: &str,
    ) {
        const MAX_RESULTS: usize = 1000;

        let targets_snapshot = targets.read().await.clone();
        let mut targets_to_remove = Vec::new();

        for target in &targets_snapshot {
            let seq = sequence.fetch_add(1, Ordering::SeqCst);
            let result = Self::do_ping(client, target, seq).await;
            let success = result.success;

            // Write to log file if enabled
            if !log_file.is_empty() {
                Self::write_to_log_file(log_file, &target.address, &result).await;
            }

            let mut results_guard = results.write().await;
            let entry = results_guard
                .entry(target.address.clone())
                .or_insert_with(Vec::new);
            entry.push(result);
            // Keep only the most recent MAX_RESULTS entries to prevent unbounded growth
            if entry.len() > MAX_RESULTS {
                let drain_count = entry.len() - MAX_RESULTS;
                entry.drain(0..drain_count);
            }
            drop(results_guard);

            // Check stop_on_loss: if enabled and ping failed, mark target for removal
            if target.stop_on_loss && !success {
                targets_to_remove.push(target.address.clone());
            }
        }

        // Remove targets that triggered stop_on_loss
        if !targets_to_remove.is_empty() {
            let mut targets_guard = targets.write().await;
            for addr in targets_to_remove {
                targets_guard.retain(|t| t.address != addr);
            }
        }
    }

    /// Perform a single ping using surge-ping
    async fn do_ping(client: &Arc<Client>, target: &PingTarget, seq: u16) -> PingResult {
        // Parse the target address
        let addr = match Self::resolve_target(target).await {
            Some(ip) => ip,
            None => {
                return PingResult {
                    seq,
                    success: false,
                    duration_ms: None,
                    ttl: None,
                    bytes: 0,
                    error: Some(format!("Failed to resolve: {}", target.address)),
                };
            }
        };

        // Create pinger (pinger() returns Pinger directly, not Result)
        let mut pinger = client.pinger(addr, PingIdentifier(random())).await;

        // Perform ping
        let payload = [0; 56];

        match pinger.ping(PingSequence(seq), &payload).await {
            Ok((packet, duration)) => {
                let duration_ms = duration.as_secs_f64() * 1000.0;
                let (ttl, bytes) = match &packet {
                    surge_ping::IcmpPacket::V4(p) => (p.get_ttl(), p.get_size()),
                    surge_ping::IcmpPacket::V6(p) => (None, 0), // IPv6 doesn't have TTL
                };
                PingResult {
                    seq,
                    success: true,
                    duration_ms: Some(duration_ms),
                    ttl,
                    bytes,
                    error: None,
                }
            }
            Err(e) => {
                PingResult {
                    seq,
                    success: false,
                    duration_ms: None,
                    ttl: None,
                    bytes: 0,
                    error: Some(format!("Ping failed: {}", e)),
                }
            }
        }
    }

    /// Write ping result to log file
    async fn write_to_log_file(log_file: &str, target: &str, result: &PingResult) {
        if log_file.is_empty() {
            return;
        }

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = if result.success {
            format!(
                "[{}] {} - Reply from {}: bytes={} time={:.1}ms TTL={}\n",
                timestamp,
                target,
                target,
                result.bytes,
                result.duration_ms.unwrap_or(0.0),
                result.ttl.unwrap_or(0)
            )
        } else {
            format!(
                "[{}] {} - {}\n",
                timestamp,
                target,
                result.error.as_deref().unwrap_or("Request timed out")
            )
        };

        // Append to log file
        use tokio::fs::OpenOptions;
        use tokio::io::AsyncWriteExt;
        
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)
            .await
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(log_line.as_bytes()).await {
                    tracing::warn!("Failed to write ping log: {}", e);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to open ping log file: {}", e);
            }
        }
    }

    /// Resolve target address to IP
    async fn resolve_target(target: &PingTarget) -> Option<IpAddr> {
        // If already an IP, use it directly
        if let Ok(ip) = target.address.parse::<IpAddr>() {
            return Some(ip);
        }

        // Try to resolve as hostname
        match tokio::net::lookup_host(format!("{}:0", target.address)).await {
            Ok(mut addrs) => addrs.next().map(|addr| addr.ip()),
            Err(_) => None,
        }
    }
}

impl Default for PingService {
    fn default() -> Self {
        Self::new()
    }
}
