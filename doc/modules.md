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
│   │       │   └── ui_refresh.rs # 集中式 UI 刷新
│   │       └── upgrade/         # 升级模块
│   │           ├── mod.rs
│   │           ├── models.rs
│   │           ├── downloader.rs
│   │           └── installer.rs
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
| `view_model.rs` | 统一配置管理 |
| `ui_state.rs` | UI 状态同步（全局状态、刷新标记） |
| `ui_events.rs` | UI 事件系统（事件发送/接收） |
| `ui/ui_refresh.rs` | 集中式 UI 刷新（100ms 定时器） |

### 对外接口

```rust
// App 初始化
pub async fn new() -> anyhow::Result<Self>
pub async fn run(&mut self) -> anyhow::Result<()>

// ViewModel 配置访问
pub fn get_config(&self) -> AppConfig
pub fn update_config(&mut self, config: AppConfig)
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
pub async fn init(&mut self) -> Result<()>     // 初始化
pub async fn update(&mut self) -> ServiceUpdateResult  // 切换状态
pub async fn is_running(&self) -> bool         // 运行状态
pub async fn stop(&mut self) -> Result<()>     // 停止
```

### Channel 推送接口

服务通过 Channel 向 UI 推送数据：

```rust
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
```

### PingService

| 项目 | 说明 |
|------|------|
| 文件 | `src/ping.rs` |
| 职责 | ICMP Ping 功能 |
| 依赖 | `surge-ping` |

**接口**：
```rust
pub fn new() -> Self
pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self
pub async fn init(&mut self) -> Result<()>
pub async fn start(&mut self) -> Result<()>
pub async fn stop(&mut self) -> Result<()>
pub async fn is_running(&self) -> bool
pub async fn add_target(&self, target: PingTarget) -> Result<()>
pub async fn update(&mut self) -> ServiceUpdateResult
```

### HttpService

| 项目 | 说明 |
|------|------|
| 文件 | `src/http.rs` |
| 职责 | HTTP 文件服务器 |
| 依赖 | `axum` |

### TftpdService

| 项目 | 说明 |
|------|------|
| 文件 | `src/tftpd.rs` |
| 职责 | TFTP 服务器 |
| 依赖 | `async-tftp` |

### TftpcService

| 项目 | 说明 |
|------|------|
| 文件 | `src/tftpc.rs` |
| 职责 | TFTP 客户端 |

### ScanService

| 项目 | 说明 |
|------|------|
| 文件 | `src/scan.rs` |
| 职责 | IP 扫描 |

### ChatService

| 项目 | 说明 |
|------|------|
| 文件 | `src/chat.rs` |
| 职责 | 局域网聊天 |

### PlanService

| 项目 | 说明 |
|------|------|
| 文件 | `src/plan.rs` |
| 职责 | 定时提醒 |

### UiChannel 模块

| 项目 | 说明 |
|------|------|
| 文件 | `src/ui_channel.rs` |
| 职责 | Channel 创建工具 |

```rust
pub struct UiChannels {
    pub ping_tx: mpsc::Sender<UiData>,
    pub http_tx: mpsc::Sender<UiData>,
    pub scan_tx: mpsc::Sender<UiData>,
    pub tftpd_tx: mpsc::Sender<UiData>,
    pub tftpc_tx: mpsc::Sender<UiData>,
    pub chat_tx: mpsc::Sender<UiData>,
    pub plan_tx: mpsc::Sender<UiData>,
}
```

---

## rabbit-models (数据层)

### UiData 枚举

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiData {
    // 通用日志
    Log(Module, String),
    
    // Ping 专用
    PingStats(String),
    PingState { address: String, progress: u32, total: u32, color: String },
    
    // Scan 专用
    ScanProgress(String),
    
    // Plan 专用
    PlanReminder(String),
    
    // Chat 专用
    ChatMessage(String, String),
    ChatUserList(String),
    
    // 错误
    Error(Module, String),
    
    // 通用状态更新
    ServiceStatus(Module, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}
```

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

### Channel 推送模式

```
┌─────────────┐         Channel          ┌─────────────┐
│  Service    │ ──────────────────────▶  │   app.rs    │
│  (tokio)    │    UiData 枚举           │  (receiver) │
└─────────────┘                          └──────┬──────┘
                                                │
                                          handle_ui_data()
                                                │
                                                ▼
                                        ┌──────────────┐
                                        │  ui_state    │
                                        │  (状态更新)  │
                                        └──────┬───────┘
                                               │
                                        fltk::app::awake()
                                               │
                                               ▼
                                        ┌──────────────┐
                                        │ ui_refresh   │
                                        │ (100ms 刷新) │
                                        └──────────────┘
```

### 优势

1. **推送代替轮询**：Service 产生数据后主动推送
2. **Channel 解耦**：使用 tokio channel 作为中介
3. **线程安全**：channel 跨线程安全
4. **实时响应**：数据立即到达 UI

---

## 模块依赖关系

```
rabbit-app
     ├── rabbit_models (UiData, AppConfig)
     ├── rabbit_core  (服务 + ui_channel)
     └── rabbit_platform (配置加载)

rabbit-core
     ├── rabbit_models (UiData, Module)
     └── rabbit_platform (配置加载)

rabbit_platform
     └── rabbit_models (配置模型)
```

---

## 配置访问方式

所有模块配置统一使用 HashMap 方式：

```rust
// 读取
config.modules.get_string("global", "language")
config.modules.get_integer("http", "port")
config.modules.get_bool("ping", "stoponloss")

// 写入
config.modules.insert("global", "systray", ConfigValue::Boolean(true))
```

---

文档版本：4.0
创建日期：2026-04-16
更新日期：2026-04-25