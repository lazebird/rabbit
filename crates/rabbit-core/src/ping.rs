//! Ping Service

use crate::{Result, ServiceError};
use rabbit_models::ping::{PingResult, PingState, PingSummary, PingTarget};
use rand::random;
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
    command_tx: Option<mpsc::Sender<PingCommand>>,
    sequence: Arc<AtomicU16>,
    client: Option<Arc<Client>>,
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
            command_tx: None,
            sequence: Arc::new(AtomicU16::new(0)),
            client: None,
        }
    }

    /// Initialize the service
    pub async fn init(&mut self) -> Result<()> {
        // Create ping client
        let config = Config::default();
        let client = Client::new(&config)
            .map_err(|e| ServiceError::Other(format!("Failed to create ping client: {}", e)))?;
        self.client = Some(Arc::new(client));
        info!("Ping service initialized");
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

        tokio::spawn(async move {
            let mut ping_interval = interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = ping_interval.tick() => {
                        if *state.read().await == PingState::Running {
                            if let Some(ref client) = client {
                                Self::ping_all(client, &targets, &results, &sequence).await;
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

    /// Get results for a target
    pub async fn get_results(&self, address: &str) -> Vec<PingResult> {
        self.results.read().await
            .get(address)
            .cloned()
            .unwrap_or_default()
    }

    /// Get summary for a target
    pub async fn get_summary(&self, address: &str) -> Option<PingSummary> {
        let results = self.get_results(address).await;
        if results.is_empty() {
            return None;
        }

        let sent = results.len() as u32;
        let received = results.iter().filter(|r| r.success).count() as u32;
        let lost = sent - received;
        let loss_rate = if sent > 0 { (lost as f64 / sent as f64) * 100.0 } else { 0.0 };

        let times: Vec<f64> = results.iter()
            .filter_map(|r| r.duration_ms)
            .collect();

        let (min_ms, max_ms, avg_ms) = if !times.is_empty() {
            let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let avg = times.iter().sum::<f64>() / times.len() as f64;
            (Some(min), Some(max), Some(avg))
        } else {
            (None, None, None)
        };

        Some(PingSummary {
            target: address.to_string(),
            sent,
            received,
            lost,
            loss_rate,
            min_ms,
            max_ms,
            avg_ms,
        })
    }

    /// Ping all targets
    async fn ping_all(
        client: &Arc<Client>,
        targets: &Arc<RwLock<Vec<PingTarget>>>,
        results: &Arc<RwLock<HashMap<String, Vec<PingResult>>>>,
        sequence: &Arc<AtomicU16>,
    ) {
        let targets = targets.read().await.clone();

        for target in targets {
            let seq = sequence.fetch_add(1, Ordering::SeqCst);
            let result = Self::do_ping(client, &target, seq).await;
            results.write().await
                .entry(target.address.clone())
                .or_insert_with(Vec::new)
                .push(result);
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
                    error: Some(format!("Failed to resolve: {}", target.address)),
                };
            }
        };

        // Create pinger (pinger() returns Pinger directly, not Result)
        let mut pinger = client.pinger(addr, PingIdentifier(random())).await;

        // Perform ping
        let payload = [0; 56];

        match pinger.ping(PingSequence(seq), &payload).await {
            Ok((_packet, duration)) => {
                let duration_ms = duration.as_secs_f64() * 1000.0;
                PingResult {
                    seq,
                    success: true,
                    duration_ms: Some(duration_ms),
                    error: None,
                }
            }
            Err(e) => {
                PingResult {
                    seq,
                    success: false,
                    duration_ms: None,
                    error: Some(format!("Ping failed: {}", e)),
                }
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
