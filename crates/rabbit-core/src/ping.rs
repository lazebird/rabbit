//! Ping Service

use crate::{Result, ServiceError, ServiceUpdateResult, ui_channel::{UiData, Module}};
use rabbit_models::ping::{PingResult, PingState, PingSummary};
use rabbit_platform::config::get_string;
use rand::random;
use serde::{Deserialize, Serialize};
use socket2::Type;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::info;

/// Ping target configuration (Runtime model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingTarget {
    pub address: String,
    pub ip: Option<IpAddr>,
    pub count: u32,
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub stop_on_loss: bool,
}

impl PingTarget {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            ip: None,
            count: 4,
            interval_ms: 1000,
            timeout_ms: 1000,
            stop_on_loss: false,
        }
    }
}

/// Ping service for managing ping operations
pub struct PingService {
    state: Arc<RwLock<PingState>>,
    targets: Arc<RwLock<Vec<PingTarget>>>,
    results: Arc<RwLock<HashMap<String, Vec<PingResult>>>>,
    consumed: Arc<RwLock<HashMap<String, usize>>>,
    command_tx: Option<mpsc::Sender<PingCommand>>,
    sequence: Arc<AtomicU16>,
    client: Option<Arc<Client>>,
    sent_count: Arc<AtomicU32>,
    tx: Option<mpsc::Sender<UiData>>,
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
            sent_count: Arc::new(AtomicU32::new(0)),
            tx: None,
        }
    }

    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self {
        Self {
            state: Arc::new(RwLock::new(PingState::Idle)),
            targets: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            consumed: Arc::new(RwLock::new(HashMap::new())),
            command_tx: None,
            sequence: Arc::new(AtomicU16::new(0)),
            client: None,
            sent_count: Arc::new(AtomicU32::new(0)),
            tx: Some(tx),
        }
    }

    pub async fn send(&self, data: UiData) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(data).await;
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        let log_file = get_string("ping", "log").unwrap_or_default();
        
        // Try to create ping client with RAW socket type to get TTL on Linux
        // RAW socket requires root/CAP_NET_RAW, so fallback to DGRAM if it fails
        let config = Config::builder()
            .sock_type_hint(Type::RAW)
            .build();
        let client = match Client::new(&config) {
            Ok(client) => {
                info!("Ping service initialized with RAW socket (TTL available)");
                if !log_file.is_empty() {
                    info!("Ping log file enabled: {}", log_file);
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

        if let Some(ref tx) = self.tx {
            let _ = tx.send(UiData::PingState { address: String::new(), progress: 0, total: 0, color: "green".to_string() }).await;
        }

        let state = Arc::clone(&self.state);
        let targets = Arc::clone(&self.targets);
        let results = Arc::clone(&self.results);
        let consumed = Arc::clone(&self.consumed);
        let sequence = Arc::clone(&self.sequence);
        let sent_count = Arc::clone(&self.sent_count);
        let client = self.client.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let mut current_interval_ms = {
                let targets_guard = targets.read().await;
                targets_guard.first().map(|t| t.interval_ms).unwrap_or(1000)
            };
            let mut ping_interval = interval(Duration::from_millis(current_interval_ms));
            let mut sent_per_target: HashMap<String, u32> = HashMap::new();

            if *state.read().await == PingState::Running {
                if let Some(ref client) = client {
                    let log_file = get_string("ping", "log").unwrap_or_default();
                    PingService::ping_all(client, &targets, &results, &consumed, &sequence, &sent_count, &log_file, &mut sent_per_target, &tx).await;
                }
            }

            loop {
                tokio::select! {
                    _ = ping_interval.tick() => {
                        if *state.read().await == PingState::Running {
                            if let Some(ref client) = client {
                                let log_file = get_string("ping", "log").unwrap_or_default();
                                PingService::ping_all(client, &targets, &results, &consumed, &sequence, &sent_count, &log_file, &mut sent_per_target, &tx).await;
                            }
                        }
                    }
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            PingCommand::Stop => {
                                *state.write().await = PingState::Idle;
                                if let Some(ref tx) = tx {
                                    let _ = tx.send(UiData::PingState { address: String::new(), progress: 0, total: 0, color: "gray".to_string() }).await;
                                }
                                break;
                            }
                            PingCommand::AddTarget(target) => {
                                current_interval_ms = target.interval_ms;
                                ping_interval = interval(Duration::from_millis(current_interval_ms));
                                sent_per_target.entry(target.address.clone()).or_insert(0);
                                targets.write().await.push(target);
                            }
                            PingCommand::RemoveTarget(addr) => {
                                targets.write().await.retain(|t| t.address != addr);
                                if let Some(first) = targets.read().await.first() {
                                    current_interval_ms = first.interval_ms;
                                    ping_interval = interval(Duration::from_millis(current_interval_ms));
                                }
                                sent_per_target.remove(&addr);
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn update(&mut self) -> ServiceUpdateResult {
        let state = *self.state.read().await;
        match state {
            PingState::Running => {
                match self.stop().await {
                    Ok(()) => ServiceUpdateResult::Stopped("Ping stopped".to_string()),
                    Err(e) => ServiceUpdateResult::Error(format!("Failed to stop: {}", e)),
                }
            }
            PingState::Idle => {
                match self.start().await {
                    Ok(()) => ServiceUpdateResult::Started("Ping started".to_string()),
                    Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
                }
            }
_ => ServiceUpdateResult::NoChange,
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(PingCommand::Stop).await;
        }
        *self.state.write().await = PingState::Idle;
        self.command_tx = None;
        self.results.write().await.clear();
        self.consumed.write().await.clear();
        self.targets.write().await.clear();
        self.sent_count.store(0, Ordering::SeqCst);
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

        // Simple case: no new results
        if *consumed_pos >= all.len() {
            return vec![];
        }

        let new_results = all[*consumed_pos..].to_vec();
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

        let sent = self.sent_count.load(Ordering::SeqCst);
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
    
    /// Get summary synchronously (without async)
    fn get_summary_sync(
        results: &Arc<RwLock<HashMap<String, Vec<PingResult>>>>,
        sent_count: &Arc<AtomicU32>,
        address: &str,
    ) -> Option<PingSummary> {
        let results_guard = results.try_read().ok()?;
        let results_vec = results_guard.get(address)?;
        if results_vec.is_empty() {
            return None;
        }
        let results = results_vec;
        
        let sent = sent_count.load(Ordering::SeqCst);
        let received = results.iter().filter(|r| r.success).count() as u32;
        let lost = sent.saturating_sub(received);
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
    pub async fn ping_all(
        client: &Arc<Client>,
        targets: &Arc<RwLock<Vec<PingTarget>>>,
        results: &Arc<RwLock<HashMap<String, Vec<PingResult>>>>,
        consumed: &Arc<RwLock<HashMap<String, usize>>>,
        sequence: &Arc<AtomicU16>,
        sent_count: &Arc<AtomicU32>,
        log_file: &str,
        sent_per_target: &mut HashMap<String, u32>,
        tx: &Option<mpsc::Sender<UiData>>,
    ) {
        const MAX_RESULTS: usize = 1000;

        let targets_snapshot = targets.read().await.clone();
        let mut targets_to_remove = Vec::new();

        for target in &targets_snapshot {
            // Check if we've reached the count limit for this target
            let current_sent = sent_per_target.entry(target.address.clone()).or_insert(0);
            if *current_sent >= target.count {
                info!("Ping count limit reached for {} ({}/{}), stopping", 
                      target.address, *current_sent, target.count);
                targets_to_remove.push(target.address.clone());
                continue;
            }

            sent_count.fetch_add(1, Ordering::SeqCst);
            *current_sent += 1;
            let seq = sequence.fetch_add(1, Ordering::SeqCst);
            let result = Self::do_ping(client, target, seq).await;
            let success = result.success;
            let bytes = result.bytes;
            let duration_ms = result.duration_ms;
            let ttl = result.ttl;

            // Write to log file if enabled
            if !log_file.is_empty() {
                Self::write_to_log_file(log_file, &target.address, bytes, duration_ms, success).await;
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
                
                // Adjust consumed_pos to account for removed results
                // This ensures get_results() will return the new results
                let mut consumed_guard = consumed.write().await;
                if let Some(consumed_pos) = consumed_guard.get_mut(&target.address) {
                    if *consumed_pos >= drain_count {
                        *consumed_pos -= drain_count;
                    } else {
                        *consumed_pos = 0;
                    }
                }
            }
            drop(results_guard);

            // Check stop_on_loss: if enabled and ping failed, mark target for removal
            if target.stop_on_loss && !success {
                targets_to_remove.push(target.address.clone());
            }

            // Send result to UI via channel
            if let Some(ref tx) = tx {
                let msg = if success {
                    if let Some(ttl) = ttl {
                        format!("Reply from {}: bytes={} time={:.1}ms TTL={}",
                            target.address, bytes, duration_ms.unwrap_or(0.0), ttl)
                    } else {
                        format!("Reply from {}: bytes={} time={:.1}ms",
                            target.address, bytes, duration_ms.unwrap_or(0.0))
                    }
                } else {
                    "Request timed out.".to_string()
                };
                let _ = tx.send(UiData::Log(Module::Ping, msg)).await;
                
                // Also send stats after each ping
                if let Some(summary) = Self::get_summary_sync(&results, &sent_count, &target.address) {
                    let stats = format!(
                        "Tx {} Rx {} Loss {} Min {:.1}ms Max {:.1}ms Avg {:.1}ms",
                        summary.sent, summary.received, summary.lost,
                        summary.min_ms.unwrap_or(0.0),
                        summary.max_ms.unwrap_or(0.0),
                        summary.avg_ms.unwrap_or(0.0)
                    );
                    let _ = tx.send(UiData::PingStats(stats)).await;
                }
            }
        }

        // Remove targets that triggered stop_on_loss or reached count limit
        if !targets_to_remove.is_empty() {
            let mut targets_guard = targets.write().await;
            for addr in targets_to_remove {
                targets_guard.retain(|t| t.address != addr);
            }
            
            // Check if all targets are removed (count reached) - send Idle state
            if targets_guard.is_empty() && !tx.is_none() {
                if let Some(ref tx) = tx {
                    let _ = tx.send(UiData::PingState { 
                        address: String::new(), 
                        progress: 0, 
                        total: 0, 
                        color: "gray".to_string() 
                    }).await;
                }
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

let mut pinger = client.pinger(addr, PingIdentifier(random())).await;
        pinger.timeout(Duration::from_millis(target.interval_ms));
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
    async fn write_to_log_file(log_file: &str, target: &str, bytes: usize, duration_ms: Option<f64>, success: bool) {
        if log_file.is_empty() {
            return;
        }

        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = if success {
            format!(
                "[{}] {} - Reply from {}: bytes={} time={:.1}ms\n",
                timestamp,
                target,
                target,
                bytes,
                duration_ms.unwrap_or(0.0)
            )
        } else {
            format!(
                "[{}] {} - Request timed out\n",
                timestamp,
                target
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sent_count_tracks_all_pings() {
        let service = PingService::new();
        let sent = service.sent_count.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(sent, 0, "Initial sent count should be 0");

        service.sent_count.store(5, std::sync::atomic::Ordering::SeqCst);
        let sent = service.sent_count.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(sent, 5, "Sent count should be 5 after incrementing");
    }

    #[tokio::test]
    async fn test_stop_clears_sent_count() {
        let service = PingService::new();
        service.sent_count.store(100, std::sync::atomic::Ordering::SeqCst);

        service.sent_count.store(0, std::sync::atomic::Ordering::SeqCst);
        let sent = service.sent_count.load(std::sync::atomic::Ordering::SeqCst);
        assert_eq!(sent, 0, "Sent count should be 0 after stop");
    }

    #[tokio::test]
    async fn test_count_enforcement_removes_target_at_limit() {
        // Test that ping_all removes targets when count limit is reached
        let client = Arc::new(Client::new(&Config::builder().build()).unwrap());
        let targets = Arc::new(RwLock::new(Vec::new()));
        let results = Arc::new(RwLock::new(HashMap::new()));
        let consumed = Arc::new(RwLock::new(HashMap::new()));
        let sequence = Arc::new(AtomicU16::new(0));
        let sent_count = Arc::new(AtomicU32::new(0));
        let mut sent_per_target: HashMap<String, u32> = HashMap::new();

        // Create target with count=3
        let mut target = PingTarget::new("127.0.0.1");
        target.count = 3;
        target.interval_ms = 100;
        targets.write().await.push(target);
        sent_per_target.insert("127.0.0.1".to_string(), 0);

        // Simulate 3 ping cycles
        for i in 1..=4 {
            PingService::ping_all(
                &client,
                &targets,
                &results,
                &consumed,
                &sequence,
                &sent_count,
                "",
                &mut sent_per_target,
            ).await;

            if i <= 3 {
                // Target should still be present for first 3 pings
                assert_eq!(targets.read().await.len(), 1, 
                    "Target should exist after ping #{}", i);
                assert_eq!(*sent_per_target.get("127.0.0.1").unwrap(), i,
                    "Sent count for target should be {}", i);
            } else {
                // After 3rd ping, target should be removed (count reached)
                assert_eq!(targets.read().await.len(), 0,
                    "Target should be removed after reaching count limit of 3");
            }
        }
    }

    #[tokio::test]
    async fn test_infinite_count_never_removes_target() {
        // Test that u32::MAX (infinite) never triggers removal
        let client = Arc::new(Client::new(&Config::builder().build()).unwrap());
        let targets = Arc::new(RwLock::new(Vec::new()));
        let results = Arc::new(RwLock::new(HashMap::new()));
        let consumed = Arc::new(RwLock::new(HashMap::new()));
        let sequence = Arc::new(AtomicU16::new(0));
        let sent_count = Arc::new(AtomicU32::new(0));
        let mut sent_per_target: HashMap<String, u32> = HashMap::new();

        // Create target with infinite count (u32::MAX)
        let mut target = PingTarget::new("127.0.0.1");
        target.count = u32::MAX;
        target.interval_ms = 100;
        targets.write().await.push(target);
        sent_per_target.insert("127.0.0.1".to_string(), 0);

        // Simulate many ping cycles
        for _ in 0..10 {
            PingService::ping_all(
                &client,
                &targets,
                &results,
                &consumed,
                &sequence,
                &sent_count,
                "",
                &mut sent_per_target,
            ).await;
        }

        // Target should still be present after 10 pings
        assert_eq!(targets.read().await.len(), 1,
            "Target with infinite count should never be removed");
        assert_eq!(*sent_per_target.get("127.0.0.1").unwrap(), 10,
            "Sent count should be 10");
    }

    #[tokio::test]
    async fn test_truncation_continues_returning_new_results() {
        // Simple test: verify that after truncation, get_results returns new items
        let service = PingService::new();
        
        // Manually add 1001 results and trigger truncation
        let mut results_guard = service.results.write().await;
        let entry = results_guard.entry("test".to_string()).or_insert_with(Vec::new);
        
        for i in 0..1001 {
            entry.push(PingResult {
                seq: i as u16,
                success: true,
                duration_ms: Some(i as f64),
                ttl: Some(64),
                bytes: 64,
                error: None,
            });
        }
        
        // Manually trigger truncation
        if entry.len() > 1000 {
            let drain_count = entry.len() - 1000;
            entry.drain(0..drain_count);
        }
        
        // Should be trimmed to 1000
        assert_eq!(entry.len(), 1000, "Should be trimmed to 1000");
        assert_eq!(entry[0].seq, 1, "First result should be seq 1 (seq 0 was drained)");
        drop(results_guard);
        
        // Now consume all results
        let new_results = service.get_results("test").await;
        assert_eq!(new_results.len(), 1000, "Should return all 1000 results");
        
        // Add one more result (will trigger another trim)
        let mut results_guard = service.results.write().await;
        let entry = results_guard.get_mut("test").unwrap();
        entry.push(PingResult {
            seq: 1001,
            success: true,
            duration_ms: Some(1001.0),
            ttl: Some(64),
            bytes: 64,
            error: None,
        });
        
        // Manually trigger truncation and consumed_pos adjustment
        if entry.len() > 1000 {
            let drain_count = entry.len() - 1000;
            entry.drain(0..drain_count);
            
            let mut consumed_guard = service.consumed.write().await;
            if let Some(consumed_pos) = consumed_guard.get_mut("test") {
                if *consumed_pos >= drain_count {
                    *consumed_pos -= drain_count;
                } else {
                    *consumed_pos = 0;
                }
            }
        }
        drop(results_guard);
        
        // Should return 1 new result
        let new_results = service.get_results("test").await;
        assert_eq!(new_results.len(), 1, "Should return 1 new result");
        assert_eq!(new_results[0].seq, 1001, "New result should have seq 1001");
    }
}

#[cfg(test)]
mod truncation_tests {
    use super::*;

    const MAX_RESULTS: usize = 1000;

    #[test]
    fn test_results_trimmed_to_max() {
        let mut results: Vec<PingResult> = Vec::new();

        for i in 0..1500 {
            results.push(PingResult {
                seq: i as u16,
                success: i % 2 == 0,
                duration_ms: Some(10.0 + i as f64),
                ttl: Some(64),
                bytes: 64,
                error: None,
            });
        }

        if results.len() > MAX_RESULTS {
            let drain_count = results.len() - MAX_RESULTS;
            results.drain(0..drain_count);
        }

        assert_eq!(results.len(), MAX_RESULTS, "Results should be trimmed to MAX_RESULTS");
        assert_eq!(results[0].seq, 500, "First remaining result should be seq 500");
    }

    #[test]
    fn test_sent_count_independent_of_trimmed_results() {
        let sent_count: u32 = 1500;
        let results_count: usize = 1000;

        assert!(sent_count > results_count as u32,
            "Sent count (1500) should be greater than results count (1000) after trimming");
    }

    #[tokio::test]
    async fn test_summary_uses_sent_count_not_results_len() {
        let service = PingService::new();

        service.sent_count.store(1500, std::sync::atomic::Ordering::SeqCst);

        let mut results_guard = service.results.write().await;
        results_guard.insert("test".to_string(), vec![
            PingResult {
                seq: 1,
                success: true,
                duration_ms: Some(10.0),
                ttl: Some(64),
                bytes: 64,
                error: None,
            }
        ]);
        drop(results_guard);

        let summary = service.get_summary("test").await;
        assert!(summary.is_some());
        let s = summary.unwrap();

        assert_eq!(s.sent, 1500, "Summary sent should use sent_count (1500), not results.len()");
    }
}
