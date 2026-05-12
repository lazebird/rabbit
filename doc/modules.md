# Rabbit 模块化结构文档

## 项目架构

```
┌────────────────────────────────────────────────────────┐
│                    表现层 (Presentation)                 │
│     app (FLTK UI + ViewModel + 事件驱动刷新)              │
├────────────────────────────────────────────────────────┤
│                    业务层 (Business)                     │
│     service (PingService / HttpService / ...)           │
├────────────────────────────────────────────────────────┤
│                  基础设施层 (Infrastructure)              │
│     ┌──────────┬──────────────┬──────────────────┐      │
│     │  schema   │   config     │    adapter        │      │
│     │ (数据模型) │ (配置管理)    │  (平台适配层)      │      │
│     └──────────┴──────────────┴──────────────────┘      │
└────────────────────────────────────────────────────────┘
```

### 依赖关系

```
app ─→ service, rabbit-config, rabbit-diag, adapter, schema
                │
service ───────→ schema, rabbit-config     (不依赖 adapter)
                │
adapter ───────→ schema, rabbit-diag        (纯平台适配，零业务逻辑)
                │
rabbit-config ──→ schema, serde, toml       (配置管理)
                │
rabbit-diag ──── (无外部依赖)
                │
schema ──────── (无外部依赖)
```

> `rabbit-config` 和 `rabbit-diag` 分别为配置管理和诊断日志的独立 crate。
> `adapter` 层通过 `rabbit-diag` 记录平台相关的诊断日志。

---

## 模块结构

```
rabbit/
├── crates/
│   ├── app/              # 主入口 + FLTK UI（表现层）
│   ├── service/          # 业务服务（业务层）
│   ├── schema/           # 数据模型（数据层，无外部依赖）
│   ├── adapter/          # 平台适配层（基础设施，集中所有 #[cfg]）
│   ├── rabbit-config/    # 配置持久化（基础设施）
│   └── rabbit-diag/      # 诊断日志（基础设施）
│
├── doc/                  # 文档
├── tests/                # 集成测试
├── release/              # 发布脚本和配置
└── Cargo.toml            # workspace 定义
```

---

## app crate（表现层）

**crate 名**: `app`（二进制: `rabbit`）
**依赖**: `service`, `adapter`, `schema`, `fltk`, `tokio`, `tracing`
**平台条件依赖**: 无 — 全部委托至 `adapter`

**职责**：程序入口、FLTK UI 构建、用户交互处理、UI 状态管理、事件分发、服务生命周期管理、升级管理。

### 文件结构

```
app/src/
├── main.rs              # 程序入口：运行时初始化、提权、启动事件循环
├── lib.rs               # 库入口，导出所有模块
├── app.rs               # App 主控制器：窗口构建、事件处理、服务编排
├── view_model.rs        # AppViewModel：配置的缓存/读写代理
├── ui_state.rs          # UiState：全局 UI 状态（文本缓冲、运行标记）
├── ui_events.rs         # UiEvent 事件系统（FLTK → async 桥接）
│                       # lifecycle/icon → adapter crate（重构后已迁移）
├── ui/                  # FLTK 界面组件
│   ├── mod.rs           # UI 模块入口 + TabComponent trait
│   ├── ping_tab.rs      # Ping 功能标签页
│   ├── scan_tab.rs      # IP 扫描标签页
│   ├── http_tab.rs      # HTTP 服务器标签页
│   ├── tftpd_tab.rs     # TFTP 服务器标签页
│   ├── tftpc_tab.rs     # TFTP 客户端标签页
│   ├── plan_tab.rs      # 计划任务标签页
│   ├── chat_tab.rs      # 局域网聊天标签页
│   ├── settings_tab.rs  # 设置标签页 + 版本更新检查
│   ├── reminder_window.rs # 计划提醒弹窗
│   ├── defaults.rs      # 各模块配置默认值（从 config 读取）
│   ├── styles.rs        # UI 样式定义（颜色、间距、格式化）
│   └── ui_refresh.rs    # 集中式 UI 刷新管理器（事件驱动）
│
├── upgrade/             # 升级管理
│   ├── mod.rs           # 模块入口，导出公共类型
│   ├── models.rs        # 版本/平台数据模型 + 平台检测（委托 adapter）
│   └── downloader.rs    # 版本清单下载 + 更新包下载
│
├── resources/
│   └── icon.ico         # 应用图标
│
├── build.rs             # 构建脚本（Windows 资源嵌入）
├── rabbit-app.rc        # Windows 资源文件
└── rabbit.manifest      # Windows 清单文件
```

> **架构合规性**：`app/` 层已消除所有 `#[cfg(target_os)]` 条件编译和平台条件依赖。systray、tray_helper、installer、current_platform 等平台相关模块已全部移至 `adapter` 层。

### 核心类型

#### App（app.rs）

```rust
pub struct App {
    view_model: Arc<RwLock<AppViewModel>>,
    // 所有业务服务
    ping_service: Arc<RwLock<PingService>>,
    http_service: Arc<RwLock<HttpService>>,
    tftp_server_service: Arc<RwLock<TftpdService>>,
    tftp_client_service: Arc<RwLock<TftpcService>>,
    plan_service: Arc<RwLock<PlanService>>,
    chat_service: Arc<RwLock<ChatService>>,
    scan_service: Arc<RwLock<ScanService>>,
    // 生命周期控制
    lifecycle: Lifecycle,
    // UI 通道
    ui_channels: UiChannels,
    event_rx: UnboundedReceiver<UiEvent>,
}

impl App {
    pub async fn new() -> anyhow::Result<Self>   // 初始化
    pub async fn run(&mut self) -> anyhow::Result<()>  // 运行主循环
}
```

#### AppViewModel（view_model.rs）

配置的内存缓存代理。持有 `AppConfig` 并提供类型安全读写接口。

```rust
pub struct AppViewModel { config: AppConfig }

impl AppViewModel {
    pub fn new(config: AppConfig) -> Self
    pub fn get_config(&self) -> AppConfig
    pub fn set_config(&mut self, config: AppConfig)
    pub fn save(&self) -> Result<()>              // 持久化到 config crate
    pub fn update_and_save(&mut self, AppConfig) -> Result<()>

    // 类型安全访问器
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

#### Lifecycle（lifecycle.rs）

应用生命周期控制器。提供统一的应用退出管理，替代之前的多个 `AtomicBool` 分散管理。

```rust
#[derive(Clone)]
pub struct Lifecycle { /* Arc<Inner> */ }

impl Lifecycle {
    pub fn new() -> Self
    pub fn request_shutdown(&self)                // 请求优雅退出（任意线程安全）
    pub fn is_shutdown_requested(&self) -> bool    // 检查是否已请求退出
    pub fn run_event_loop(&self)                  // 运行 FLTK 事件循环（阻塞直到 shutdown）
}
```

**信号流**：退出请求可来自 ESC 键、窗口关闭按钮、托盘 Quit 菜单、Ctrl+C
→ `Lifecycle::request_shutdown()` → `run_event_loop()` 退出。

#### UiEvent（ui_events.rs）

FLTK UI 回调 → async 事件循环 的桥接。

```rust
pub enum UiEvent {
    ModuleToggle { module: String },   // 模块启动/停止
    TftpClientPut { server, local, remote, options },
    TftpClientGet { server, local, remote, options },
    PlanAdd { date, time, cycle, unit, msg },
    PlanRemove { msg },
    ChatSend { message },
    ChatRefresh,
    ChatNotify,
    SettingsSave,
    VersionCheck,
}

pub fn init_event_system() -> UnboundedReceiver<UiEvent>
pub fn send_event(event: UiEvent)
```

#### UiState（ui_state.rs）

全局 UI 状态，线程安全。使用 `parking_lot::Mutex`。

```rust
pub struct UiState {
    pub ping_output: String,         // Ping 日志文本
    pub ping_stats: String,          // Ping 统计文本
    pub scan_output: String,         // Scan 日志
    pub http_log: String,            // HTTP 日志
    pub http_items: Vec<String>,     // HTTP 目录列表
    pub tftpd_log: String,           // TFTP 服务器日志
    pub tftpd_dirs: Vec<String>,     // TFTP 工作目录
    pub tftpc_log: String,           // TFTP 客户端日志
    pub plan_log: String,            // 计划日志
    pub plan_tasks: Vec<PlanTask>,   // 计划任务列表
    pub chat_log: String,            // 聊天日志
    pub chat_users: Vec<String>,     // 在线用户列表
    pub module_running: HashMap<String, bool>,  // 各模块运行状态
}
```

#### TabComponent Trait（ui/mod.rs）

所有标签页统一实现此 trait。

```rust
pub trait TabComponent {
    fn build(x: i32, y: i32, w: i32, h: i32, config: &AppConfig) -> Flex;
}
```

### 数据流：UI 事件处理

```
FLTK UI 回调
  │  send_event(UiEvent::ModuleToggle { module: "ping" })
  ▼
event_rx (tokio::mpsc::unbounded_channel)
  │  App::handle_event(event)
  ▼
async handler
  │  service.update() → ServiceUpdateResult
  │  save_config()
  ▼
Service 推送 UiData 到 channel
  │  tx.send(UiData::Log(Module::Ping, msg))
  ▼
App::handle_ui_data(data)
  │  fltk::app::awake_callback(|| ui_refresh::update_xxx(...))
  ▼
FLTK 主线程更新 UI
```

---

## service crate（业务层）

**crate 名**: `service`
**依赖**: `schema`, `config`, `tokio`, `axum`, `async-tftp`, `surge-ping` 等

**职责**：业务逻辑实现，所有服务的启动/停止/状态管理。**不依赖 `adapter` crate**，通过 `config` crate 读取配置，通过构造函数注入获取平台服务。

### 文件结构

```
service/src/
├── lib.rs           # 库入口，导出所有服务 + ServiceError + ServiceUpdateResult
├── ping.rs          # PingService：ICMP Ping
├── scan.rs          # ScanService：IP 扫描
├── http.rs          # HttpService：HTTP 文件服务器
├── tftpd.rs         # TftpdService：TFTP 服务器
├── tftpc.rs         # TftpcService：TFTP 客户端
├── chat.rs          # ChatService：UDP 广播聊天
├── plan.rs          # PlanService：计划任务提醒
└── ui_channel.rs    # UiChannels/UiReceivers + send_ui 工具函数
```

### 统一接口模式

所有服务遵循相同的接口约定：

```rust
// 构造
pub fn new() -> Self                              // 占位初始化
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self  // 绑定 UI 通道

// 生命周期
pub async fn update(&mut self) -> ServiceUpdateResult  // 切换状态（启动/停止）
pub async fn destroy(&mut self) -> Result<()>          // 销毁资源（退出时）
```

> ⚠️ `destroy()` 是所有服务的必选方法。退出时 app 层遍历所有服务调用 `destroy()` 释放资源。
> `send()` 是各服务内部向 UI 推送数据的工具方法，不属对外接口。
> 不对外暴露 `is_running()` 等状态查询方法——运行状态通过 `UiData::ServiceStatus` 事件驱动同步。

业务状态通过 `ServiceUpdateResult` 返回：

```rust
pub enum ServiceUpdateResult {
    Started(String),    // 启动成功
    Stopped(String),    // 停止成功
    Error(String),      // 操作失败
    NoChange,           // 状态未改变
}
```

配置通过 `config` crate 的全局 API 读取（各服务在其 `update()` 方法中读取需要的配置值）。

### 各服务详情

#### PingService

| 项目 | 说明 |
|------|------|
| 文件 | `ping.rs` |
| 职责 | ICMP Ping + 统计 + 任务栏状态回传 |
| 依赖 | `surge-ping` |
| 状态 | 通过 `PingState { address, progress, total, color }` 推送 UiData |

```rust
impl PingService {
    pub fn new() -> Self
    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
    pub async fn update(&mut self) -> ServiceUpdateResult  // 切换运行/停止
    pub async fn destroy(&mut self) -> Result<()>          // 释放资源
}
```

#### HttpService

| 项目 | 说明 |
|------|------|
| 文件 | `http.rs` |
| 职责 | HTTP 静态文件服务器，支持目录浏览、视频播放 |
| 依赖 | `axum`, `tower`, `tower-http` |

```rust
impl HttpService {
    pub fn new() -> Self
    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
    pub async fn update(&mut self) -> ServiceUpdateResult
    pub async fn destroy(&mut self) -> Result<()>
}
```

#### TftpdService

| 项目 | 说明 |
|------|------|
| 文件 | `tftpd.rs` |
| 职责 | TFTP 文件服务器 |
| 依赖 | `async-tftp` |

```rust
impl TftpdService {
    pub fn new() -> Self
    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
    pub async fn update(&mut self) -> ServiceUpdateResult
    pub async fn destroy(&mut self) -> Result<()>
}
```

#### TftpcService

| 项目 | 说明 |
|------|------|
| 文件 | `tftpc.rs` |
| 职责 | TFTP 客户端（文件上传/下载） |

```rust
impl TftpcService {
    pub fn new() -> Self
    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
    pub async fn update(&mut self) -> ServiceUpdateResult   // 启动/停止（当前无状态，统一接口占位）
    pub async fn destroy(&mut self) -> Result<()>           // 释放资源
    pub async fn put(&self, local_path: &str, remote_filename: &str) -> Result<String>
    pub async fn get(&self, remote_filename: &str, local_path: &str) -> Result<String>
}
```

> ⚠️ **代码缺口**：当前 `TftpcService` 的实际实现缺少 `update()` 和 `destroy()` 方法。如果 app 层统一调用所有服务的 `update()`/`destroy()`，需要为 TftpcService 补充这两个方法（即使内部为空实现）。

#### ScanService

| 项目 | 说明 |
|------|------|
| 文件 | `scan.rs` |
| 职责 | IP 扫描（TCP 端口探测 + DNS 反向解析 + ARP MAC 查询） |
| 网络依赖 | 通过 `NetworkProvider` trait 获取 MAC 地址 |

```rust
impl ScanService {
    pub fn new() -> Self
    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
    pub async fn update(&mut self) -> ServiceUpdateResult
    pub async fn destroy(&mut self) -> Result<()>
    pub async fn cancel(&mut self) -> Result<()>       // 取消正在进行的扫描
}
```

#### ChatService

| 项目 | 说明 |
|------|------|
| 文件 | `chat.rs` |
| 职责 | UDP 广播局域网聊天 |

```rust
impl ChatService {
    pub fn new() -> Self
    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
    pub async fn update(&mut self) -> ServiceUpdateResult
    pub async fn destroy(&mut self) -> Result<()>
    pub async fn send_text(&self, content: &str) -> Result<()>
    pub async fn refresh_users(&self) -> Result<()>
}
```

#### PlanService

| 项目 | 说明 |
|------|------|
| 文件 | `plan.rs` |
| 职责 | 计划任务定时提醒 |

```rust
impl PlanService {
    pub fn new() -> Self
    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
    pub async fn update(&mut self) -> ServiceUpdateResult
    pub async fn destroy(&mut self) -> Result<()>
    pub async fn add_task(&self, date: &str, time: &str, cycle: i32, unit: &str, msg: &str, override_conflict: bool) -> Result<bool>
    pub async fn remove_task(&self, msg: &str) -> Result<()>
}
```

### UiChannel 模块

| 项目 | 说明 |
|------|------|
| 文件 | `ui_channel.rs` |
| 职责 | 统一管理所有业务的 UI 通道 + 工具函数 |

```rust
// 通道集合（发送端）
pub struct UiChannels {
    pub ping_tx: mpsc::Sender<UiData>,
    pub http_tx: mpsc::Sender<UiData>,
    pub scan_tx: mpsc::Sender<UiData>,
    pub tftpd_tx: mpsc::Sender<UiData>,
    pub tftpc_tx: mpsc::Sender<UiData>,
    pub chat_tx: mpsc::Sender<UiData>,
    pub plan_tx: mpsc::Sender<UiData>,
}

// 通道集合（接收端）
pub struct UiReceivers {
    pub ping: mpsc::Receiver<UiData>,
    pub http: mpsc::Receiver<UiData>,
    pub scan: mpsc::Receiver<UiData>,
    // ...
}

// 发送辅助
pub async fn send_ui(tx: &Option<mpsc::Sender<UiData>>, data: UiData)

// 创建单通道（用于独立 service）
pub fn create_channel() -> (mpsc::Sender<UiData>, mpsc::Receiver<UiData>)

// re-export schema::UiData, schema::Module
pub use schema::{Module, UiData};
```

---

## schema crate（数据模型层）

**crate 名**: `schema`
**依赖**: `serde`, `chrono`

**职责**：定义所有跨模块共享的数据结构和枚举。**无外部平台依赖**，可作为纯数据契约使用。

### 文件结构

```
schema/src/
├── lib.rs           # 入口 + UiData/Module 枚举定义
├── config.rs        # AppConfig / ModuleConfigs / ConfigValue / PlanTask / WindowConfig + keys 常量
├── network.rs       # NetworkProvider trait（依赖反转接口，service 不依赖 adapter）
└── scan.rs          # ScanRange / ScannerConfig / ScannerState
```

### 核心类型

#### UiData 枚举 — 统一数据通道消息

所有服务通过此枚举向后端推送数据。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiData {
    Log(Module, String),                                    // 通用日志
    PingStats(String),                                      // Tx 10 Rx 9 Loss 10%
    PingState { address: String, progress: u32, total: u32, color: String },
    ScanProgress(String),                                   // 扫描进度
    PlanReminder(String),                                   // 计划到期提醒
    ChatMessage(String, String),                            // (用户名, 消息)
    ChatUserList(Vec<String>),                              // 在线用户列表
    ServiceStatus(Module, bool, Option<String>),            // (模块, 是否运行, 原因)
    Error(Module, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}
```

#### AppConfig — 配置模型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub modules: ModuleConfigs,
}

impl AppConfig {
    pub fn merge_defaults(&mut self)  // 补充缺失的默认值
}

pub struct ModuleConfigs {
    pub global: HashMap<String, ConfigValue>,
    pub ping: HashMap<String, ConfigValue>,
    pub scan: HashMap<String, ConfigValue>,
    pub http: HashMap<String, ConfigValue>,
    pub tftpd: HashMap<String, ConfigValue>,
    pub tftpc: HashMap<String, ConfigValue>,
    pub plan: HashMap<String, ConfigValue>,
    pub chat: HashMap<String, ConfigValue>,
}
```

`ConfigValue` 支持四种变体：`String`, `Integer(i64)`, `Boolean`, `Array(Vec<ConfigValue>)`。

> **配置键常量**（`schema::config::keys` 模块）：所有模块配置键现已定义为类型安全常量（如 `keys::ping::TARGET`、`keys::http::PORT`），替换了散落在代码中的魔术字符串。新增键需在对应子模块中补充常量定义。

#### 扫描数据模型

```rust
pub struct ScanRange {
    pub start: Ipv4Addr,
    pub end: Ipv4Addr,
    pub port: u16,
}

pub struct ScannerConfig {
    pub timeout_ms: u64,
    pub concurrent: usize,
    pub retry_count: u32,
}

pub enum ScannerState {
    Idle, Scanning { progress: u8 }, Completed, Cancelled, Error,
}
```

---

## adapter crate（平台适配层）

**crate 名**: `adapter`
**依赖**: `schema`, `fltk`, `dirs`, `xelevate`, `libc`, `ksni`(linux), `tray-icon`(non-linux), `windows-sys`(windows)

**职责**: 封装所有平台相关功能，对外提供**平台无关的统一抽象接口**。

**原则**：
- 零业务逻辑 — 只做平台适配
- 不暴露平台类型（如 HWND）到接口中
- 所有 `#[cfg(…)]` 隐藏在实现内部

### 设计目标

```
其他业务模块（app）
  │
  └──→ adapter::*    ← 统一的平台无关 API
          ├── systray::init/cleanup/update     // 系统托盘统一入口
          ├── tray::init_systray/remove_systray // 平台分派实现
          ├── tray_helper::maybe_run_as_helper  // Linux 提权助手
          ├── tray_helper::pre_main_init        // 启动前初始化
          ├── dialog::open_url/file_manager     // 文件对话框
          ├── platform::current_platform        // 平台标识
          ├── platform::getuid                  // 用户 ID（跨平台）
          ├── installer::install_update         // 更新安装
          ├── install_platform_handlers         // X11 诊断
          ├── maybe_run_as_helper               // 统一入口
          ├── pre_main_init                     // 统一入口
          ├── set_autostart(bool)
          ├── set_window_on_top(bool)
          ├── show_notification(...)
          ├── check_ping_permission()
          ├── ensure_elevated()
          ├── update_taskbar(...)
          └── ...

      adapter::NetworkProvider      ← 网络查询
          ├── get_mac_from_arp(ip)
          ├── get_interfaces()
          └── get_local_ip()
```

### 文件结构（当前）

```
adapter/src/
├── lib.rs           # 统一导出 + 平台无关包裹函数（install_platform_handlers / maybe_run_as_helper / pre_main_init）
├── platform.rs      # 平台标识（current_platform）+ 用户 ID（getuid）
├── dialog.rs        # 文件对话框、URL/文件管理器打开
├── autostart.rs     # 开机自启（Win注册表/Linux .desktop/macOS plist）
├── shell.rs         # Shell 右键菜单集成（Windows 注册表）
├── taskbar.rs       # Windows 任务栏进度（ITaskbarList3 COM）
├── window.rs        # 窗口置顶/显隐（Win32 SetWindowPos/ShowWindow）
├── elevation.rs     # 提权（Linux sudo VAR=value / Windows UAC）
├── notification.rs  # 系统通知（Linux notify-send / Win PowerShell / macOS osascript）
├── ping.rs          # Ping 权限检查（Linux CAP_NET_RAW / Win 管理员）
├── network.rs       # 网络接口枚举 + ARP MAC 查询（平台特定实现）
├── lifecycle.rs     # 应用生命周期控制器
├── icon.rs          # 图标数据
├── installer.rs     # 更新安装器（从 app 移入）
├── systray.rs       # [新增] 统一系统托盘生命周期（init/cleanup/update）
│
├── tray/            # 系统托盘平台实现
│   ├── mod.rs       # 平台分派（#[cfg] 分发）
│   ├── linux.rs     # ksni StatusNotifierItem（D-Bus）
│   └── non_linux.rs # tray-icon（Win/macOS 原生）
│
├── tray_helper.rs   # 提权后托盘助手（Linux Unix socket IPC）
└── x11_diag.rs      # X11 错误诊断（#![cfg(target_os = "linux")]）
```

### 关键架构变更（P0+P1+P2）

| 变更 | 说明 |
|------|------|
| `systray.rs` (新增) | 从 app 层移入，统一封装 `init()`/`cleanup()`/`update()`，内部使用 `#[cfg]` 分平台 |
| `installer.rs` (新增) | 从 `app/src/upgrade/` 整体移入，替换脚本逻辑不变 |
| `platform::current_platform()` (新增) | 原 `upgrade/models.rs` 中的平台检测函数委托至此 |
| `platform::getuid()` (新增) | 封装 `libc::getuid()`，非 Linux 返回 `None` |
| `lib.rs::install_platform_handlers()` | 内部 `#[cfg(target_os = "linux")]` 保护 X11 诊断安装 |
| `lib.rs::maybe_run_as_helper()` | 内部 `#[cfg]` 保护，非 Linux 空操作，供 main.rs 统一调用 |
| `lib.rs::pre_main_init()` | 内部 `#[cfg]` 保护，非 Linux/debug 空操作 |
| `app/Cargo.toml` | 删除 `ksni`/`libc`/`windows-sys` 三项平台条件依赖 |

### 当前公共 API

```rust
// ─── 平台标识 ───────────────────────────────────
pub fn current_platform() -> &'static str         // "linux-x64", "windows-arm64" ...
pub fn getuid() -> Option<u32>                    // Linux: Some(uid), 其他: None

// ─── 统一入口（lib.rs 包裹函数） ─────────────────
pub fn maybe_run_as_helper()                      // 内部 #[cfg] 保护
pub fn pre_main_init()                            // 内部 #[cfg] 保护
pub fn install_platform_handlers()                // 内部 #[cfg] 保护

// ─── 系统托盘 ──────────────────────────────────
pub mod systray {
    pub async fn init(lifecycle, enabled, main_win) // 统一初始化
    pub fn cleanup()                                 // 统一清理
    pub async fn update(enabled)                     // 运行时开关
}
pub mod tray {
    pub fn init_systray(lifecycle) -> Result<()>
    pub fn remove_systray()
    pub fn update_systray(enabled)
    pub fn set_lifecycle(lifecycle)
    pub fn set_main_window(win)
    pub fn is_active() -> bool
}

// ─── 托盘助手（仅 Linux） ──────────────────────
pub mod tray_helper {
    pub fn maybe_run_as_helper()                   // --tray-helper 模式入口
    pub fn pre_main_init()                         // 启动前 spawn helper
    pub fn spawn() -> io::Result<()>
    pub fn connect(lifecycle, show_win, hide_win) -> bool
    pub fn hide_tray() / show_tray()
    pub fn shutdown_helper()
    pub fn is_connected() -> bool
}

// ─── 更新安装 ──────────────────────────────────
pub mod installer {
    pub fn install_update(path, sha256) -> Result<(), String> // 内部 #[cfg] 分平台
}

// ─── 文件对话框 / URL 打开 ─────────────────────
pub fn open_file_dialog(...) -> Result<Option<PathBuf>>
pub fn open_folder_dialog(...) -> Result<Option<PathBuf>>
pub fn open_url(url: &str) -> Result<()>
pub fn open_file_manager(path: &str) -> Result<()>

// ─── 平台服务 ──────────────────────────────────
pub fn set_autostart(enabled: bool) -> Result<()>
pub fn set_shell_integration(enabled: bool, exe_path: &str) -> Result<(), String>
pub fn set_window_on_top(hwnd: usize, on_top: bool)
pub fn hide_window(hwnd: usize) / show_window(hwnd: usize)
pub fn show_notification(title: &str, message: &str) -> Result<()>
pub fn show_task_reminder(title: &str, description: Option<&str>) -> Result<()>
pub fn is_elevated() -> bool
pub fn ensure_elevated() -> Result<(), ElevationError>
pub fn check_ping_permission() -> bool

// ─── 网络 ──────────────────────────────────────
pub fn get_mac_from_arp(ip: Ipv4Addr) -> Option<String>
pub async fn get_interfaces() -> Result<Vec<NetworkInterface>>
pub fn get_local_ip() -> Option<Ipv4Addr>

// ─── 任务栏 ────────────────────────────────────
pub struct TaskbarProgress;
impl TaskbarProgress {
    pub fn clear()
    pub fn update(progress: u32, total: u32, color: &str)
    pub fn set_main_window_hwnd(hwnd: usize)
}

// ─── 生命周期 ──────────────────────────────────
pub use lifecycle::Lifecycle;

// ─── 错误类型 ──────────────────────────────────
pub enum PlatformError {
    Io(std::io::Error),
    Config(String),
    Network(String),
    NotSupported,
}
```

---

## rabbit-config crate（配置管理层）

**crate 名**: `rabbit-config`
**依赖**: `schema`, `serde`, `toml`, `dirs`, `thiserror`

**职责**：配置文件的加载、保存、缓存、脏检查、默认值合并。使用 `dirs` crate 获取平台配置目录（纯路径，不涉及其他平台适配）。

---

## 数据流概览

### 关键数据流

#### 1. 用户操作 → 服务状态变更

```
用户点击 "Start" 按钮
  │
  ▼
FLTK callback → send_event(UiEvent::ModuleToggle { module })
  │
  ▼
Event handler (async) → service.update()
  │                   ├── 从 config 读取参数
  │                   ├── 启动/停止服务
  │                   └── 返回 ServiceUpdateResult
  ▼
结果 → awake_callback 更新 UI
     → push UiData 到对应 channel
```

#### 2. 服务运行中 → UI 推送

```
Service 检测到状态变化
  │
  ▼
tx.send(UiData::Log(module, msg))
tx.send(UiData::PingStats(...))
tx.send(UiData::ServiceStatus(module, running, reason))
  │
  ▼
App::handle_ui_data() 接收
  │
  ├─ 更新 UiState 缓冲
  ├─ 通过 awake_callback 驱动 ui_refresh
  └─ 特殊处理：ServiceStatus → 更新按钮状态 / 标题栏 / 任务栏
```

#### 3. 退出流程

```
ESC / 关闭按钮 / Ctrl+C / 托盘 Quit
  │
  ▼
Lifecycle::request_shutdown()
  │
  ▼
循环检测 → run_event_loop() 退出
  │
  ▼
App::cleanup()
  ├─ 调用各 service.destroy()
  ├─ 移除托盘
  ├─ 关闭 tray helper
  └─ std::process::exit(0)
```

### 通道架构

```
                      服务层                       表现层
              ┌────────────────┐          ┌──────────────────┐
              │  PingService   │──UiData──│  app::handle_ui  │
              │  HttpService   │──UiData──│       │          │
              │  ScanService   │──UiData──│       │          │
              │  TftpdService  │──UiData──│  ┌────▼──────┐   │
              │  TftpcService  │──UiData──│  │  UiState   │   │
              │  ChatService   │──UiData──│  │  (缓冲)    │   │
              │  PlanService   │──UiData──│  └────┬──────┘   │
              └────────────────┘          │       │          │
                                          │  ┌────▼──────┐   │
                                          │  │ui_refresh │   │
                                          │  │(FLTK更新)  │   │
                                          │  └───────────┘   │
                                          └──────────────────┘

  FLTK → UiEvent → tokio channel → async handler → service
```

---

## 模块隔离原则

| 层 | crate | 可依赖 | 不可依赖 | 平台条件编译 |
|----|-------|--------|---------|-------------|
| 表现层 | `app` | service, adapter, schema | — | 零 `#[cfg]` |
| 业务层 | `service` | schema | adapter | 零 `#[cfg]` |
| 数据模型 | `schema` | —（仅 serde/chrono） | 任何平台依赖 | 零 `#[cfg]` |
| 平台适配 | `adapter` | schema, fltk | service | 全部 `#[cfg]` 集中于此 |
| 配置管理 | `config` | schema, dirs | adapter, service | 零 `#[cfg]` |

---

## 配置字段参考

### 模块配置键

| 模块 | 键 | 类型 | 默认值 | 说明 |
|------|-----|------|--------|------|
| global | language | String | "System" | 语言选择 |
| global | theme | String | "System" | 主题选择 |
| global | systray | Boolean | true | 系统托盘 |
| global | top | Boolean | false | 窗口置顶 |
| global | autostart | Boolean | false | 开机自启 |
| global | last_active_tab | Integer | 0 | 上次标签页 |
| global | window | String | JSON | 窗口位置大小 |
| ping | target | String | "1.1.1.1" | Ping 目标 |
| ping | interval | Integer | 1000 | 间隔(ms) |
| ping | count | Integer | -1 | 次数(-1=无限) |
| ping | stoponloss | Boolean | false | 丢包停止 |
| scan | start_ip | String | "192.168.1.1" | 起始 IP |
| scan | end_ip | String | "254" | 结束 IP |
| http | port | Integer | 8000 | 端口 |
| http | shell | Boolean | false | 右键菜单集成 |
| http | autoindex | Boolean | true | 目录浏览 |
| http | videoplay | Boolean | true | 视频播放 |
| tftpd | port | Integer | 69 | 端口 |
| tftpd | timeout | Integer | 200 | 超时秒数 |
| tftpd | maxretry | Integer | 10 | 最大重试 |
| tftpd | blksize | Integer | 512 | 块大小 |
| tftpd | qsize | Integer | 2000 | 队列大小 |
| tftpd | qtout | Integer | 1000 | 队列超时(ms) |
| tftpd | override | Boolean | false | 覆盖冲突 |
| tftpd | fslog | Boolean | false | 文件服务日志 |
| tftpc | server_addr | String | "127.0.0.1" | 服务器地址 |
| tftpc | server_port | Integer | 69 | 端口 |
| tftpc | timeout | Integer | 200 | 超时(ms) |
| tftpc | maxretry | Integer | 10 | 最大重试 |
| tftpc | blksize | Integer | 1024 | 块大小 |
| plan | override | Boolean | false | 覆盖冲突 |
| chat | username | String | "User@PC" | 用户名 |
| chat | port | Integer | 1314 | 端口 |
| chat | broadcast_addr | String | "255.255.255.255" | 广播地址 |

---

## 相关文档

| 文档 | 内容 |
|------|------|
| `doc/architecture.md` | 技术选型与架构设计 |
| `doc/data-flow-design.md` | 数据流设计 v2.0（通道架构） |
| `doc/config-structure.md` | 配置结构设计 |
| `doc/platform-refactoring-plan.md` | 平台适配层重构方案 |
| `doc/code-structure-optimization-plan.md` | 代码结构优化计划（P0+P1+P2 执行记录） |
| `doc/progress.md` | 开发进度跟踪 |
| `doc/requirements.md` | 功能需求与 UI 规格 |

---

文档版本：7.2
创建日期：2026-04-16
最后更新：2026-05-12
