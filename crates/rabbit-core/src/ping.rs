//! Ping Service

use crate::{Result, ServiceError, ServiceUpdateResult, ui_channel::{UiData, Module}};
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
use tracing::{info, error};

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
    sent_per_target: Arc<RwLock<HashMap<String, u32>>>,
    tx: Option<mpsc::Sender<UiData>>,
}

#[derive(Debug)]
enum PingCommand {
    Stop,
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
            sent_per_target: Arc::new(RwLock::new(HashMap::new())),
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
            sent_per_target: Arc::new(RwLock::new(HashMap::new())),
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
            error!("Ping service cannot start: service is not Idle");
            return Err(ServiceError::AlreadyRunning);
        }
        *state = PingState::Running;
        
        self.sent_per_target.write().await.clear();
        
        drop(state);

        info!("Creating new ping command channel");
        let (tx, mut rx) = mpsc::channel(32);
        self.command_tx = Some(tx);

        let service = self.clone_for_task();

        tokio::spawn(async move {
            info!("Ping task started");
            
            // 内部自动加载配置并添加目标
            if let Err(e) = service.add_target().await {
                error!("Failed to add target: {}", e);
            }
            
            // 确保任务启动时立即检查 targets
            let mut current_interval_ms = 1000;
            if let Some(first) = service.targets.read().await.first() {
                current_interval_ms = first.interval_ms;
            }
            let mut ping_interval = interval(Duration::from_millis(current_interval_ms));

            loop {
                tokio::select! {
                    _ = ping_interval.tick() => {
                        info!("Ping interval tick");
                        if service.is_running().await {
                            info!("Calling ping_all");
                            service.ping_all().await;
                        } else {
                            info!("Ping service not running, breaking loop");
                            break;
                        }
                    }
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            PingCommand::Stop => {
                                info!("Ping task received Stop command");
                                *service.state.write().await = PingState::Idle;
                                break;
                            }
                        }
                    }
                }
            }
            info!("Ping task loop exited");
        });

        Ok(())
    }

    fn clone_for_task(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            targets: Arc::clone(&self.targets),
            results: Arc::clone(&self.results),
            command_tx: self.command_tx.clone(),
            sequence: Arc::clone(&self.sequence),
            client: self.client.clone(),
            sent_count: Arc::clone(&self.sent_count),
            sent_per_target: Arc::clone(&self.sent_per_target),
            tx: self.tx.clone(),
        }
    }

    pub async fn update(&mut self) -> ServiceUpdateResult {
        if self.is_running().await {
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

    pub async fn is_running(&self) -> bool {
        self.state.read().await.clone() == PingState::Running
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(PingCommand::Stop).await;
        }
        *self.state.write().await = PingState::Idle;
        self.results.write().await.clear();
        self.targets.write().await.clear();
        self.sent_count.store(0, Ordering::SeqCst);
        self.sent_per_target.write().await.clear();
        Ok(())
    }

    fn load_target_from_config(&self) -> Option<PingTarget> {
        let config = rabbit_platform::config::load_config().ok()?;
        let target = config.modules.get_string("ping", "target")?;
        if target.is_empty() {
            return None;
        }
        
        let interval = config.modules.get_integer("ping", "interval").unwrap_or(1000) as u64;
        let count: i32 = config.modules.get_integer("ping", "count").unwrap_or(-1) as i32;
        let stop_on_loss = config.modules.get_bool("ping", "stoponloss").unwrap_or(false);
        
        let mut target_obj = PingTarget::new(&target);
        target_obj.interval_ms = interval;
        target_obj.count = if count < 0 { u32::MAX } else { count as u32 };
        target_obj.stop_on_loss = stop_on_loss;
        
        Some(target_obj)
    }

    pub async fn add_target(&self) -> Result<()> {
        if let Some(target) = self.load_target_from_config() {
            info!("Adding target from config: {}", target.address);
            self.targets.write().await.push(target);
        }
        Ok(())
    }

    async fn ping_all(&self) {
        let mut targets_guard = self.targets.write().await;
        let mut sent_per_target = self.sent_per_target.write().await;
        let mut targets_to_remove = Vec::new();

        info!("ping_all: targets.len()={}", targets_guard.len());
        for t in targets_guard.iter() {
            info!("ping_all: target={}", t.address);
        }

        if targets_guard.is_empty() {
            return;
        }

        if let Some(ref client) = self.client {
            for target in targets_guard.iter_mut() {
                let address = target.address.clone();
                let current_sent = *sent_per_target.get(&address).unwrap_or(&0);
                info!("Checking target {}: sent={} limit={}", address, current_sent, target.count);

                if target.count > 0 && current_sent >= target.count {
                    info!("Target {} reached count limit, removing", address);
                    targets_to_remove.push(address);
                    continue;
                }
                
                *sent_per_target.entry(address.clone()).or_insert(0) += 1;
                self.sent_count.fetch_add(1, Ordering::SeqCst);
                let seq = self.sequence.fetch_add(1, Ordering::SeqCst);

                let result = match PingService::do_ping(client, target, seq).await {
                    Ok(res) => res,
                    Err(_) => PingResult {
                        success: false,
                        duration_ms: None,
                        ttl: None,
                    },
                };

                let mut results_guard = self.results.write().await;
                let target_results = results_guard.entry(address.clone()).or_insert(Vec::new());
                target_results.push(result.clone());

                if let Some(ref ui_tx) = self.tx {
                    info!("ping_all: Sending UiData to channel, tx is available");
                    let msg = if result.success {
                        let ttl_str = result.ttl.map(|t| format!(" TTL={}", t)).unwrap_or_default();
                        format!("Reply from {}: time={:.2}ms{}", 
                            address, result.duration_ms.unwrap_or(0.0), ttl_str)
                    } else {
                        format!("Request to {} timed out", address)
                    };
                    
                    match ui_tx.send(UiData::Log(Module::Ping, msg)).await {
                        Ok(_) => info!("ping_all: Log message sent successfully"),
                        Err(e) => error!("ping_all: Failed to send Log: {}", e),
                    }
                    
                    let total_sent = current_sent;
                    let received = target_results.iter().filter(|r| r.success).count() as u32;
                    let loss_count = total_sent.saturating_sub(received);
                    let loss = (loss_count as f32 / total_sent as f32) * 100.0;
                    
                    let durations: Vec<f64> = target_results.iter()
                        .filter_map(|r| r.duration_ms)
                        .collect();
                    let (min, max, avg) = if durations.is_empty() {
                        (0.0, 0.0, 0.0)
                    } else {
                        let min = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                        let max = durations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                        let avg = durations.iter().sum::<f64>() / durations.len() as f64;
                        (min, max, avg)
                    };

                    let stats = format!(
                        "Tx: {} Rx: {} Loss: {:.1}% Min: {:.1}ms Max: {:.1}ms Avg: {:.1}ms",
                        total_sent, received, loss, min, max, avg
                    );
                    
                    match ui_tx.send(UiData::PingStats(stats)).await {
                        Ok(_) => info!("ping_all: Stats message sent successfully"),
                        Err(e) => error!("ping_all: Failed to send Stats: {}", e),
                    }
                } else {
                    error!("ping_all: PingService tx is None, cannot send UI updates");
                }
            }
        }

        for addr in targets_to_remove {
            targets_guard.retain(|t| t.address != addr);
        }

        if targets_guard.is_empty() {
            *self.state.write().await = PingState::Idle;
            if let Some(ref ui_tx) = self.tx {
                let _ = ui_tx.send(UiData::ServiceStatus(Module::Ping, false)).await;
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

