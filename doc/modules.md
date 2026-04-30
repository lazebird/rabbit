# Rabbit 模块化结构文档

## 项目架构

```
┌─────────────────────────────────────────┐
│           Presentation Layer             │
│   View (FLTK UI) ← ViewModel (State)     │
├─────────────────────────────────────────┤
│           Business Layer               │
│   PingService / HttpService / ...        │
├─────────────────────────────────────────┤
│           Data Layer                   │
│   Models + Repository                 │
├─────────────────────────────────────────┤
│        Infrastructure Layer           │
│   Platform / Config / Utils            │
└─────────────────────────────────────────┘
```

---

## 模块结构

```
rabbit/
├── crates/
│   ├── rabbit-app/       # 主入口 + UI
│   │   └── src/
│   │       ├── main.rs           # 程序入口
│   │       ├── lib.rs           # 库入口
│   │       ├── app.rs           # 主应用、事件处理、生命周期
│   │       ├── view_model.rs    # 统一配置管理
│   │       ├── ui_state.rs      # UI 状态管理
│   │       ├── ui_events.rs     # UI 事件系统
│   │       ├── ui/              # FLTK 界面组件
│   │       │   ├── mod.rs       # UI 模块入口
│   │       │   ├── ping_tab.rs  # Ping 界面
│   │       │   ├── scan_tab.rs  # Scan 界面
│   │       │   ├── http_tab.rs  # HTTP 界面
│   │       │   ├── tftpd_tab.rs # TFTP 服务器界面
│   │       │   ├── tftpc_tab.rs # TFTP 客户端界面
│   │       │   ├── plan_tab.rs  # Plan 界面
│   │       │   ├── chat_tab.rs  # Chat 界面
│   │       │   ├── settings_tab.rs # 设置界面
│   │       │   ├── defaults.rs  # 默认值
│   │       │   ├── styles.rs    # 样式定义
│   │       │   └── ui_refresh.rs # UI 更新（事件驱动回调）
│   │       └── upgrade/         # 升级模块
│   │           ├── mod.rs       # 升级模块入口
│   │           ├── models.rs   # 版本数据结构
│   │           ├── downloader.rs # 下载器
│   │           └── installer.rs   # 安装器
│   │
│   ├── rabbit-core/       # 业务服务
│   │   └── src/
│   │       ├── lib.rs           # 库入口、ServiceError 定义
│   │       ├── ping.rs          # Ping 服务
│   │       ├── scan.rs          # 扫描服务
│   │       ├── http.rs          # HTTP 服务
│   │       ├── tftpd.rs         # TFTP 服务器
│   │       ├── tftpc.rs         # TFTP 客户端
│   │       ├── chat.rs          # 聊天服务
│   │       ├── plan.rs          # 计划服务
│   │       └── ui_channel.rs    # UI 通道管理
│   │
│   ├── rabbit-models/    # 数据模型
│   │   └── src/
│   │       ├── lib.rs           # UiData、Module 枚举
│   │       ├── config.rs        # AppConfig 配置模型
│   │       └── scan.rs          # Scan 数据模型
│   │
│   └── rabbit-platform/  # 基础设施
│       └── src/
│           ├── lib.rs           # 平台接口
│           ├── config.rs        # 配置持久化
│           ├── autostart.rs     # 开机自启
│           ├── elevation.rs     # 权限提升
│           ├── notification.rs  # 系统通知
│           ├── taskbar.rs       # 任务栏集成
│           ├── shell.rs         # Shell 操作
│           ├── dialog.rs        # 对话框
│           ├── network.rs       # 网络工具
│           └── ping.rs          # 平台级 Ping（Windows）
│
├── doc/
│   ├── architecture.md         # 架构设计
│   ├── data-flow-design.md    # 数据流设计
│   ├── config-structure.md    # 配置结构
│   ├── modules.md             # 模块化结构
│   └── progress.md            # 开发进度
│
└── Cargo.toml
```

---

## rabbit-app (表现层 + 业务层入口)

### 主要文件

| 文件 | 职责 |
|------|------|
| `main.rs` | 程序入口、Tokio 运行时初始化、日志初始化 |
| `lib.rs` | 库入口、App 导出 |
| `app.rs` | 主应用、事件处理、生命周期管理、UI channel 接收 |
| `view_model.rs` | 统一配置管理、运行状态跟踪 |
| `ui_state.rs` | UI 状态同步（全局状态、刷新标记） |
| `ui_events.rs` | UI 事件系统（事件发送/接收） |
| `ui/ui_refresh.rs` | 集中式 UI 刷新（100ms 定时器） |

### AppViewModel

```rust
pub struct AppViewModel {
    config: AppConfig,
}

impl AppViewModel {
    // 基础
    pub fn new(config: AppConfig) -> Self
    pub fn get_config(&self) -> AppConfig
    pub fn set_config(&mut self, config: AppConfig)
    pub fn save(&self) -> Result<()>
    pub fn update_config(&mut self, config: AppConfig)
    pub fn update_and_save(&mut self, config: AppConfig) -> Result<()>
    
    // 通用读写 (section + key)
    pub fn get_string(&self, section: &str, key: &str) -> Option<String>
    pub fn get_integer(&self, section: &str, key: &str) -> Option<i64>
    pub fn get_bool(&self, section: &str, key: &str) -> Option<bool>
    pub fn get_array(&self, section: &str, key: &str) -> Option<Vec<String>>
    pub fn set_string(&mut self, section: &str, key: &str, value: String)
    pub fn set_integer(&mut self, section: &str, key: &str, value: i64)
    pub fn set_bool(&mut self, section: &str, key: &str, value: bool)
    pub fn set_array(&mut self, section: &str, key: &str, value: Vec<String>)
}
```

运行状态由各 Service 内部管理，通过通用接口读写。

### App

```rust
pub struct App {
    view_model: Arc<RwLock<AppViewModel>>,
    ping_service: Arc<RwLock<PingService>>,
    http_service: Arc<RwLock<HttpService>>,
    tftp_server_service: Arc<RwLock<TftpdService>>,
    tftp_client_service: Arc<RwLock<TftpcService>>,
    plan_service: Arc<RwLock<PlanService>>,
    chat_service: Arc<RwLock<ChatService>>,
    scan_service: Arc<RwLock<ScanService>>,
    ping_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    scan_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    shutdown_flag: Arc<AtomicBool>,
    // UI data receivers
    http_rx: Option<mpsc::Receiver<UiData>>,
    ping_rx: Option<mpsc::Receiver<UiData>>,
    scan_rx: Option<mpsc::Receiver<UiData>>,
    tftpd_rx: Option<mpsc::Receiver<UiData>>,
    tftpc_rx: Option<mpsc::Receiver<UiData>>,
    chat_rx: Option<mpsc::Receiver<UiData>>,
    plan_rx: Option<mpsc::Receiver<UiData>>,
}

impl App {
    pub async fn new() -> anyhow::Result<Self>
    pub async fn run(&mut self) -> anyhow::Result<()>
}
```

### 对外接口

```rust
// App 初始化
pub async fn new() -> anyhow::Result<Self>
pub async fn run(&mut self) -> anyhow::Result<()>

// ViewModel 配置访问
pub fn get_config(&self) -> AppConfig
pub fn update_config(&mut self, config: AppConfig)
```

### Upgrade Module

```rust
// Upgrade 模块接口
pub use models::{UpdateStatus, VersionsManifest, PlatformInfo};
pub use downloader::{download_update, DownloadProgress};
pub use installer::{install_update, get_current_exe_path};

// 主要类型
pub struct VersionsManifest { ... }
pub struct PlatformInfo { ... }
pub enum UpdateStatus { NoUpdate, Available, Downloading, Ready, Installing, Done }
pub struct DownloadProgress { bytes_downloaded, total_bytes, ... }
```

### UI Channel 接收

```rust
// app.rs 中的 UI 数据处理
async fn handle_ui_data(data: UiData, view_model: &Arc<RwLock<AppViewModel>>)

// 处理类型：
// - ServiceStatus(module, running)  -> 模块状态变化
// - PingStats(stats)                -> Ping 统计
// - Log(module, msg)                -> 通用日志
// - ScanProgress(msg)               -> Scan 进度
// - ChatMessage(username, msg)      -> Chat 消息
```

---

## rabbit-core (业务服务层)

### 统一接口

所有服务实现统一接口：

```rust
pub async fn update(&mut self) -> ServiceUpdateResult  // 切换状态 (启动/停止)
```

> 注：init() 已在 0.2.0 中移除，各服务按需懒初始化

### Channel 推送接口

服务通过 Channel 向 UI 推送数据：

```rust
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
```

### UiData/Module 来源

`UiData` 和 `Module` 枚举定义在 `rabbit-models`，`rabbit-core` 通过 `ui_channel.rs` re-export：

```rust
// rabbit-models/src/lib.rs - 权威定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiData {
    // 通用日志（HTTP/TFTP/Chat/Ping/Scan）
    Log(Module, String),
    
    // Ping 专用
    PingStats(String),        // "Tx 10 Rx 9 Loss 10%"
    PingState { address: String, progress: u32, total: u32, color: String },
    
    // Scan 专用
    ScanProgress(String),       // "Progress: 50% - Found 5 hosts"
    
    // Plan 专用
    PlanReminder(String),      // 计划到期提醒
    
    // Chat 专用
    ChatMessage(String, String), // (用户名, 消息)
    ChatUserList(String),        // 在线用户列表（逗号分隔）
    
    // 服务状态更新（核心：业务状态通知）
    ServiceStatus(Module, bool), // (模块, 是否运行中)
    
    // 错误通知
    Error(Module, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}
```

> **注意**：`ServiceStatus` 是业务状态更新的核心接口，用于：
> - Ping count 达到时 → `ServiceStatus(Ping, false)`
> - Scan 完成时 → `ServiceStatus(Scan, false)`
> - 手动停止服务 → `ServiceStatus(Module, false)`

### PingService

| 项目 | 说明 |
|------|------|
| 文件 | `src/ping.rs` |
| 职责 | ICMP Ping 功能 + 任务栏状态 |
| 依赖 | `surge-ping` |
| Channel | `with_channel(tx)` |

**接口**：
```rust
pub fn new() -> Self
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
pub async fn update(&mut self) -> ServiceUpdateResult  // 切换状态 (启动/停止)
pub async fn is_running(&self) -> bool  // 检查是否运行中
pub fn send(&self, data: UiData)  // 发送 UI 数据
```

**状态更新**：
- 启动时发送：`UiData::ServiceStatus(Module::Ping, true)`
- 停止时发送：`UiData::ServiceStatus(Module::Ping, false)`
- 任务栏标题：通过读取配置中的 `target` 设置窗口标题

### HttpService

| 项目 | 说明 |
|------|------|
| 文件 | `src/http.rs` |
| 职责 | HTTP 文件服务器 |
| 依赖 | `axum` |

**接口**：
```rust
pub fn new() -> Self
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
pub async fn update(&mut self) -> ServiceUpdateResult
```

### TftpdService

| 项目 | 说明 |
|------|------|
| 文件 | `src/tftpd.rs` |
| 职责 | TFTP 服务器 |
| 依赖 | `async-tftp` |

**接口**：
```rust
pub fn new() -> Self
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
pub async fn update(&mut self) -> ServiceUpdateResult
```

### TftpcService

| 项目 | 说明 |
|------|------|
| 文件 | `src/tftpc.rs` |
| 职责 | TFTP 客户端 |

**接口**：
```rust
pub fn new() -> Self
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
pub async fn put(&self, local_path: &str, remote_filename: &str) -> Result<String>
pub async fn get(&self, remote_filename: &str, local_path: &str) -> Result<String>
```

### ScanService

| 项目 | 说明 |
|------|------|
| 文件 | `src/scan.rs` |
| 职责 | IP 扫描 |
| 依赖 | `tokio`, `dns-lookup` |

**接口**：
```rust
pub fn new() -> Self
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
pub async fn update(&mut self) -> ServiceUpdateResult  // 切换状态 (启动/停止)
pub async fn destroy(&mut self) -> Result<()>  // 销毁资源，不发状态通告
pub fn send(&self, data: UiData)  // 发送 UI 数据
```

**状态更新**：
- 完成时发送：`UiData::ServiceStatus(Module::Scan, false)`（让按钮从 "Stop" → "Start"）
- 发现主机时发送：`UiData::Log(Module::Scan, "Found online host: ...")`
- 进度更新：通过 `ScannerState::Scanning { progress }` 内部跟踪

### ChatService

| 项目 | 说明 |
|------|------|
| 文件 | `src/chat.rs` |
| 职责 | 局域网聊天 |

**接口**：
```rust
pub fn new() -> Self
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
pub async fn update(&mut self) -> ServiceUpdateResult
pub async fn send_text(&self, content: &str) -> Result<()>
pub async fn refresh_users(&self) -> Result<()>
```

### PlanService

| 项目 | 说明 |
|------|------|
| 文件 | `src/plan.rs` |
| 职责 | 定时提醒 |

**接口**：
```rust
pub fn new() -> Self
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
pub async fn update(&mut self) -> ServiceUpdateResult
pub async fn add_task(&self, date: &str, time: &str, cycle: i32, unit: &str, msg: &str) -> Result<()>
pub async fn remove_task(&self, id: &str) -> Result<()>
```

### UiChannel 模块

| 项目 | 说明 |
|------|------|
| 文件 | `src/ui_channel.rs` |
| 职责 | Channel 创建工具 + 统一导出 UiData/Module |

```rust
// 发送辅助函数（消除重复实现）
pub async fn send_ui(tx: &Option<mpsc::Sender<UiData>>, data: UiData) {
    if let Some(sender) = tx {
        let _ = sender.send(data).await;
    }
}

// Channel 管理器（每个模块独立 channel）
pub struct UiChannels {
    pub ping_tx: mpsc::Sender<UiData>,
    pub http_tx: mpsc::Sender<UiData>,
    pub scan_tx: mpsc::Sender<UiData>,
    pub tftpd_tx: mpsc::Sender<UiData>,
    pub tftpc_tx: mpsc::Sender<UiData>,
    pub chat_tx: mpsc::Sender<UiData>,
    pub plan_tx: mpsc::Sender<UiData>,
}

// 接收端集合
pub struct UiReceivers {
    pub ping: mpsc::Receiver<UiData>,
    pub http: mpsc::Receiver<UiData>,
    // ... 其他模块
}
```

---

## rabbit-models (数据层)

### UiData 枚举

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiData {
    // 通用日志（HTTP/TFTP/Chat/Ping/Scan）
    Log(Module, String),

    // Ping 专用
    PingStats(String),        // "Tx 10 Rx 9 Loss 10% Min 1ms Max 5ms Avg 2ms"
    PingState { address: String, progress: u32, total: u32, color: String },

    // Scan 专用
    ScanProgress(String),       // "Progress: 50% - Found 5 hosts"

    // Plan 专用
    PlanReminder(String),      // 计划到期提醒

    // Chat 专用
    ChatMessage(String, String), // (用户名, 消息)
    ChatUserList(Vec<String>), // ⚠️ 待改：从逗号分隔改为数组

    // 服务状态更新（核心：业务状态通知）
    // ⚠️ 待扩展：增加可选原因字符串
    ServiceStatus(Module, bool, Option<String>), // (模块, 是否运行中, 原因)

    // 错误通知
    Error(Module, String),
}
```

> **源码位置**: `rabbit-models/src/lib.rs`（权威定义）
> 
> **⚠️ 待修订**（见 `data-flow-design.md` §5.2）：
> - `ChatUserList` 从逗号分隔改为 `Vec<String>`
> - `ServiceStatus` 增加可选原因字符串 `Option<String>`

### 配置模型

| 结构 | 说明 |
|------|------|
| `AppConfig` | 应用配置（仅含 modules） |
| `ModuleConfigs` | 所有模块配置（HashMap） |
| `ConfigValue` | 配置值类型（String/Integer/Boolean/Array） |

**接口**：
```rust
// 读取
modules.get_string(module, key)
modules.get_integer(module, key)
modules.get_bool(module, key)
modules.get_array(module, key)

// 写入
modules.insert(module, key, value)
```

---

## rabbit-platform (基础设施层)

### 主要文件

| 文件 | 职责 |
|------|------|
| `config.rs` | 配置加载/保存 |
| `lib.rs` | 平台接口统一导出 |
| `autostart.rs` | 开机自启 |
| `elevation.rs` | 权限提升（Windows） |
| `notification.rs` | 系统通知 |
| `taskbar.rs` | 任务栏集成 |
| `shell.rs` | Shell 操作 |
| `dialog.rs` | 对话框 |
| `network.rs` | 网络工具 |
| `ping.rs` | 平台级 Ping（Windows） |

**接口**：
```rust
pub fn load_config() -> Result<AppConfig>
pub fn save_config(config: &AppConfig) -> Result<()>
pub fn update_config<F>(modifier: F) -> Result<()>
pub fn get_config_dir() -> Result<PathBuf>
pub fn get_data_dir() -> Result<PathBuf>
```

---

## 数据流设计

详见 [data-flow-design.md](../doc/data-flow-design.md)（v2.0 已重写）。

核心机制：
- **推送代替轮询**：Service 通过 Channel 主动推送数据
- **ServiceStatus 接口**：统一处理业务状态更新（启动/停止/完成）
- **线程安全**：Channel 跨线程安全，ui_state 集中处理 UI 更新

### 业务状态更新场景

| 场景 | 服务 | 发送的消息 | UI 更新 |
|------|------|----------|----------|
| Ping 启动 | PingService | `ServiceStatus(Ping, true)` | 按钮变 "Stop"，窗口标题设为目标地址 |
| Ping count 达到 | PingService | `ServiceStatus(Ping, false)` | 按钮变 "Start"，窗口标题重置为 "Rabbit" |
| Scan 完成 | ScanService | `ServiceStatus(Scan, false)` | 按钮从 "Stop" → "Start" |
| 手动停止 | 任意 Service | `ServiceStatus(Module, false)` | 对应按钮状态更新 |

### 模块依赖关系

```
rabbit-app
    ├── rabbit-models (UiData, AppConfig)
    ├── rabbit-core  (服务 + ui_channel)
    └── rabbit-platform (配置加载)

rabbit-core
    ├── rabbit-models (UiData, Module)
    └── rabbit-platform (配置加载)

rabbit-platform
    └── rabbit-models (配置模型)
```

---

## 配置访问方式

### 通用模块配置

所有模块配置统一使用 HashMap 方式：

```rust
// 读取
config.modules.get_string("global", "language")
config.modules.get_integer("http", "port")
config.modules.get_bool("ping", "stoponloss")

// 写入
config.modules.insert("global", "systray", ConfigValue::Boolean(true))
```

### TFTP 配置字段

| 模块 | 字段 | 类型 | 默认值 | 说明 |
|------|------|------|--------|------|
| tftpd | timeout | Integer | 200 | 超时秒数 |
| tftpd | maxretry | Integer | 10 | 最大重试次数 |
| tftpd | blksize | Integer | 512 | 块大小 |
| tftpd | override_conflicts | Boolean | false | 覆盖冲突文件 |
| tftpd | qsize | Integer | 2000 | 队列大小 |
| tftpd | qtimeout | Integer | 1000 | 队列超时(ms) |
| tftpd | fslog | Boolean | false | 文件服务日志 |
| tftpc | server_addr | String | "" | 服务器地址 |
| tftpc | port | Integer | 69 | 服务器端口 |

---

文档版本：6.0
创建日期：2026-04-16
更新日期：2026-04-30