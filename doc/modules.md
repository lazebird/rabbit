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
│   │       ├── app.rs        # 主应用入口
│   │       ├── view_model.rs # 统一配置管理
│   │       ├── ui/          # FLTK 界面组件
│   │       │   ├── mod.rs
│   │       │   ├── ping_tab.rs
│   │       │   ├── scan_tab.rs
│   │       │   ├── http_tab.rs
│   │       │   ├── tftpd_tab.rs
│   │       │   ├── tftpc_tab.rs
│   │       │   ├── plan_tab.rs
│   │       │   ├── chat_tab.rs
│   │       │   ├── settings_tab.rs
│   │       │   └── defaults.rs
│   │       └── ui_state.rs   # UI 状态管理
│   │
│   ├── rabbit-core/       # 业务服务
│   │   └── src/
│   │       ├── ping.rs     # Ping 服务
│   │       ├── scan.rs     # 扫描服务
│   │       ├── http.rs    # HTTP 服务
│   │       ├── tftpd.rs   # TFTP 服务器
│   │       ├── tftpc.rs   # TFTP 客户端
│   │       ├── chat.rs   # 聊天服务
│   │       └── plan.rs   # 计划服务
│   │
│   ├── rabbit-models/    # 数据模型
│   │   └── src/
│   │       ├── config.rs  # 配置模型
│   │       ├── ping.rs   # Ping 数据模型
│   │       ├── http.rs   # HTTP 数据模型
│   │       ├── tftp.rs   # TFTP 数据模型
│   │       ├── chat.rs  # Chat 数据模型
│   │       ├── scan.rs  # Scan 数据模型
│   │       └── plan.rs  # Plan 数据模型
│   │
│   └── rabbit-platform/  # 基础设施
│       └── src/
│           ├── config.rs   # 配置持久化
│           ├── lib.rs    # 平台接口
│           ├── autostart.rs
│           ├── elevation.rs
│           └── notification.rs
│
├── doc/
│   ├── architecture.md
│   ├── config-structure.md
│   ├── modules.md
│   └── progress.md
│
└── Cargo.toml
```

---

## rabbit-app (表现层 + 业务层入口)

### 主要文件

| 文件 | 职责 |
|------|------|
| `app.rs` | 主入口、事件处理、生命周期管理 |
| `view_model.rs` | 统一配置管理 |
| `ui/mod.rs` | UI 模块入口 |
| `ui/ping_tab.rs` | Ping 界面 |
| `ui_state.rs` | UI 状态同步 |

### 对外接口

```rust
// App 初始化
pub async fn new() -> anyhow::Result<Self>

// ViewModel 配置访问
pub fn get_config(&self) -> AppConfig
pub fn update_config(&mut self, config: AppConfig)
```

---

## rabbit-core (业务服务层)

所有服务统一接口：
```rust
pub async fn init(&mut self) -> Result<()>     // 初始化，从配置加载
pub async fn update(&mut self) -> Result<()>  // 切换状态：运行中→停止，或停止→运行
```

### PingService

| 项目 | 说明 |
|------|------|
| 文件 | `src/ping.rs` |
| 职责 | ICMP Ping 功能 |

### HttpService

| 项目 | 说明 |
|------|------|
| 文件 | `src/http.rs` |
| 职责 | HTTP 文件服务器 |

### TftpdService

| 项目 | 说明 |
|------|------|
| 文件 | `src/tftpd.rs` |
| 职责 | TFTP 服务器 |

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

---

## rabbit-models (数据层)

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

### 数据模型

| 模型 | 说明 |
|------|------|
| `PingSummary` | Ping 结果摘要 |
| `HttpAccessLog` | HTTP 访问日志 |
| `TftpTransfer` | TFTP 传输信息 |
| `TftpServerConfig` | TFTP 服务器配置 |
| `TftpClientConfig` | TFTP 客户端配置 |
| `ChatMessage` | 聊天消息 |
| `ChatConfig` | 聊天配置 |
| `ScanRange` | 扫描范围 |
| `ScannerConfig` | 扫描器配置 |

---

## rabbit-platform (基础设施层)

### 主要文件

| 文件 | 职责 |
|------|------|
| `config.rs` | 配置加载/保存 |
| `lib.rs` | 平���接口 |
| `autostart.rs` | 开机自启 |
| `notification.rs` | 系统通知 |

**接口**：
```rust
pub fn load_config() -> Result<AppConfig>
pub fn save_config(config: &AppConfig) -> Result<()>
pub fn update_config<F>(modifier: F) -> Result<()>
pub fn get_config_dir() -> Result<PathBuf>
pub fn get_data_dir() -> Result<PathBuf>
```

---

## 模块依赖关系

```
rabbit-app
    ├── rabbit_models (配置)
    ├── rabbit_core  (服务)
    └── rabbit_platform (平台)

rabbit-core
    ├── rabbit_models (数据模型)
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

文档版本：3.0
创建日期：2026-04-16
更新日期：2026-04-24