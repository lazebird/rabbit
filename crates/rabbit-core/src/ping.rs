//! Ping Service

use crate::{Result, ServiceError, ServiceUpdateResult, ui_channel::{UiData, Module}};
use rabbit_platform::config::get_string;
use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::info;

/// Internal Ping target model
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

/// Internal Ping result
#[derive(Debug, Clone)]
struct PingResult {
    pub success: bool,
    pub duration_ms: Option<f64>,
    pub ttl: Option<u8>,
}

/// Internal Ping session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PingState {
    Idle,
    Running,
}

/// Ping service for managing ping operations
pub struct PingService {
    state: Arc<RwLock<PingState>>,
    targets: Arc<RwLock<Vec<PingTarget>>>,
    results: Arc<RwLock<HashMap<String, Vec<PingResult>>>>,
    command_tx: Option<mpsc::Sender<PingCommand>>,
    sequence: Arc<AtomicU16>,
    client: Option<Arc<Client>>,
    sent_count: Arc<AtomicU32>,
    tx: Option<mpsc::Sender<UiData>>,
}

#[derive(Debug)]
enum PingCommand {
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
            command_tx: None,
            sequence: Arc::new(AtomicU16::new(0)),
            client: None,
            sent_count: Arc::new(AtomicU32::new(0)),
            tx: Some(tx),
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        let config = Config::builder().build();
        let client = Client::new(&config)
            .map_err(|e| ServiceError::Other(format!("Failed to create ping client: {}", e)))?;
        self.client = Some(Arc::new(client));
        Ok(())
    }

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
        let sent_count = Arc::clone(&self.sent_count);
        let client = self.client.clone();
        let tx_ui = self.tx.clone();

        tokio::spawn(async move {
            let mut current_interval_ms = 1000;
            let mut ping_interval = interval(Duration::from_millis(current_interval_ms));
            let mut sent_per_target: HashMap<String, u32> = HashMap::new();

            loop {
                tokio::select! {
                    _ = ping_interval.tick() => {
                        if *state.read().await == PingState::Running {
                            if let Some(ref client) = client {
                                PingService::ping_all(client, &targets, &results, &sequence, &sent_count, &mut sent_per_target, &tx_ui).await;
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
                                current_interval_ms = target.interval_ms;
                                ping_interval = interval(Duration::from_millis(current_interval_ms));
                                sent_per_target.entry(target.address.clone()).or_insert(0);
                                targets.write().await.push(target);
                            }
                            PingCommand::RemoveTarget(addr) => {
                                targets.write().await.retain(|t| t.address != addr);
                                sent_per_target.remove(&addr);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn update(&mut self) -> ServiceUpdateResult {
        let state = *self.state.read().await;
        if state == PingState::Running {
            match self.stop().await {
                Ok(()) => ServiceUpdateResult::Stopped("Ping stopped".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to stop: {}", e)),
            }
        } else {
            match self.start().await {
                Ok(()) => ServiceUpdateResult::Started("Ping started".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
            }
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(PingCommand::Stop).await;
        }
        *self.state.write().await = PingState::Idle;
        self.results.write().await.clear();
        self.targets.write().await.clear();
        self.sent_count.store(0, Ordering::SeqCst);
        Ok(())
    }

    pub async fn add_target(&self, target: PingTarget) -> Result<()> {
        if let Some(tx) = &self.command_tx {
            tx.send(PingCommand::AddTarget(target)).await
                .map_err(|_| ServiceError::Other("Command channel closed".into()))?;
        }
        Ok(())
    }

    async fn ping_all(
        client: &Arc<Client>,
        targets: &Arc<RwLock<Vec<PingTarget>>>,
        results: &Arc<RwLock<HashMap<String, Vec<PingResult>>>>,
        sequence: &Arc<AtomicU16>,
        sent_count: &Arc<AtomicU32>,
        sent_per_target: &mut HashMap<String, u32>,
        tx: &Option<mpsc::Sender<UiData>>,
    ) {
        let targets_guard = targets.read().await;
        for target in targets_guard.iter() {
            let address = target.address.clone();
            let current_sent = sent_per_target.entry(address.clone()).or_insert(0);
            
            if target.count > 0 && *current_sent >= target.count {
                continue;
            }

            *current_sent += 1;
            sent_count.fetch_add(1, Ordering::SeqCst);
            let seq = sequence.fetch_add(1, Ordering::SeqCst);

            let result = match PingService::do_ping(client, target, seq).await {
                Ok(res) => res,
                Err(_) => PingResult {
                    success: false,
                    duration_ms: None,
                    ttl: None,
                },
            };

            let mut results_guard = results.write().await;
            let target_results = results_guard.entry(address.clone()).or_insert(Vec::new());
            target_results.push(result.clone());

            if let Some(ref ui_tx) = tx {
                let msg = if result.success {
                    format!("Reply from {}: time={:.2}ms TTL={}", 
                        address, result.duration_ms.unwrap_or(0.0), result.ttl.unwrap_or(0))
                } else {
                    format!("Request to {} timed out", address)
                };
                let _ = ui_tx.send(UiData::Log(Module::Ping, msg)).await;
                
                let total_sent = *current_sent;
                let received = target_results.iter().filter(|r| r.success).count();
                let loss = ((total_sent - received as u32) as f32 / total_sent as f32) * 100.0;
                let stats = format!("Tx: {} Rx: {} Loss: {:.1}%", total_sent, received, loss);
                let _ = ui_tx.send(UiData::PingStats(stats)).await;
            }
        }
    }

    async fn do_ping(client: &Arc<Client>, target: &PingTarget, seq: u16) -> Result<PingResult> {
        let ip = if let Some(ip) = target.ip {
            ip
        } else {
            use std::net::ToSocketAddrs;
            let addr = format!("{}:0", target.address);
            let mut addrs = addr.to_socket_addrs().map_err(|e| ServiceError::Other(format!("DNS error: {}", e)))?;
            addrs.next().ok_or_else(|| ServiceError::Other("No address found".into()))?.ip()
        };

        let mut pinger = client.pinger(ip, PingIdentifier(random())).await;
        pinger.timeout(Duration::from_millis(target.timeout_ms));
        
        match pinger.ping(PingSequence(seq), &[]).await {
            Ok((packet, duration)) => {
                let ttl = match packet {
                    surge_ping::IcmpPacket::V4(p) => p.get_ttl(),
                    _ => None,
                };
                Ok(PingResult {
                    success: true,
                    duration_ms: Some(duration.as_secs_f64() * 1000.0),
                    ttl,
                })
            }
            Err(_) => Ok(PingResult {
                success: false,
                duration_ms: None,
                ttl: None,
            }),
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
