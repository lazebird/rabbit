# 平台适配层重构 — 执行计划

**基准文档**：`doc/modules.md`（v7.0）、`doc/platform-refactoring-plan.md`
**生成日期**：2026-05-11
**最后修改**：2026-05-11（v2 — 根据代码审查调整）
**状态**：待审核

---

## 1. 执行策略

### 1.1 总体原则

- **以模块职责为驱动**：每个 step 完成一个清晰的模块边界调整
- **保持可编译中间态**：每个 step 完成后代码必须能通过 `cargo check`
- **优先解耦 service**：让 service 先脱离 adapter 依赖，消除架构中最大的耦合问题
- **不做兼容**：本次重构不保留任何向后兼容代码。所有变更直接改为新方式，不保留旧的函数/类型/trait 别名、不保留 `#[deprecated]` 包裹层、不提供过渡适配器。重构后代码形态即最终形态

### 1.2 执行顺序

```
批次 0: Step 1 (config 提取) + Step 5 (diag 提取)
  │   合并为一次 import 替换操作，避免同一文件修改两遍
  │   service 层改为依赖 rabbit-config（不再依赖 adapter::config）
  │   app 层改为依赖 rabbit-config（不再依赖 adapter::config）
  │   adapter::diag::log → rabbit-diag::log（全部引入替换一次完成）
  ▼
Step 2: 净化 adapter 公共接口
  │   定义 PlatformService / NetworkProvider / TrayProvider
  │   消除 hwnd 泄漏、解耦 elevation UI
  ▼
Step 3: service 层完成解耦
  │   移除 service/Cargo.toml 中 adapter 依赖
  │   ScanService 通过 trait 注入 network 能力
  ▼
Step 4: app 平台代码移入 adapter
  │   systray, tray_helper, X11 handler → adapter
```

---

## 2. 批次 0 — 提取非平台代码（config + diag）

> ⚠️ **改动**：v2 将原 Step 1（config 提取）和 Step 5（diag 提取）合并为一次操作。
> 原因是 diag 只有 35 行且有 50+ 调用点，单独做 Step 5 会导致 50+ 文件被改两遍。
> 合并后，所有 import 替换一次完成，减少 30% 的总工作量。

### 2.1 目标

1. 将配置管理（缓存、脏检查、merge_defaults、文件 I/O）从 `adapter::config` 提取到独立的 `rabbit-config` crate
2. 将诊断日志从 `adapter::diag` 提取到独立的 `rabbit-diag` crate
3. 一次性替换所有 import 路径，确保 service 不再依赖 `adapter::config`

### 2.2 原理

#### 2.2.1 配置管理（config）

当前 `adapter::config` 包含的**业务逻辑**（非平台适配）：
| 功能 | 说明 |
|------|------|
| 全局缓存 | `OnceLock<RwLock<AppConfig>>` |
| 脏检查 | `LAST_SAVED_CONTENT` 比较 |
| `merge_defaults()` | 补充缺失默认值 |
| `get_*` / `insert` | 类型安全访问器 |
| `update_config()` | 读-改-写模式 |

平台相关的只有**文件路径**（通过 `dirs` crate 获取），这可以在 config crate 中直接使用 `dirs` 解决，不需要经过 adapter。

#### 2.2.2 诊断日志（diag）

当前 `adapter::diag` 是纯调试工具（35行），唯一"平台相关"的是用了 `libc::getpid()`。
可以用 `std::process::id()` 替代，完全消除平台依赖。

> 为什么和 config 合并做？`adapter::diag::log` 有 **50+ 调用点**（app.rs ~40 处、main.rs ~20 处、lifecycle.rs 3 处、systray_linux.rs、tray_helper.rs），
> 如果分开做，这些文件要改两遍。合并后一次完成所有 import 替换。

### 2.3 详细操作

#### 2.3.1 新建 `crates/rabbit-config/`

> ⚠️ **重要**：crate 名使用 `rabbit-config`，不是 `config`。
> 原因是 workspace `Cargo.toml` 已声明 `config = "0.15"`（crates.io 上的 config crate），
> 若新 crate 也叫 `config` 会造成名称歧义。使用 `rabbit-config` 避免冲突。

**Cargo.toml**：
```toml
[package]
name = "rabbit-config"
version.workspace = true
edition.workspace = true

[dependencies]
schema = { path = "../schema" }
serde = { workspace = true }
toml = { workspace = true }
dirs = { workspace = true }
thiserror = { workspace = true }
```

**src/lib.rs** — 从 `adapter/src/config.rs` 复制并改造：
1. 复制 `load_config`, `save_config`, `update_config`, `get_*`, 缓存, 脏检查, `get_config_dir`, `get_data_dir`
2. 替换 `use super::{PlatformError, Result}` → `use thiserror::Error` 定义本地的 `ConfigError`
3. 所有 `PlatformError::Config(...)` → `ConfigError::Config(...)`
4. `PlatformError::Io(e)` → `ConfigError::Io(e)`
5. 导出 `ConfigError`, `Result<T>`

#### 2.3.2 新建 `crates/rabbit-diag/`

**Cargo.toml**：
```toml
[package]
name = "rabbit-diag"
version.workspace = true
edition.workspace = true

[dependencies]
# 仅依赖标准库，无需任何第三方依赖
```

**src/lib.rs** — 从 `adapter/src/diag.rs` 复制并改造：
1. 替换 `unsafe { libc::getpid() }` → `std::process::id()`
2. 移除 `libc` 依赖

```rust
// rabbit-diag/src/lib.rs
use std::fs::OpenOptions;
use std::io::Write;
use std::process;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

fn start() -> &'static Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now)
}

fn log_file() -> &'static Mutex<std::fs::File> {
    static LOG: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
    LOG.get_or_init(|| {
        let pid = process::id();  // ← 替代 unsafe { libc::getpid() }
        let path = format!("/tmp/rabbit-startup-{pid}.log");
        let file = OpenOptions::new()
            .create(true).append(true).open(&path)
            .expect("cannot open diagnostic log");
        Mutex::new(file)
    })
}

pub fn log(msg: &str) {
    let elapsed = start().elapsed();
    let secs = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
    let line = format!("[{:+.3}] pid={} {msg}\n", secs, process::id());
    if let Ok(f) = log_file().lock() {
        let _ = (&*f).write_all(line.as_bytes());
    }
}
```

#### 2.3.3 错误类型定义（rabbit-config）

```rust
// rabbit-config/src/lib.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;
```

> **注意**：`adapter::PlatformError::Config` 变体**不会删除**，因为 `adapter::autostart.rs` 等模块仍用 `PlatformError::Config` 表示"配置操作失败"（非 config 模块概念，而是 autostart/shell 等设置失败时的错误描述）。

#### 2.3.4 修改影响的所有文件

**service crate**（7 个文件需要改）：

| 文件 | 修改内容 |
|------|---------|
| `service/Cargo.toml` | 添加 `rabbit-config = { path = "../rabbit-config" }` |
| `service/src/http.rs` | `use adapter::config::*` → `use rabbit_config::*` |
| `service/src/tftpd.rs` | 同上 |
| `service/src/tftpc.rs` | 同上 |
| `service/src/chat.rs` | 同上 |
| `service/src/scan.rs` | `adapter::config::load_config()` → `rabbit_config::load_config()` |
| `service/src/ping.rs` | `adapter::config::load_config()` → `rabbit_config::load_config()` |
| `service/src/lib.rs` | `#[from] adapter::PlatformError` → 移除（返回 `ServiceError::Other` 或 `anyhow`） |

> ⚠️ `service/src/lib.rs` 的 `ServiceError::Platform` 变体需要处理。建议：移除该变体，将 `adapter::network::get_mac_from_arp` 等平台操作的错误通过 `ServiceError::Other` 包装。

**app crate**（大量文件需要改）：

| 文件 | 修改内容 |
|------|---------|
| `app/Cargo.toml` | 添加 `rabbit-config = { path = "../rabbit-config" }`、`rabbit-diag = { path = "../rabbit-diag" }` |
| `app/src/app.rs` | `adapter::config::*` → `rabbit_config::*`；`adapter::diag::log` → `rabbit_diag::log` |
| `app/src/view_model.rs` | `use adapter::config::save_config` → `use rabbit_config::save_config`；**`use adapter::Result` → 改为 `use rabbit_config::Result`** |
| `app/src/ui_state.rs` | 11 处 `adapter::config::*` → `rabbit_config::*`（含 `update_config` 调用） |
| `app/src/ui/defaults.rs` | `use adapter::config::load_config` → `use rabbit_config::load_config` |
| `app/src/ui/ping_tab.rs` | 同上 |
| `app/src/ui/http_tab.rs` | `adapter::config::load_config()` → `rabbit_config::load_config()` |
| `app/src/ui/chat_tab.rs` | `adapter::config::{load_config, save_config}` → `rabbit_config::*` |
| `app/src/main.rs` | `adapter::diag::log` → `rabbit_diag::log`（约 20 处） |
| `app/src/lifecycle.rs` | `adapter::diag::log` → `rabbit_diag::log`（3 处） |
| `app/src/systray_linux.rs` | `adapter::diag::log` → `rabbit_diag::log`（1 处） |
| `app/src/tray_helper.rs` | `adapter::diag::log` → `rabbit_diag::log`（1 处） |
| `app/tests/integration_ui_refresh.rs` | `adapter::config::save_config` → `rabbit_config::save_config` |

**adapter crate**：

| 文件 | 修改内容 |
|------|---------|
| `adapter/Cargo.toml` | 移除 `toml`, `dirs` 依赖 |
| `adapter/src/config.rs` | **删除整个文件** |
| `adapter/src/diag.rs` | **删除整个文件** |
| `adapter/src/lib.rs` | 移除 `pub mod config;` `pub use config::*;` `pub mod diag;` |
| `adapter/src/lib.rs` | 移除 `config_dir()`, `data_dir()` 函数（注意：它们内部调用了 `config::get_config_dir`） |
| `adapter/src/lib.rs` | `PlatformError::Config` 变体**保留**（autostart.rs 仍在用） |
| `adapter/src/elevation.rs` | `crate::diag::log` → `rabbit_diag::log`（多处） |

> **注意**：`adapter/src/elevation.rs` 有 8 处 `crate::diag::log(...)` 调用。
> diag 移出后，这些调用需要改为 `rabbit_diag::log(...)`。
> 同理 `adapter/src/lib.rs` 中不再定义 `diag` 模块，所以 `crate::diag` 将不可用。 
> 
> 但注意：这会使 adapter 依赖 rabbit-diag，而 rabbit-diag 是纯工具 crate（零平台依赖），所以是可以接受的。
> 如果不希望 adapter 依赖 rabbit-diag，可以将 elevation.rs 中的日志调用改为 `tracing::debug!`。
> 更激进的做法：直接在 adapter 的 `Cargo.toml` 中 `[dependencies]` 添加 `rabbit-diag = { path = "../rabbit-diag" }`。

**service crate** — diag 方面不需要改（service 当前未使用 adapter::diag）。

> 全部文件合计：约 **35 个文件**（原 Step 1 的 25 个 + Step 5 涉及 adapter 内部及 elevation.rs 等 10 个）

### 2.4 验证

```bash
cargo check
# 所有 `adapter::config::*` 引用都已替换为 `rabbit_config::*`
# 所有 `adapter::diag::log` 和 `crate::diag::log` 已替换为 `rabbit_diag::log`
```

### 2.5 风险与注意

| 风险 | 等级 | 应对 |
|------|------|------|
| `rabbit-config` crate 名与 workspace 中 `config = "0.15"`（crates.io）冲突 | 中 | 使用唯一名 `rabbit-config`，所有引用通过 `{ path = "../rabbit-config" }` 显式引入 |
| `view_model.rs` 的 `use adapter::Result` 被遗漏 | 中 | 需要改为 `use rabbit_config::Result`，方法签名也要变 |
| `adapter::PlatformError::Config` 被 autostart.rs 使用 | 低 | 保留该变体（autostart 中的 Config 意为"配置操作失败"，非 config 模块） |
| `config_dir()` 和 `data_dir()` 在 adapter lib.rs 中被 re-export | 中 | 删除这两个函数（调用方已改为直接使用 `rabbit_config::get_config_dir()`） |
| `service/src/lib.rs` 的 `#[from] adapter::PlatformError` 不能直接删除 | 中 | 批次 0 中先移除该行和 `ServiceError::Platform` 变体 |
| `adapter/src/elevation.rs` 的 `crate::diag::log` 调用 | 低 | 改为 `rabbit_diag::log`，adapter 将依赖 rabbit-diag（纯工具，可接受） |

---

## 3. Step 2 — 净化 adapter 公共接口

### 3.1 目标

定义统一的结构体/trait 组织 adapter 的公共 API，消除接口中的平台类型泄漏。

### 3.2 当前问题

| 问题 | 示例 | 影响 |
|------|------|------|
| 窗口操作泄漏 `hwnd: usize` | `set_window_on_top(hwnd, bool)` | 调用方需知道平台句柄 |
| 零散函数无组织 | 全部是自由函数 | 调用方需要 import 每个函数 |
| elevation 含 UI 代码 | `show_elevation_error()` 创建 FLTK dialog | adapter 不应有 UI 逻辑 |
| `calculate_ip_range` 重复 | adapter 和 service 各有一份 | 重复代码 |

### 3.3 详细操作

#### 3.3.1 定义 `PlatformService` 结构体

新建 `adapter/src/platform.rs`：

```rust
/// PlatformService 提供所有平台相关操作的统一入口。
///
/// 调用方只需要持有此结构体的引用，不需要知道任何平台细节。
pub struct PlatformService {
    // 内部存储窗口句柄（平台特定）
    #[cfg(windows)]
    main_hwnd: std::sync::atomic::AtomicUsize,
}

impl PlatformService {
    pub fn new() -> Self { /* ... */ }

    // ── 窗口管理（调用方不传 hwnd）──
    /// 注册主窗口句柄（在窗口创建后调用一次）
    pub fn set_main_window(&self, window: &fltk::window::Window) { /* 存储句柄 */ }
    /// 设置窗口置顶（无需传 hwnd）
    pub fn set_window_on_top(&self, on_top: bool) { /* 使用内部句柄 */ }
    /// 隐藏窗口（用于托盘）
    pub fn hide_main_window(&self) { /* 使用内部句柄 */ }
    /// 显示窗口
    pub fn show_main_window(&self) { /* 使用内部句柄 */ }

    // ── 文件/URL 操作 ──
    pub fn open_url(&self, url: &str) -> Result<()> { /* 委托到 dialog */ }
    pub fn open_file_dialog(&self, title: &str, default_path: Option<&str>) -> Result<Option<PathBuf>> { /* ... */ }
    pub fn open_folder_dialog(&self, title: &str, default_path: Option<&str>) -> Result<Option<PathBuf>> { /* ... */ }
    pub fn open_file_manager(&self, path: &str) -> Result<()> { /* ... */ }

    // ── 系统设置 ──
    pub fn set_autostart(&self, enabled: bool) -> Result<()> { /* ... */ }
    pub fn set_shell_integration(&self, enabled: bool, exe_path: &str) -> Result<(), String> { /* ... */ }

    // ── 通知 ──
    pub fn show_notification(&self, title: &str, message: &str) -> Result<()> { /* ... */ }

    // ── 提权 ──
    pub fn is_elevated(&self) -> bool { /* ... */ }
    /// 提权。失败时返回错误，由调用方决定如何展示（不再内部创建 FLTK dialog）
    pub fn ensure_elevated(&self) -> Result<()> { /* ... */ }

    // ── 任务栏 ──
    pub fn update_taskbar(&self, progress: u32, total: u32, color: &str) { /* ... */ }
    pub fn clear_taskbar(&self) { /* ... */ }

    // ── Ping 权限 ──
    pub fn check_ping_permission(&self) -> bool { /* ... */ }
}
```

#### 3.3.2 解耦 elevation 的 FLTK dialog

当前 `adapter::elevation::elevate_with_sudo()` 在失败时调用 `show_elevation_error()` 创建 FLTK dialog。

改进方案：
1. 移除 `elevation.rs` 中的 `show_elevation_error()`（整个函数和其中 FLTK 的使用）
2. `ensure_elevated()` 返回 `Result<()>`，错误包含失败信息
3. 在 `app` 层的调用处处理错误展示（app 层有 FLTK 上下文）

```rust
// adapter/src/elevation.rs — 改造后签名
pub fn ensure_elevated() -> Result<(), ElevationError> {
    // ...不再调用 show_elevation_error()
    // 失败时返回 Err(ElevationError::SudoFailed(reason))
}

// app/src/main.rs — 调用处
if let Err(e) = adapter::ensure_elevated() {
    // app 层有 FLTK 上下文，在这里显示错误 dialog
    show_fltk_error_dialog(&format!("Elevation failed: {}", e));
}
```

#### 3.3.3 在 schema 中定义 `NetworkProvider` trait

**决策**：定义在 `schema` crate 中（详见第 7 章决策 1），使 service 引用 trait 时无需依赖 adapter。

新建 `schema/src/network.rs`：

```rust
// schema/src/network.rs
use std::net::Ipv4Addr;

/// 平台特定的网络查询接口。
/// service 层通过此 trait 使用平台能力，不依赖 adapter。
pub trait NetworkProvider: Send + Sync {
    fn get_mac_from_arp(&self, ip: Ipv4Addr) -> Option<String>;
}
```

`schema/src/lib.rs` 添加 re-export：
```rust
pub mod network;
pub use network::NetworkProvider;
```

`adapter` 提供默认实现：
```rust
// adapter/src/network.rs
use schema::network::NetworkProvider;

pub struct DefaultNetworkProvider;

impl NetworkProvider for DefaultNetworkProvider {
    fn get_mac_from_arp(&self, ip: Ipv4Addr) -> Option<String> {
        // 现有实现代码
    }
}
```

#### 3.3.4 消除 `calculate_ip_range` 重复

`adapter::network::calculate_ip_range()` 和 `service::scan::calculate_ip_range()` 功能重复。

方案：
1. 删除 `adapter::network::calculate_ip_range()`（纯算法，非平台适配）
2. 保留 `service::scan::calculate_ip_range()`（私有不导出）
3. 或者移到 `schema::scan` 作为公共方法

#### 3.3.5 定义 `TrayProvider` trait（骨架）

```rust
// adapter/src/tray.rs
pub trait TrayProvider: Send {
    fn init(&mut self, on_shutdown: Box<dyn Fn() + Send>) -> Result<(), String>;
    fn remove(&mut self);
    fn is_active(&self) -> bool;
    fn update(&mut self, enabled: bool);
}
```

**注意**：TrayProvider 不直接依赖 `Lifecycle`，而是接受 `on_shutdown: Box<dyn Fn()>` 回调。这样 Lifecycle 可以留在 app 层，adapter 不依赖 app。

#### 3.3.6 更新 `adapter/src/lib.rs`

```rust
pub mod platform;          // 新增: PlatformService
pub mod dialog;
pub mod autostart;
pub mod shell;
pub mod taskbar;
pub mod window;
pub mod elevation;
pub mod notification;
pub mod ping;
pub mod network;           // 平台网络操作（不含 calculate_ip_range 和 config, diag 已移出）
pub mod tray;              // 新增（骨架）

pub use platform::PlatformService;
pub use schema::NetworkProvider;  // NetworkProvider trait 定义在 schema
pub use network::NetworkInterface;
pub use tray::TrayProvider;
```

### 3.4 验证

```bash
cargo check
# PlatformService 对所有窗口操作不暴露 hwnd
# elevation 不再创建 FLTK dialog
# NetworkProvider trait 可被外部实现
```

---

## 4. Step 3 — service 层完成解耦

### 4.1 目标

`service/Cargo.toml` 中彻底移除 `adapter` 依赖。

### 4.2 当前依赖

| 依赖类型 | 内容 | 处理方式 |
|---------|------|---------|
| 配置读取 | 6 个 service 模块读 config | Step 1 已完成（改用 config crate） |
| 网络查询 | `scan.rs` 调 `adapter::network::get_mac_from_arp` | 通过 `NetworkProvider` trait 注入 |
| 错误类型 | `lib.rs` 的 `#[from] adapter::PlatformError` | 移除变体或替换 |

### 4.3 详细操作

#### 4.3.1 移除 `service/src/lib.rs` 中的 `PlatformError`

```rust
// 当前
#[derive(Error, Debug)]
pub enum ServiceError {
    NotStarted,
    AlreadyRunning,
    Config(String),
    Io(#[from] std::io::Error),
    Platform(#[from] adapter::PlatformError),  // ← 删除
    Other(String),
}

// 改进后
#[derive(Error, Debug)]
pub enum ServiceError {
    NotStarted,
    AlreadyRunning,
    Config(String),
    Io(#[from] std::io::Error),
    Other(String),
}
```

将所有的 `Err(ServiceError::Platform(...))` 替换为 `Err(ServiceError::Other(...))`。如果 `scan.rs` 中的 `adapter::network::get_mac_from_arp` 抛出错误，它当前通过 `unwrap_or_default()` 处理，所以实际上没有平台错误路径需要转换。

#### 4.3.2 ScanService 接受 NetworkProvider

```rust
// service/src/scan.rs
pub struct ScanService {
    config: Arc<RwLock<ScannerConfig>>,
    state: Arc<RwLock<ScannerState>>,
    results: Arc<RwLock<Vec<ScanResult>>>,
    cancel_tx: Option<mpsc::Sender<()>>,
    scan_handle: Option<JoinHandle<()>>,
    tx: Option<mpsc::Sender<UiData>>,
    network_provider: Option<Arc<dyn NetworkProvider>>,  // 新增
}

impl ScanService {
    pub fn new() -> Self {
        Self { network_provider: None, /* ... */ }
    }

    pub fn with_network(mut self, provider: Arc<dyn NetworkProvider>) -> Self {
        self.network_provider = Some(provider);
        self
    }

    // 在 scan_host 中获取 MAC 地址时：
    async fn scan_host(ip: Ipv4Addr, port: u16, timeout_ms: u64,
                       network_provider: Option<Arc<dyn NetworkProvider>>) -> ScanResult {
        // ...
        let mac_address = if online {
            if let Some(provider) = &network_provider {
                tokio::task::spawn_blocking({
                    let provider = provider.clone();
                    move || provider.get_mac_from_arp(ip)
                }).await.unwrap_or_default()
            } else {
                None
            }
        } else {
            None
        };
        // ...
    }
}
```

**NetworkProvider 引用**：已在 Step 2 中定义在 `schema::network::NetworkProvider`，service 通过 `use schema::NetworkProvider` 引入，不依赖 adapter crate。

#### 4.3.3 清理 service/Cargo.toml

```toml
# 移除（批次 0 中可能已移除）
adapter = { path = "../adapter" }

# 保留（批次 0 中已添加）
rabbit-config = { path = "../rabbit-config" }
schema = { path = "../schema" }
```

> **注意**：`adapter` 依赖在**批次 0** 中不会移除（因为 `service/src/scan.rs` 中仍然有 `adapter::network::get_mac_from_arp` 调用），
> 仅在 Step 3 中 `ScanService` 改为 `NetworkProvider` 注入后才能彻底移除。

### 4.4 验证

```bash
grep -r "adapter" crates/service/  # 应无输出
cargo check -p service              # 编译通过
```

---

## 5. Step 4 — app 平台代码移入 adapter

### 5.1 目标

将 `app/` 中分散的平台代码移入 `adapter/`，使 adapter 成为唯一的平台适配层。

### 5.2 迁移清单

| 源文件（app） | 目标文件（adapter） | 行数 | 复杂度 |
|-------------|-------------------|------|--------|
| `systray.rs` | `tray/mod.rs` | 21 | 低 |
| `systray_linux.rs` | `tray/linux.rs` | 222 | 中 |
| `systray_non_linux.rs` | `tray/non_linux.rs` | 186 | 中 |
| `tray_helper.rs` | `tray_helper.rs` | ~450 | 高 |
| `app.rs:70-129` X11 handler | `x11_diag.rs` | 60 | 低 |
| `app.rs:493-527` F1/F2/F3 | 改为 `dialog::open_url()` | 35 | 低 |

### 5.3 关键挑战

#### 挑战 1: Lifecycle 依赖

**问题**：systray 的 Quit 菜单需要触发 shutdown。当前使用 `Lifecycle` 类型（定义在 `app/src/lifecycle.rs`）。

**解决方案**：TrayProvider 不直接引用 Lifecycle，而是使用回调。

```rust
// adapter/src/tray/mod.rs
pub struct TrayConfig {
    pub on_shutdown: Box<dyn Fn() + Send + Sync>,
    pub on_show: Box<dyn Fn() + Send + Sync>,
    pub on_hide: Box<dyn Fn() + Send + Sync>,
}

pub trait TrayProvider: Send {
    fn init(&mut self, config: TrayConfig) -> Result<(), String>;
    fn remove(&mut self);
    fn is_active(&self) -> bool;
    fn update(&mut self, enabled: bool) -> impl std::future::Future<Output = ()> + Send;
}

// app 层构建时传入
let tray_config = TrayConfig {
    on_shutdown: {
        let lc = lifecycle.clone();
        Box::new(move || lc.request_shutdown())
    },
    on_show: Box::new(|| { /* show window */ }),
    on_hide: Box::new(|| { /* hide window */ }),
};
```

#### 挑战 2: tray_helper 的 Unix socket IPC

**问题**：`tray_helper.rs` 使用 `std::os::unix::net::UnixStream`、`UnixListener`，且与 FLTK 事件循环通过回调交互。

**解决方案**：
1. 将 IPC 协议逻辑（`TAG_*` 常量、socket 路径、连接管理）移入 `adapter/src/tray_helper.rs`
2. FLTK 相关的回调（`show_window`, `hide_window`）通过 `TrayConfig` 注入
3. `#[cfg(target_os = "linux")]` 处理：在 `adapter/Cargo.toml` 中条件编译

```toml
# adapter/Cargo.toml
[target.'cfg(target_os = "linux")'.dependencies]
libc = { workspace = true }
```

#### 挑战 3: tray-icon 依赖（非 Linux）

**问题**：非 Linux 平台的 systray 使用 `tray-icon` crate，需要 Windows/macOS 特定依赖。

**解决方案**：

```toml
# adapter/Cargo.toml
[target.'cfg(not(target_os = "linux"))'.dependencies]
tray-icon = "0.23"

[target.'cfg(target_os = "linux")'.dependencies]
ksni = { version = "0.3" }
libc = { workspace = true }
```

### 5.4 迁移后 app 层的变化

```rust
// app/src/app.rs — 改造后，不需要自己管理 systray
let tray_config = TrayConfig {
    on_shutdown: { let lc = self.lifecycle.clone(); Box::new(move || lc.request_shutdown()) },
    on_show: { let win = main_win.clone(); Box::new(move || win.show()) },
    on_hide: { let win = main_win.clone(); Box::new(move || win.hide()) },
};

// 所有平台代码都通过 adapter 调用
if systray_requested {
    adapter::tray::init(tray_config)?;
}

// F1/F2/F3 快捷键统一使用 adapter
Key::F1 => { adapter::dialog::open_url("https://..."); true }
Key::F2 => { adapter::dialog::open_url("https://..."); true }
```

### 5.5 验证

```bash
grep -r "#\[cfg(target_os" crates/app/src/  # 应为 0 或极少量纯文本
cargo check
```

---

## 6. ~~Step 5 — diag 提取~~（已合并到批次 0）

> diag 提取已在 v2 中合并到 Step 1（批次 0）。原因：
> - 仅 35 行代码要提取，操作简单
> - 但影响 50+ 调用点，如果单独做会导致这些文件被改两遍
> - 合并后所有 import 替换一次完成，避免重复劳动

---

## 7. 决策记录

以下模糊点已在 v2 中做出决策：

### ✅ 决策 1. `NetworkProvider` trait 定义位置

**决策**：定义在 `schema` crate 中。

```rust
// schema/src/network.rs
pub trait NetworkProvider: Send + Sync {
    fn get_mac_from_arp(&self, ip: Ipv4Addr) -> Option<String>;
}

// adapter/src/network.rs
impl NetworkProvider for DefaultNetworkProvider { ... }
```

**理由**：service 不能依赖 adapter（这是重构核心目标），而 schema 已有 `std::net::Ipv4Addr` 依赖，加一个 trait 定义没有任何额外成本。

### ✅ 决策 2. `PlatformError` 在 adapter 中的存留

**决策**：保留 `PlatformError`，保留 `Config` 变体。

`autostart.rs`（5 处）和 `dialog.rs`（18 处）等模块仍使用 `PlatformError::Config` 来表示"设置操作失败"（非 config 模块概念，而是 autostart/shell 配置失败时的错误描述）。Config 是个通用的命名，在这里语义为"配置操作失败"而非"配置模块"。无需改名，注释说明即可。

### ✅ 决策 3. `elevation.rs` 中 FLTK dialog 解耦

**决策**：在 Step 2 中解决。这是 adapter 包含 UI 逻辑的最明显违规。

### ✅ 决策 4. `defaults.rs` 的定位

**结论**：纯 import 替换，`adapter::config::load_config` → `rabbit_config::load_config`。无模糊点。

### ✅ 决策 5. `Lifecycle` 归属

**决策**：留在 app，不移动。

`Lifecycle` 包含平台信号处理（`pthread_sigmask`、`libc::signal`、`Fl_wait_for` FFI），这不是纯生命周期管理。但将其拆开会引入不必要的复杂度。TrayProvider 通过回调使用 Lifecycle，不需要直接引用。

### ✅ 决策 6. `tray_helper.rs` 的 FLTK 耦合

**决策**：通过 `TrayConfig.on_show/on_hide` 回调注入。IPC 协议逻辑移入 adapter，FLTK 窗口操作留在 app 层（通过回调注入）。

### ✅ 决策 7. 升级模块平台代码

**决策**：暂不移入 adapter。现有 `#[cfg]` 隔离已足够，与 app 的升级 UI 耦合较深，作为后续优化项。

### ✅ 决策 8. Diag 提取优先级

**决策**：提升到批次 0，与 Step 1 合并。避免 50+ 文件被改两遍。

### ⚠️ 注意：view_model.rs 的 `adapter::Result`

`view_model.rs:3` 使用了 `use adapter::Result`（即 `Result<T, PlatformError>`）。批次 0 中 `save_config` 将返回 `rabbit_config::Result<T>`（即 `Result<T, ConfigError>`），所以 `view_model.rs` 的 `save()` 和 `update_and_save()` 方法签名需要改为返回 `rabbit_config::Result<T>`。这是在文件变更表中容易被遗漏的。

---

## 8. 文件变更汇总

### 新建文件

| 文件 | 批次 | 说明 |
|------|------|------|
| `crates/rabbit-config/Cargo.toml` | 0 | 配置管理 crate |
| `crates/rabbit-config/src/lib.rs` | 0 | 从 `adapter/config.rs` 迁移 |
| `crates/rabbit-diag/Cargo.toml` | 0 | 诊断日志 crate |
| `crates/rabbit-diag/src/lib.rs` | 0 | 从 `adapter/diag.rs` 迁移 |
| `schema/src/network.rs` | 2 | `NetworkProvider` trait 定义 |
| `adapter/src/platform.rs` | 2 | `PlatformService` 统一入口 |
| `adapter/src/tray/mod.rs` | 4 | 托盘模块入口 |
| `adapter/src/tray/linux.rs` | 4 | 从 `app/systray_linux.rs` 移植 |
| `adapter/src/tray/non_linux.rs` | 4 | 从 `app/systray_non_linux.rs` 移植 |
| `adapter/src/x11_diag.rs` | 4 | 从 `app/app.rs` 移植 X11 handler |

### 删除文件

| 文件 | 批次 | 说明 |
|------|------|------|
| `adapter/src/config.rs` | 0 | 移到 `rabbit-config` |
| `adapter/src/diag.rs` | 0 | 移到 `rabbit-diag` |

### 修改文件（按批次）

**批次 0**（约 **35 个文件**）：
- 新建 `crates/rabbit-config/` + `crates/rabbit-diag/`
- `app/Cargo.toml` — 添加 `rabbit-config`、`rabbit-diag` 依赖
- `service/Cargo.toml` — 添加 `rabbit-config` 依赖
- `adapter/Cargo.toml` — 移除 `toml`、`dirs`
- `app/src/app.rs` — config import 替换 + `adapter::diag::log` → `rabbit_diag::log`（约 40 处）
- `app/src/main.rs` — `adapter::diag::log` → `rabbit_diag::log`（约 20 处）
- `app/src/lifecycle.rs` — `adapter::diag::log` → `rabbit_diag::log`（3 处）
- `app/src/systray_linux.rs` — `adapter::diag::log` → `rabbit_diag::log`
- `app/src/tray_helper.rs` — `adapter::diag::log` → `rabbit_diag::log`
- `app/src/view_model.rs` — 替换 config import + **`use adapter::Result` → `use rabbit_config::Result`**
- `app/src/ui_state.rs` — 11 处 config import 替换（含 `update_config` 调用）
- `app/src/ui/defaults.rs` — `adapter::config::load_config` → `rabbit_config::load_config`
- `app/src/ui/ping_tab.rs` — 同上
- `app/src/ui/http_tab.rs` — config import 替换
- `app/src/ui/chat_tab.rs` — config import 替换
- `app/tests/integration_ui_refresh.rs` — config import 替换
- `service/src/http.rs`、`tftpd.rs`、`tftpc.rs`、`chat.rs`、`scan.rs`、`ping.rs` — config import 替换
- `service/src/lib.rs` — 移除 `#[from] adapter::PlatformError` + `ServiceError::Platform` 变体
- `adapter/src/lib.rs` — 移除 `pub mod config;` `pub use config::*;` `pub mod diag;` `config_dir()` `data_dir()`
- `adapter/src/elevation.rs` — `crate::diag::log` → `rabbit_diag::log`（8 处）

**Step 2**（约 **7 个文件**）：
- `schema/src/lib.rs` — 添加 `pub mod network;` + re-export NetworkProvider
- `schema/src/network.rs` — 新建，定义 `NetworkProvider` trait
- `adapter/src/lib.rs` — 更新导出，添加 `pub mod platform;` `pub mod tray;`
- `adapter/src/platform.rs` — 新建，`PlatformService` 统一入口（约 200 行）
- `adapter/src/elevation.rs` — 移除 `show_elevation_error()`，`ensure_elevated()` 返回 `Result<()>`
- `adapter/src/network.rs` — 实现 `NetworkProvider for DefaultNetworkProvider`，删除 `calculate_ip_range()`
- `adapter/src/window.rs` — 函数内部存储 hwnd，不再暴露给调用方

**Step 3**（约 **4 个文件**）：
- `service/Cargo.toml` — 移除 `adapter` 依赖（已提前在批次 0 中准备）
- `service/src/lib.rs` — 确认 `ServiceError::Platform` 已移除
- `service/src/scan.rs` — 添加 `NetworkProvider` 注入
- `app/src/app.rs` — `ScanService` 构造时传入 `NetworkProvider`

**Step 4**（约 **12 个文件**）：
- `app/src/app.rs` — 删除 X11 handler 模块，替换 F1/F2/F3 为 adapter 调用，简化 systray 启动为 adapter 调用
- `app/src/main.rs` — 调整 tray helper 启动
- `app/src/lib.rs` — 移除 `pub mod systray;` `pub mod tray_helper;` 重新导出
- `app/src/systray.rs` — **删除**
- `app/src/systray_linux.rs` — **删除**
- `app/src/systray_non_linux.rs` — **删除**
- `app/src/tray_helper.rs` — **删除**
- `adapter/Cargo.toml` — 添加 `ksni`、`tray-icon` 条件依赖
- `adapter/src/lib.rs` — 添加 `pub mod tray;` `pub mod tray_helper;` `pub mod x11_diag;`
- `adapter/src/tray/mod.rs` — 新建，平台分派
- `adapter/src/tray/linux.rs` — 新建，从 `app/systray_linux.rs` 移植
- `adapter/src/tray/non_linux.rs` — 新建，从 `app/systray_non_linux.rs` 移植
- `adapter/src/tray_helper.rs` — 新建，从 `app/tray_helper.rs` 移植（IPC 协议）
- `adapter/src/x11_diag.rs` — 新建，从 `app/app.rs` 移植 X11 handler

---

## 9. 依赖图（各 Step 的先后关系）

```
批次 0 (config + diag 提取，合并)
  │
  └──→ Step 2 (adapter 接口净化 + NetworkProvider 定义)
          │
          ├──→ Step 3 (service 解耦)     ── 并行执行 ──→ Step 4 (平台代码移入)
          │                                                ↑
          └──── Step 4 依赖 Step 2 (PlatformService 定义)，不依赖 Step 3
```

**并行可能性**：
- Step 3 和 Step 4 无依赖关系，可并行
- Step 3 依赖批次 0（config/import 替换完成）+ Step 2（NetworkProvider 定义）
- Step 4 依赖 Step 2（PlatformService/tray/tray_helper 骨架）

### 建议的执行批次

```
批次 0: config + diag 提取（合并，一次 import 替换）
        预计编译检查一次通过
批次 1: Step 2 (adapter 接口净化) + schema::network::NetworkProvider 定义
        纯 adapter + schema 操作，不影响 app/service
批次 2: Step 3 (service 解耦) + Step 4 (平台代码移入)
        无依赖关系，可完全并行
```

---

## 10. 验收标准

### 批次 0 验收（config + diag 提取）
- [ ] `cargo check` 通过（全部 workspace）
- [ ] `service` 不再导入 `adapter::config::*`
- [ ] 所有 `adapter::config::*` 调用已替换为 `rabbit_config::*`
- [ ] 所有 `adapter::diag::log` 或 `crate::diag::log` 已替换为 `rabbit_diag::log`
- [ ] 所有 `libc::getpid()` 已替换为 `std::process::id()`
- [ ] `adapter/src/config.rs` 已删除
- [ ] `adapter/src/diag.rs` 已删除
- [ ] 配置文件的路径和行为不变（仍在 `~/.config/rabbit/rabbit.toml`）
- [ ] `view_model.rs` 的 `save()` / `update_and_save()` 返回类型改为 `rabbit_config::Result<T>`

### Step 2 验收
- [ ] `schema::network::NetworkProvider` trait 定义清晰可被 service 使用
- [ ] `PlatformService` 对外提供统一入口（不再暴露零散函数）
- [ ] `set_window_on_top` 不暴露 `hwnd`
- [ ] `elevation` 不直接创建 FLTK dialog（`show_elevation_error()` 已删除）
- [ ] `calculate_ip_range` 不再在 adapter 中存在

### Step 3 验收
- [ ] `service/Cargo.toml` 无 `adapter` 行
- [ ] `ScanService` 通过构造函数注入获取 `NetworkProvider`
- [ ] `ServiceError` 无 `Platform` 变体
- [ ] `grep -r "adapter" crates/service/` 无输出

### Step 4 验收
- [ ] `app/src/` 中无 `systray*.rs`
- [ ] `app/src/` 中无 `tray_helper.rs`
- [ ] X11 handler 在 adapter 中（`adapter/src/x11_diag.rs`）
- [ ] F1/F2/F3 通过 `adapter::dialog::open_url()` 调用
- [ ] `app/src/` 中 `#[cfg(target_os)]` 数量显著减少（当前 53 处，目标 < 10 处）
- [ ] 托盘功能正常（创建、显示/隐藏、退出、Quit 菜单触发 shutdown）
- [ ] `cargo build --release` 编译通过

---

## 11. 附录：全部变更清单

> 本重构不保留任何向后兼容。以下变更全部直接替换为新方式，无过渡层。

| 变更 | 说明 |
|------|------|
| `adapter::config::*` 全部移到 `rabbit_config::*` | 纯 import 替换，语义不变 |
| `adapter::diag::log` 全部移到 `rabbit_diag::log` | 纯 import 替换，`libc::getpid()` 替换为 `std::process::id()` |
| `adapter::PlatformError::Config` 变体保留（autostart 等仍使用） | 不涉及删除 |
| `service::ServiceError::Platform(adapter::PlatformError)` 移除 | 改为 `ServiceError::Other(String)` |
| `adapter::network::calculate_ip_range()` 删除 | service 中私有版本保留 |
| `elevation::ensure_elevated()` 不再创建 FLTK dialog | 返回错误由 app 层展示 |
| `window::set_window_on_top(hwnd, bool)` 改变 | 改为 `PlatformService::set_window_on_top(bool)` |
| `app::systray::*` 移到 `adapter::tray::*` | 接口改为 TrayProvider trait + TrayConfig |
| `app::tray_helper::*` 移到 `adapter::tray_helper::*` | FLTK 回调通过 TrayConfig.on_show/on_hide 注入 |
| `app::app.rs` 中 X11 handler 移到 `adapter::x11_diag::*` | 代码迁移 |
| `app::app.rs` 中 F1/F2/F3 `std::process::Command` 调用替换 | 改为 `PlatformService::open_url()` |
