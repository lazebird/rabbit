//! Ping Service

use crate::{
    ui_channel::{Module, UiData},
    Result, ServiceError, ServiceUpdateResult,
};
use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use socket2::Type as SockType;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{error, info};

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
#[derive(Clone)]
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

#[async_trait::async_trait]
impl crate::Service for PingService {
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

impl Default for PingService {
    fn default() -> Self {
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
}

impl PingService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self {
        Self { tx: Some(tx), ..Self::default() }
    }

    /// 公开接口：启停切换，会发状态通告
    pub async fn update(&mut self) -> ServiceUpdateResult {
        if *self.state.read().await == PingState::Running {
            match self.stop().await {
                Ok(()) => {
                    // 发状态通告，让 UI 更新配置
                    if let Some(ref ui_tx) = self.tx {
                        // 停止原因：手动停止
                        let _ = ui_tx.send(UiData::ServiceStatus(Module::Ping, false, None)).await;
                    }
                    ServiceUpdateResult::Stopped("Ping stopped".to_string())
                }
                Err(e) => ServiceUpdateResult::Error(format!("Failed to stop: {}", e)),
            }
        } else {
            match self.start().await {
                Ok(()) => {
                    if let Some(ref ui_tx) = self.tx {
                        let _ = ui_tx.send(UiData::ServiceStatus(Module::Ping, true, None)).await;
                    }
                    ServiceUpdateResult::Started("Ping started".to_string())
                }
                Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
            }
        }
    }

    /// 程序退出时调用，销毁资源，不发状态通告
    pub async fn destroy(&mut self) -> Result<()> {
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

    pub async fn send(&self, data: UiData) {
        crate::send_ui(&self.tx, data).await;
    }

    /// 内部启动 Ping 服务
    async fn start(&mut self) -> Result<()> {
        if self.client.is_none() {
            let config = Config::builder()
                .sock_type_hint(SockType::RAW)
                .build();
            let client = Client::new(&config).map_err(|e| ServiceError::Other(format!("Failed to create ping client: {}", e)))?;
            self.client = Some(Arc::new(client));
        }

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

        let service = self.clone();

        tokio::spawn(async move {
            info!("Ping task started");

            // 内部自动加载配置并添加目标
            if let Err(e) = service.add_target().await {
                error!("Failed to add target: {}", e);
            }

            // 确保任务启动时立即检查 targets
            let current_interval_ms = if let Some(first) = service.targets.read().await.first() {
                first.interval_ms
            } else {
                1000
            };
            let mut ping_interval = interval(Duration::from_millis(current_interval_ms));

            loop {
                tokio::select! {
                    _ = ping_interval.tick() => {
                        info!("Ping interval tick");
                        if *service.state.read().await == PingState::Running {
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

    fn load_target_from_config(&self) -> Option<PingTarget> {
        let config = rabbit_config::load_config().ok()?;
let target = config.modules.get_string("ping", schema::config::keys::ping::TARGET)?;
    let interval = config.modules.get_integer("ping", schema::config::keys::ping::INTERVAL).unwrap_or(1000) as u64;
    let count: i32 = config.modules.get_integer("ping", schema::config::keys::ping::COUNT).unwrap_or(-1) as i32;
    let stop_on_loss = config.modules.get_bool("ping", schema::config::keys::ping::STOP_ON_LOSS).unwrap_or(false);

        let mut target_obj = PingTarget::new(&target);
        target_obj.interval_ms = interval;
        target_obj.count = if count < 0 { u32::MAX } else { count as u32 };
        target_obj.stop_on_loss = stop_on_loss;

        Some(target_obj)
    }

    async fn add_target(&self) -> Result<()> {
        if let Some(target) = self.load_target_from_config() {
            info!("Adding target from config: {}", target.address);
            self.targets.write().await.push(target);
        }
        Ok(())
    }

    async fn ping_all(&self) {
        // 收集需要 ping 的目标（短时间持有锁）
        let targets_to_ping: Vec<(PingTarget, u32)> = {
            let targets_guard = self.targets.read().await;
            let sent_per_target = self.sent_per_target.read().await;

            targets_guard
                .iter()
                .filter_map(|target| {
                    let address = target.address.clone();
                    let current_sent = *sent_per_target.get(&address).unwrap_or(&0);

                    if target.count > 0 && current_sent >= target.count {
                        None
                    } else {
                        Some((target.clone(), current_sent))
                    }
                })
                .collect()
        };

        if targets_to_ping.is_empty() {
            // 检查是否需要清理已完成的目标
            let mut targets_guard = self.targets.write().await;
            let sent_per_target = self.sent_per_target.read().await;
            let mut targets_to_remove = Vec::new();

            for target in targets_guard.iter() {
                let address = target.address.clone();
                let current_sent = *sent_per_target.get(&address).unwrap_or(&0);
                if target.count > 0 && current_sent >= target.count {
                    targets_to_remove.push(address);
                }
            }

            for addr in &targets_to_remove {
                targets_guard.retain(|t| t.address != *addr);
            }

            if targets_guard.is_empty() {
                *self.state.write().await = PingState::Idle;
                if let Some(ref ui_tx) = self.tx {
                    // 停止原因：count 达到自动停止
                    let reason = Some("count reached".into());
                    let _ = ui_tx.send(UiData::ServiceStatus(Module::Ping, false, reason)).await;
                }
            }
            return;
        }

        if let Some(ref client) = self.client {
            for (target, current_sent) in targets_to_ping {
                let address = target.address.clone();

                // 更新计数（短时间持有锁）
                {
                    let mut sent_per_target = self.sent_per_target.write().await;
                    *sent_per_target.entry(address.clone()).or_insert(0) += 1;
                }
                self.sent_count.fetch_add(1, Ordering::SeqCst);
                let seq = self.sequence.fetch_add(1, Ordering::SeqCst);

                // 执行 ping（不持有锁）
                let result = match Self::do_ping(client, &target, seq).await {
                    Ok(res) => res,
                    Err(_) => PingResult {
                        success: false,
                        duration_ms: None,
                        ttl: None,
                    },
                };

                // 更新结果（短时间持有锁）
                let target_results = {
                    let mut results_guard = self.results.write().await;
                    let tr = results_guard.entry(address.clone()).or_insert(Vec::new());
                    tr.push(result.clone());
                    tr.clone() // 克隆用于统计计算
                };

                if let Some(ref ui_tx) = self.tx {
                    let msg = if result.success {
                        let ttl_str = result.ttl.map(|t| format!(" TTL={}", t)).unwrap_or_default();
                        format!("Reply from {}: time={:.2}ms{}", address, result.duration_ms.unwrap_or(0.0), ttl_str)
                    } else {
                        format!("Request to {} timed out", address)
                    };

                    let _ = ui_tx.send(UiData::Log(Module::Ping, msg)).await;

                    let total_sent = current_sent + 1;
                    let received = target_results.iter().filter(|r| r.success).count() as u32;
                    let loss_count = total_sent.saturating_sub(received);
                    let loss = (loss_count as f32 / total_sent as f32) * 100.0;

                    let durations: Vec<f64> = target_results.iter().filter_map(|r| r.duration_ms).collect();
                    let (min, max, avg) = if durations.is_empty() {
                        (0.0, 0.0, 0.0)
                    } else {
                        let min = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                        let max = durations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                        let avg = durations.iter().sum::<f64>() / durations.len() as f64;
                        (min, max, avg)
                    };

                    let stats = format!("Tx: {} Rx: {} Loss: {:.1}% Min: {:.1}ms Max: {:.1}ms Avg: {:.1}ms", total_sent, received, loss, min, max, avg);

                    let _ = ui_tx.send(UiData::PingStats(stats)).await;

                    // 计算任务栏状态并发送（根据最新需求文档规则：滑动窗口 5 次）
                    // 1. 截取最近 5 次结果
                    let recent_results = if target_results.len() > 5 {
                        &target_results[target_results.len() - 5..]
                    } else {
                        &target_results[..]
                    };

                    let recent_success = recent_results.iter().filter(|r| r.success).count() as u32;
                    let recent_fail = recent_results.len() as u32 - recent_success;
                    
                    let full_count = 5u32;
                    let (progress, color) = if recent_fail > 0 {
                        // 只要有失败：红色，progress = 失败次数
                        (recent_fail, "red")
                    } else {
                        // 全部成功：绿色，progress = 成功次数
                        (recent_success, "green")
                    };

                    let _ = ui_tx
                        .send(UiData::PingState {
                            address: address.clone(),
                            progress,
                            total: full_count,
                            color: color.to_string(),
                        })
                        .await;
                }
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

    /// 内部停止 Ping 服务
    async fn stop(&mut self) -> Result<()> {
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
}
