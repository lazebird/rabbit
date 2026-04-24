# Rabbit 数据流与状态同步设计文档

## 1. 当前实现问题

### 1.1 现有数据流模式 (全部低效)

| 模块 | 方式 | 问题 |
|------|------|------|
| **Ping** | tokio spawn 轮询 (1s) | 每次都调用 get_results，即使无新数据 |
| **Scan** | tokio spawn 轮询 (200ms) | 同上，且状态变化未主动通知 |
| **HTTP** | tokio spawn 轮询 get_recent_logs | 每次轮询获取10条，重复处理 |
| **Chat** | tokio spawn 轮询 | 同上 |
| **Tftpd** | 直接 append_*_log() | 异步任务直接调用同步函数 |

### 1.2 当前代码分析

```rust
// app.rs:866-935 - Ping 轮询 (最典型)
let handle = tokio::spawn(async move {
    loop {
        tokio::time::sleep(1s).await;
        
        // 问题1: 每次都调用，即使无新数据
        let results = service.get_results(&target).await;
        let summary = service.get_summary(&target).await;
        
        // 问题2: 直接调用 ui_state (非线程安全)
        ui_state::append_ping_output(&line);
        ui_state::set_ping_stats(&stats);
        
        // 唯一正确的地方: 窗口标题用 awake_callback
        fltk::app::awake_callback(move || {
            win.set_label(&target_label);
        });
    }
});

// app.rs:1049-1064 - HTTP 轮询
let handle = tokio::spawn(async move {
    loop {
        tokio::time::sleep(1s).await;
        let logs = service.get_recent_logs(10).await;  // 每次获取10条
        for log in logs {
            ui_state::append_http_log(&line);  // 同步写入
        }
    }
});

// 问题:
// 1. 轮询间隔固定，无法及时响应
// 2. 重复处理已有数据 (get_recent_logs 需要去重)
// 3. tokio 线程直接调用 ui_state (线程不安全)
// 4. Service 和 ui_state 紧耦合
```

### 1.3 核心问题总结

```
┌─────────────────────────────────────────────────────────────────┐
│                    核心问题                              │
├─────────────────────────────────────────────────────────────────┤
│  1. 轮询机制低效                                        │
│     - 无数据也重复调用                                  │
│     - 固定间隔响应慢                                  │
│                                                            │
│  2. 紧耦合                                            │
│     - Service 和 ui_state 直接通信                      │
│     - 无法解耦测试                                    │
│                                                            │
│  3. 线程安全风险                                     │
│     - tokio 线程直接调用 ui_state                     │
│     - ui_state 用全局 Mutex                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 统一数据传输方案

### 2.1 设计原则

1. **推送代替轮询**: Service 产生数据后主动推送，不等待 UI 拉取
2. **Channel 解耦**: 使用 tokio channel 作为中介，完全解耦
3. **线程安全**: channel 跨线程安全，ui_state 集中处理 UI 线程更新
4. **单一职责**: Service 只负责产生数据，不直接调用 UI

### 2.2 统一 Channel 接口

```rust
// ��══════════════════════════════════════════════════════════════════════
// 统一数据通道 - 所有模块使用
// ═══════════════════════════════════════════════════════════════════════

use tokio::sync::mpsc;

/// 统一数据 - 所有模块
#[derive(Debug, Clone)]
pub enum UiData {
    // ═══ 通用日志 ═══
    // HTTP/TFTP/Chat/PingResult/ScanResult/PlanResult
    Log(Module, String),
    
    // ═══ Ping 专用 ═══
    PingStats(String),                               // "Tx 10 Rx 9 Loss 10%"
    PingState { address: String, progress: u32, total: u32, color: String },
    
    // ═══ Scan 专用 ═══
    ScanProgress(String),                           // "Progress: 50% - Found 5 hosts"
    
    // ═══ Plan 专用 ═══
    PlanReminder(String),                          // 计划到期提醒消息
    
    // ═══ Chat 专用 ═══
    ChatMessage(String, String),               // (用户名, 消息)
    ChatUserList(String),                      // 在线用户列表 "," 分隔
    
    // ═══ 错误 ═══
    Error(Module, String),
}

/// 模块枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}

/// 统一 Channel Manager
pub struct UiChannels {
    // 每个模块一个 channel
    ping_tx: mpsc::Sender<UiData>,
    http_tx: mpsc::Sender<UiData>,
    scan_tx: mpsc::Sender<UiData>,
    tftp_tx: mpsc::Sender<UiData>,
    chat_tx: mpsc::Sender<UiData>,
    plan_tx: mpsc::Sender<UiData>,
}

### 6.4 app.rs 统一处理

```rust
async fn handle_ui_data(data: UiData) {
    match data {
        // 通用日志 - 直接输出到对应模块的日志框
        UiData::Log(module, text) => {
            match module {
                Module::Http => ui_state::append_http_log(&text),
                Module::Tftpd => ui_state::append_tftpd_log(&text),
                Module::Tftpc => ui_state::append_tftpc_log(&text),
                Module::Ping => ui_state::append_ping_output(&text),
                Module::Scan => ui_state::append_scan_output(&text),
                // Plan 无通用日志
                // Chat 无通用日志，使用 ChatMessage
                _ => {}
            }
        }
        
        // Ping 统计 - 直接更新统计框
        UiData::PingStats(text) => {
            ui_state::set_ping_stats(&text);
        }
        
        // Ping 状态 - 更新按钮状态和窗口标题
        UiData::PingState { address, progress, total, color } => {
            let running = !address.is_empty();
            ui_state::set_ping_running(running);
            
            // progress/total 计算进度
            let progress_text = if total > 0 {
                let pct = (progress as f32 / total as f32 * 100.0) as u32;
                format!("{}/{} ({}%)", progress, total, pct)
            } else {
                format!("{}/{}", progress, total)
            };
            
            // 更新窗口标题显示地址和进度
            if running {
                if let Some(mut win) = ui_state::get_main_window() {
                    let title = format!("Rabbit - {} [{}]", address, progress_text);
                    fltk::app::awake_callback(move || {
                        win.set_label(&title);
                    });
                }
            }
        }
        
        // Scan 进度 - 直接字符串
        UiData::ScanProgress(text) => {
            ui_state::append_scan_output(&text);
        }
        
        // Plan 提醒 - 显示提醒消息
        UiData::PlanReminder(text) => {
            ui_state::append_plan_event(...);
        }
        
        // Chat 消息 - 用户名和内容
        UiData::ChatMessage(username, content) => {
            ui_state::append_chat_message(&username, &content);
        }
        
        // Chat 用户列表 - 更新用户显示区域
        UiData::ChatUserList(users) => {
            ui_state::set_chat_users(&users);
        }
        
        // 错误 - 窗口标题显示
        UiData::Error(module, msg) => {
            if let Some(mut win) = ui_state::get_main_window() {
                fltk::app::awake_callback(move || {
                    win.set_label(&format!("Rabbit - Error: {}", msg));
                });
            }
        }
        
        _ => {}
    }
}
```

impl HttpService {
    pub fn new(channel: mpsc::Sender<UiData>) -> Self {
        Self { channel, .. }
    }
    
    // 产生日志时推送
    async fn on_request(&self, req: &Request) {
        let data = UiData::HttpRequest {
            method: req.method.clone(),
            path: req.path.clone(),
            status: req.status,
            size: req.size,
        };
        let _ = self.channel.send(data).await;
    }
}

// ════════════════════════════════════════════════════════════════��══════
// app.rs 接收端 - 统一处理
// ═══════════════════════════════════════════════════════════════════════

pub struct App {
    ping_receiver: mpsc::Receiver<UiData>,
    http_receiver: mpsc::Receiver<UiData>,
    // ...
}

impl App {
    pub async fn start_ui_receiver_tasks(&mut self) {
        // 每个模块一个任务
        let mut ping_rx = std::mem::replace(&mut self.ping_receiver,/unimplemented!());
        tokio::spawn(async move {
            while let Some(data) = ping_rx.recv().await {
                Self::handle_ping_data(data).await;
            }
        });
        
        let mut http_rx = std::mem::replace(&mut self.http_receiver,unimplemented!());
        tokio::spawn(async move {
            while let Some(data) = http_rx.recv().await {
                Self::handle_http_data(data).await;
            }
        });
    }
    
    async fn handle_ping_data(data: UiData) {
        match data {
            UiData::PingResult { target, success, rtt_ms, ttl } => {
                let line = if success {
                    format!("Reply from {}: time={:.1}ms TTL={}\r\n", target, rtt_ms, ttl.unwrap_or(64))
                } else {
                    "Request timed out.\r\n".to_string()
                };
                // 线程安全的 UI 更新
                ui_state::append_ping_output(&line);
            }
            UiData::PingStats { sent, received, loss_rate } => {
                let stats = format!("Tx {} Rx {} Loss {:.1}%", sent, received, loss_rate * 100.0);
                ui_state::set_ping_stats(&stats);
            }
            UiData::PingStateChanged { state } => {
                let running = matches!(state, PingState::Running);
                ui_state::set_ping_running(running);
            }
            _ => {}
        }
    }
    
    async fn handle_http_data(data: UiData) {
        match data {
            UiData::HttpRequest { method, path, status, size } => {
                let line = format!("{} {} {} - {}\r\n",
                    chrono::Local::now().format("%H:%M:%S"),
                    method, path, status
                );
                ui_state::append_http_log(&line);
            }
            _ => {}
        }
    }
}
```

### 2.3 方案对比

| 方案 | 实时性 | 复杂度 | 耦合 | 线程安全 |
|------|--------|--------|------|------|----------|
| **轮询 (当前)** | ~1s 延迟 | 低 | 紧耦合 | 风险高 |
| **Channel (新)** | 实时 | 中 | 完全解耦 | 安全 |

### 2.4 迁移计划

```
┌─────────────────────────────────────────────────────────────────┐
│                    迁移步骤                              │
├─────────────────────────────────────────────────────────────────┤
│  Phase 1: 定义 UiChannelManager + UiData                │
│     - 定义 UiData 枚举                                   │
│     - 定义 UiChannelManager                           │
│                                                            │
│  Phase 2: HTTP 改造 (最简单)                         │
│     - HttpService 添加 channel 字段                   │
│     - 产生日志时改用 channel.send()                │
│     - app.rs 改用 receiver 处理                     │
│                                                            │
│  Phase 3: Ping 改造                                     │
│     - 同上，但需要处理状态变化                        │
│     - 完成/异常等状态主动通知                        │
│                                                            │
│  Phase 4: 其他模块迁移                                 │
│     - Scan, TFTP, Chat 同上                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 接口设计

### 3.1 UiData 枚举 (核心)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UiData {
    // 模块特定
    PingResult { target: String, bytes: u32, rtt_ms: f32, ttl: Option<u8> },
    PingStats { sent: u32, received: u32, min_ms: f32, max_ms: f32, avg_ms: f32 },
    PingState { state: PingState },
    
    HttpRequest { time: String, method: String, path: String, status: u16 },
    
    ScanResult { ip: String, online: bool, hostname: Option<String> },
    ScanProgress { current: u32, total: u32, found: u32 },
    
    TftpProgress { file: String, transferred: u64, total: u64, mode: String },
    
    ChatMessage { sender: String, content: String, time: String },
    ChatUsers { users: Vec<String> },
    
    // 通用
    ServiceState { module: Module, running: bool, message: String },
    Error { module: Module, message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingState {
    Idle, Running, Stopped, Completed, Error,
}
```

### 3.2 Channel 创建

```rust
// 每个模块的 channel 容量
const CHANNEL_BUFFER: usize = 100;

pub struct UiChannels {
    pub ping: (mpsc::Sender<UiData>, mpsc::Receiver<UiData>),
    pub http: (mpsc::Sender<UiData>, mpsc::Receiver<UiData>),
    pub scan: (mpsc::Sender<UiData>, mpsc::Receiver<UiData>),
    pub tftp: (mpsc::Sender<UiData>, mpsc::Receiver<UiData>),
    pub chat: (mpsc::Sender<UiData>, mpsc::Receiver<UiData>),
}

impl UiChannels {
    pub fn new() -> Self {
        Self {
            ping: mpsc::channel(CHANNEL_BUFFER),
            http: mpsc::channel(CHANNEL_BUFFER),
            scan: mpsc::channel(CHANNEL_BUFFER),
            tftp: mpsc::channel(CHANNEL_BUFFER),
            chat: mpsc::channel(CHANNEL_BUFFER),
        }
    }
}
```

---

## 4. 实现计划

### Phase 1: 核心
- [ ] 定义 UiData 枚举
- [ ] 定义 UiChannels 结构

### Phase 2: HTTP 改造
- [ ] HttpService 添加 channel
- [ ] 改造 app.rs 接收端
- [ ] 删除轮询代码

### Phase 3: Ping 改造
- [ ] PingService 添加 channel
- [ ] 状态变化主动通知
- [ ] 删除轮询代码

### Phase 4: 其他模块
- [ ] Scan 改造
- [ ] TFTP 改造
- [ ] Chat 改造

---

## 5. Channel 方案选择: 1对1 vs N对1

### 5.1 Tokio Channel 特性

```
┌─────────────────────────────────────────────────────────────────┐
│                  tokio::sync::mpsc 特性                        │
├─────────────────────────────────────────────────────────────────┤
│  - Sender 可克隆 → 多个发送者 → 1 个接收者                     │
│  - Receiver 不可克隆 → 只支持单个接收者                        │
│  - bounded channel: 带背压，防止内存爆炸                       │
│  - 发送阻塞: 缓冲区满时 send().await 会等待                   │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 方案对比

| 方案 | 结构 | 优点 | 缺点 |
|------|------|------|------|
| **1对1** | 每个模块独立 channel | 隔离好，互不影响 | 多个 receiver 管理复杂 |
| **N对1** | 所有模块共享 channel | 统一管理，简单 | 不推荐 |

### 5.3 推荐: 多个 1对1

```rust
// 每个模块独立 channel
pub struct UiChannels {
    pub ping_tx: mpsc::Sender<UiData>,
    pub http_tx: mpsc::Sender<UiData>,
    pub scan_tx: mpsc::Sender<UiData>,
    pub tftp_tx: mpsc::Sender<UiData>,
    pub chat_tx: mpsc::Sender<UiData>,
}
```

### 5.4 为什么不用单个 N对1

1. **数据混杂**: 不同模块的数据格式不同，混在一起需要额外路由
2. **接收端复杂**: 需要用 `tag` 区分，代码丑化
3. **无隔离**: 一个模块问题可能影响其他模块
4. **Tokio 不支持** : mpsc 只支持 1 receiver

---

## 6. 简化设计

### 6.1 数据分类

| 类型 | 模块 | 处理方式 |
|------|------|----------|
| **日志型** | HTTP/TFTP/Chat | 直接输出到日志框 |
| **结构型** | Ping/Scan | 拆解字段更新 UI 组件 |

### 6.2 统一 UiData

```rust
// 只需要两种: 通用日志 + 特殊数据
#[derive(Debug, Clone)]
pub enum UiData {
    // 通用日志 - 直接输出到日志框
    // HTTP/TFTP/Chat/PingResult 使用
    Log(Module, String),  // (module, formatted_text)
    
    // Ping 统计 - 字符串直接更新统计框
    PingStats(String),  // "Tx 10 Rx 9 Loss 10% Min 1ms Max 5ms Avg 2ms"
    
    // Ping 状态 - 地址、进度、颜色 (业务状态由 Service 自身处理)
    PingState {
        address: String,      // 当前 ping 的地址
        progress: String,     // "5/100" 或 "50%"
        color: String,       // "green", "red" 用于状态指示
    },
    
    // Scan 进度
    ScanProgress(String),  // "Progress: 50% - Found 5 hosts"
}
```

pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}
```

### 6.3 简化实现

```rust
// HTTP Service 只需发送格式化字符串
impl HttpService {
    async fn on_request(&self, req: &Request) {
        let text = format!("{} {} {} - {}\r\n",
            Local::now().format("%H:%M:%S"),
            req.method,
            req.path,
            req.status
        );
        // 直接发送字符串，不需要字段拆分
        let _ = self.tx.send(UiData::Log(Module::Http, text)).await;
    }
}

// app.rs 统一处理
async fn handle_data(data: UiData) {
    match data {
        UiData::Log(module, text) => {
            match module {
                Module::Http => ui_state::append_http_log(&text),
                Module::Tftpd => ui_state::append_tftpd_log(&text),
                _ => {}
            }
        }
        UiData::PingResult { target, success, rtt_ms, ttl } => {
            // 结构化处理
        }
        // ...
    }
}
```

### 5.4 为什么不用单个 N对1

1. **数据混杂**: 不同模块的数据格式不同，混在一起需要额外路由
2. **接收端复杂**: 需要用 `tag` 区分，代码丑化
3. **无隔离**: 一个模块问题可能影响其他模块
4. ** Tokio 不支持** : mpsc 只支持 1 receiver，需要 broadcast 才能多 receiver

### 5.5 结构设计

```rust
// ═══════════════════════════════════════════════════════════════════════
// 统一数据结构: 服务状态 + 服务数据 分离
// ═══════════════════════════════════════════════════════════════════════

// 服务状态 - 不通过 channel，用 watch 或直接查询
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Idle,
    Starting,
    Running,
    Stopping,
    Stopped,
    Completed,  // 正常结束 (如次数到限)
    Error,       // 异常结束
}

// 服务数据分类:
// 1. 简单日志型 (HTTP/TFTP/Chat) - 只需格式化字符串
// 2. 结构型 (Ping/Scan) - 需要具体字段用于 UI 组件

#[derive(Debug, Clone)]
pub enum UiData {
    // ═══ 简单日志型 (直接输出到日志框) ═══
    Log(LogData),  // 通用日志，包含格式化好的字符串
    
    // ═══ 结构型 (需要拆解字段) ═══
    PingResult { target: String, bytes: u32, rtt_ms: f32, ttl: Option<u8>, success: bool },
    PingStats { sent: u32, received: u32, loss: u32, min_ms: f32, max_ms: f32, avg_ms: f32 },
    
    ScanResult { ip: String, online: bool, hostname: Option<String> },
    ScanProgress { progress: u32, found: u32 },
    
    // ═══ 错误通知 ═══
    Error { module: Module, message: String },
}

/// 通用日志数据 (HTTP/TFTP/Chat 等模块使用)
#[derive(Debug, Clone)]
pub struct LogData {
    pub module: Module,
    pub text: String,      // 格式化好的文本，直接输出
    pub level: LogLevel,
    pub timestamp: String, // 如 "12:34:56"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// 模块枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}

// Channel 管理
pub struct UiChannels {
    ping_tx: mpsc::Sender<UiData>,
    http_tx: mpsc::Sender<UiData>,
    scan_tx: mpsc::Sender<UiData>,
    tftp_tx: mpsc::Sender<UiData>,
    chat_tx: mpsc::Sender<UiData>,
}

impl UiChannels {
    pub fn new() -> Self {
        Self {
            ping_tx: mpsc::Sender::channel(100).0,
            http_tx: mpsc::Sender::channel(100).0,
            scan_tx: mpsc::Sender::channel(100).0,
            tftp_tx: mpsc::Sender::channel(100).0,
            chat_tx: mpsc::Sender::channel(100).0,
        }
    }
}
}

pub struct UiChannelReceipts {
    ping_rx: Option<mpsc::Receiver<UiData>>,
    http_rx: Option<mpsc::Receiver<UiData>>,
    scan_rx: Option<mpsc::Receiver<UiData>>,
    tftp_rx: Option<mpsc::Receiver<UiData>>,
    chat_rx: Option<mpsc::Receiver<UiData>>,
}

// app.rs 使用
pub struct AppState {
    pub http_service: HttpService,
    pub ping_service: PingService,
    // ...
    pub channel_receipts: UiChannelReceipts,
}

impl AppState {
    pub async fn start_receiver_tasks(&mut self) {
        // 每个模块一个接收任务
        if let Some(mut rx) = self.channel_receipts.http_rx.take() {
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    Self::handle_http(data).await;
                }
            });
        }
        
        if let Some(mut rx) = self.channel_receipts.ping_rx.take() {
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    Self::handle_ping(data).await;
                }
            });
        }
    }
    
    // HTTP/TFTP/Chat 等模块: 统一的 Log 处理
    async fn handle_http(data: UiData) {
        match data {
            UiData::Log(log) => {
                // 直接输出到日志框
                ui_state::append_http_log(&log.text);
            }
            _ => {}
        }
    }
    
    // Ping 模块: 结构化数据处理
    async fn handle_ping(data: UiData) {
        match data {
            UiData::PingResult { target, success, rtt_ms, ttl } => {
                let line = if success {
                    format!("Reply from {}: time={:.1}ms TTL={}\r\n", 
                        target, rtt_ms, ttl.unwrap_or(64))
                } else {
                    "Request timed out.\r\n".to_string()
                };
                ui_state::append_ping_output(&line);
            }
            UiData::PingStats { sent, received, loss, min_ms, max_ms, avg_ms } => {
                let stats = format!("Tx {} Rx {} Loss {} Min {:.1}ms Max {:.1}ms Avg {:.1}ms",
                    sent, received, loss, min_ms, max_ms, avg_ms);
                ui_state::set_ping_stats(&stats);
            }
            // Ping 模块的日志也通过 Log 发送
            UiData::Log(log) => {
                ui_state::append_ping_output(&log.text);
            }
            _ => {}
        }
    }
}
```