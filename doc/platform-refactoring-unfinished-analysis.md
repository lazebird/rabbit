# 平台适配层重构 — 未完成项分析及方案参考

**基准文档**：`doc/platform-refactoring-execution-plan.md`（v3）
**分析日期**：2026-05-11
**状态**：方案参考，待评审决策后实施

---

## 1. 已完成项回顾

| 步骤 | 状态 | 关键交付 |
|------|------|---------|
| 批次 0 (config + diag 提取) | ✅ | `rabbit-config`、`rabbit-diag` 独立 crate，import 替换完成 |
| Step 2 (接口净化) | ⚠️ 部分 | `NetworkProvider` trait 已定义，`PlatformService`/`tray/` 未创建 |
| Step 3 (service 解耦) | ✅ | `service/Cargo.toml` 彻底移除 `adapter` 依赖 |
| Step 4 (平台代码移入) | ⏳ 5 项未完成 | 详见下文 |

---

## 2. 未完成项全景

### 2.1 依赖关系图

```
┌──────────────────────────────────────────────────┐
│               adapter crate 边界                   │
│                                                    │
│  目前 adapter 内的功能：                             │
│  ┌─────────┐ ┌──────────┐ ┌──────────────┐        │
│  │ window  │ │ dialog   │ │ taskbar      │        │
│  │ (泄漏hwnd)│ │ (自由函数) │ │ (自由函数)    │        │
│  ├─────────┤ ├──────────┤ ├──────────────┤        │
│  │ elevation│ │network   │ │ notification │        │
│  │ (含FLTK) │ │(已完成)   │ │ (自由函数)    │        │
│  ├─────────┤ ├──────────┤ ├──────────────┤        │
│  │ x11_diag│ │autostart │ │ shell/ping   │        │
│  │ (已完成)  │ │          │ │              │        │
│  └─────────┘ └──────────┘ └──────────────┘        │
│                                                    │
│  尚在 app 层、待移入 adapter 的：                    │
│  ┌──────────────────┐  ┌──────────────────────┐    │
│  │ systray*.rs      │  │ tray_helper.rs       │    │
│  │ (ksni/tray-icon) │  │ (Unix socket IPC)    │    │
│  │ 依赖: Lifecycle  │  │ 依赖: Lifecycle+FLTK │    │
│  │ 依赖: icon       │  │ 依赖: icon+libc     │    │
│  └──────────────────┘  └──────────────────────┘    │
│                                                    │
│  缺失的整合层：                                      │
│  ┌──────────────────────────────────────────┐      │
│  │ PlatformService 统一入口 (adapter/src/    │      │
│  │ platform.rs) — 不存在                     │      │
│  └──────────────────────────────────────────┘      │
└──────────────────────────────────────────────────┘
```

### 2.2 5 个未完成项

| # | 项目 | 当前所在 | 目标 | 行数 | 复杂度 |
|---|------|---------|------|------|--------|
| 1 | systray 迁移 | `app/systray*.rs` (3 文件) | `adapter/tray/` | ~430 | 中 |
| 2 | tray_helper 迁移 | `app/tray_helper.rs` | `adapter/tray_helper.rs` | 607 | 高 |
| 3 | PlatformService | 不存在 | `adapter/src/platform.rs` | — | 中 |
| 4 | elevation FLTK 解耦 | `adapter/elevation.rs` | 分离 UI 代码 | 30 | 低 |
| 5 | window hwnd 私有化 | `adapter/window.rs` | 消除 hwnd 参数 | 15 | 低 |

---

## 3. 项目 1：systray 迁移

### 3.1 当前状态

`crates/app/src/` 下 3 个文件：

| 文件 | 行数 | 技术栈 | 关键依赖 |
|------|------|--------|---------|
| `systray.rs` | 21 | 平台分派 (`#[cfg]`) | — |
| `systray_linux.rs` | 222 | `ksni` (D-Bus StatusNotifierItem) | `Lifecycle`, `icon`, `tray_helper` |
| `systray_non_linux.rs` | 186 | `tray-icon` (Windows/macOS) | `Lifecycle`, `icon` |

### 3.2 耦合点与解耦方案

| 耦合 | 源文件 | 当前代码 | 解耦方案 |
|------|--------|---------|---------|
| **Lifecycle** | `systray_linux.rs:26,31` | `LIFECYCLE: Mutex<Option<Lifecycle>>` | 通过 `TrayConfig.on_shutdown: Box<dyn Fn() + Send + Sync>` 回调注入 |
| **icon** | `systray_linux.rs:103` | `crate::icon::load_app_icon()` | 将 `icon.rs` 移到 `adapter/src/icon.rs`（纯数据解码，零平台依赖） |
| **tray_helper** | `systray_linux.rs:136-143` | `crate::tray_helper::is_connected()` | 通过 `TrayConfig.is_helper_connected: Option<Box<dyn Fn() -> bool>>` 注入 |
| **MAIN_WIN** | `systray_linux.rs:25` | `MAIN_WIN: Mutex<Option<Window>>` | 改为 `TrayConfig.on_show` / `on_hide` 回调 |

### 3.3 推荐方案：完整迁移到 adapter

#### 步骤 1：将 `icon.rs` 移到 adapter

`app/src/icon.rs` → `adapter/src/icon.rs`。该模块仅依赖 `image` crate（已存在于 workspace），与平台无关。

```rust
// adapter/src/icon.rs — 纯数据解码，无平台依赖
pub struct IconData {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn load_app_icon() -> IconData {
    let bytes = include_bytes!("../resources/icon.ico");
    // 注意：路径相对于 adapter/Cargo.toml
    // 需要从 app/resources/ 复制 icon.ico 到 adapter/resources/
    // 或 adapter 直接引用 app 的路径（不推荐）
}
```

**路径问题**：`app/resources/icon.ico` → 移到 `adapter/resources/icon.ico`。或者将 icon 嵌入到 `schema` crate 中（更干净，但 schema 需要加 `image` 依赖）。

#### 步骤 2：新建 adapter 中的 tray 模块

```rust
// adapter/src/tray/mod.rs
pub struct TrayConfig {
    pub on_shutdown: Box<dyn Fn() + Send + Sync>,
    pub on_show: Box<dyn Fn() + Send + Sync>,
    pub on_hide: Box<dyn Fn() + Send + Sync>,
    /// When Some, the tray delegates to the tray-helper IPC
    /// (Linux elevation scenario).  Returns `true` if helper is active.
    pub is_helper_connected: Option<Box<dyn Fn() -> bool + Send + Sync>>,
}

pub trait TrayProvider: Send {
    fn init(&mut self, config: TrayConfig) -> Result<(), String>;
    fn remove(&mut self);
    fn is_active(&self) -> bool;
    fn update(&mut self, enabled: bool);
}

// 平台分派
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;
#[cfg(target_os = "linux")]
pub use self::imp::*;

#[cfg(not(target_os = "linux"))]
#[path = "non_linux.rs"]
mod imp;
#[cfg(not(target_os = "linux"))]
pub use self::imp::*;
```

#### 步骤 3：适配 app 层调用

```rust
// app/src/app.rs — 改造后
use adapter::tray::{TrayConfig, TrayProvider};

let tray_config = TrayConfig {
    on_shutdown: { let lc = self.lifecycle.clone(); Box::new(move || lc.request_shutdown()) },
    on_show: { let win = main_win.clone(); Box::new(move || win.show()) },
    on_hide: { let win = main_win.clone(); Box::new(move || win.hide()) },
    is_helper_connected: Some(Box::new(|| crate::tray_helper::is_connected())),
};

if systray_requested {
    // 委托给 adapter，不再直接操作 systray 模块
    adapter::tray::init_systray(tray_config).await?;
}
```

### 3.4 替代方案：暂缓

维持当前架构，仅做最小清理。等 `tray_helper` 也准备好后再统一迁移。

### 3.5 依赖变更

```toml
# adapter/Cargo.toml — 新增条件依赖
[target.'cfg(target_os = "linux")'.dependencies]
ksni = { version = "0.3" }

[target.'cfg(not(target_os = "linux"))'.dependencies]
tray-icon = "0.23"

# 通用
image = "0.25"
```

---

## 4. 项目 2：tray_helper 迁移

### 4.1 当前状态

`app/tray_helper.rs`（607 行），Linux 专用。功能：

| 角色 | 代码 | 说明 |
|------|------|------|
| **Parent 侧** | `spawn()`、`connect()`、`hide_tray()`、`show_tray()`、`shutdown_helper()` | 提权后的主进程与 helper 通信 |
| **Helper 侧** | `run()` | `--tray-helper` 子进程入口，拥有 D-Bus tray |
| **IPC 协议** | `TAG_*` 常量、Unix socket 管理、消息格式 | 固定 4 字节 LE u32 tag |

### 4.2 耦合点

| 耦合 | 代码位置 | 说明 |
|------|---------|------|
| **Lifecycle** | `tray_helper.rs:243,378` | `connect()` 和 `start_event_listener()` 中用于 Quit |
| **FLTK** | `tray_helper.rs:371,374,380` | `fltk::app::awake_callback()` 在事件监听线程中唤醒 UI 线程 |
| **icon** | `tray_helper.rs:62` | `include_bytes!("../resources/icon.ico")` 路径相对于 app |
| **ksni** | `tray_helper.rs:406-443` | Helper 侧创建 ksni tray，需要 tokio runtime |
| **main.rs 入口** | `main.rs:27` | `app::tray_helper::run()` 作为 `--tray-helper` 的入口 |

### 4.3 推荐方案：分两步走

#### 阶段 A：IPC 协议提取（短期，低风险）

将纯粹的 IPC 协议部分移入 `adapter`，不涉及 Lifecycle 和 FLTK：

```
提取到 adapter/src/tray_ipc.rs:
  - TAG_* 常量
  - socket_path_for(), helper_socket_path(), parent_socket_path()
  - send_command()
  - try_connect_inner()
  
留在 app/src/tray_helper.rs:
  - run()  — helper 进程入口（含 ksni + FLTK 交互）
  - connect() — parent 侧连接（含 FLTK awake_callback）
  - spawn() — 启动子进程
  - show_tray() / hide_tray() / shutdown_helper()
```

```rust
// adapter/src/tray_ipc.rs — 纯 IPC 协议
pub const TAG_ACTIVATE: u32 = 0x01;
pub const TAG_MENU_SHOW: u32 = 0x02;
pub const TAG_MENU_HIDE: u32 = 0x03;
pub const TAG_MENU_QUIT: u32 = 0x04;
pub const TAG_TRAY_HIDE: u32 = 0x10;
pub const TAG_TRAY_SHOW: u32 = 0x11;

pub fn socket_path_for(uid: u32) -> String { format!("/tmp/rabbit-tray-{uid}.sock") }

pub fn send_command(tag: u32) { /* 写 4 字节到 HELPER_STREAM */ }
pub fn try_connect_inner() -> Option<UnixStream> { /* ... */ }
```

这可以立即将 607 行中的约 80 行协议代码移入 adapter，减少 app 层平台代码。

#### 阶段 B：完整迁移（长期，需先完成 PlatformService）

条件：
1. `icon.rs` 已移到 adapter（项目 1 的前提）
2. `PlatformService` 已提供 `on_shutdown` 回调机制（项目 3）
3. FLTK 回调通过 `TrayConfig` 注入

```rust
// adapter/src/tray_helper.rs — 完整迁移后
use crate::tray::TrayConfig;  // 注入回调

pub fn run(config: TrayConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 原来对 crate::icon, crate::lifecycle 的引用
    // 改为 config.on_shutdown(), adapter::icon::load_app_icon()
}
```

### 4.4 主要障碍

`run()` 函数内部在 `forward_events` 前是线性逻辑（accept → 启动命令线程 → 等待事件），难以拆分为纯 IPC 部分和非 IPC 部分。**建议阶段 B 暂缓**，先完成其他 4 个项目。

---

## 5. 项目 3：PlatformService 统一入口

### 5.1 当前状态

不存在。所有 adapter 功能通过自由函数暴露：

```rust
// app.rs 中涉及 adapter 的调用分布（不完全统计）
adapter::window::set_window_on_top(hwnd, true);        // 第 561 行
adapter::window::set_window_on_top(hwnd, top_value);   // 第 1147 行
adapter::dialog::open_url(url);                        // 第 430, 436 行
adapter::dialog::open_file_manager(path);              // 第 445 行
adapter::taskbar::set_main_window_hwnd(hwnd);           // 第 557 行
adapter::autostart::set_autostart(req);                 // 第 475, 1115 行
adapter::shell::set_shell_integration(req, &path);      // 第 482 行
adapter::taskbar::TaskbarProgress::update(...);         // scan/ping 服务中调用
adapter::notification::show_notification(...);          // 服务中调用
adapter::ping::check_ping_permission();                 // 服务中调用
```

### 5.2 方案 A：全局单例 PlatformService（推荐）

```rust
// adapter/src/platform.rs
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// PlatformService 提供所有平台相关操作的统一入口。
///
/// 使用全局单例模式，app 层在窗口创建后调用一次 `set_main_window()`
/// 注册 hwnd，后续所有操作无需传递平台句柄。
pub struct PlatformService {
    main_hwnd: AtomicUsize,
}

impl PlatformService {
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<PlatformService> = OnceLock::new();
        INSTANCE.get_or_init(|| PlatformService {
            main_hwnd: AtomicUsize::new(0),
        })
    }

    /// 注册主窗口句柄（窗口创建后调用一次）
    pub fn set_main_window(hwnd: usize) {
        Self::global().main_hwnd.store(hwnd, Ordering::Relaxed);
    }

    fn hwnd(&self) -> usize {
        self.main_hwnd.load(Ordering::Relaxed)
    }

    // ── 窗口管理 ──
    pub fn set_window_on_top(&self, on_top: bool) {
        let hwnd = self.hwnd();
        if hwnd != 0 {
            crate::window::set_window_on_top(hwnd, on_top);
        }
    }

    // ── URL/文件操作（委托到 dialog, 不依赖 hwnd）──
    pub fn open_url(&self, url: &str) -> crate::Result<()> {
        crate::dialog::open_url(url)
    }

    pub fn open_file_manager(&self, path: &str) -> crate::Result<()> {
        crate::dialog::open_file_manager(path)
    }

    // ── 系统设置 ──
    pub fn set_autostart(&self, enabled: bool) -> crate::Result<()> {
        crate::autostart::set_autostart(enabled)
    }

    // ── 通知 ──
    pub fn show_notification(&self, title: &str, message: &str) -> crate::Result<()> {
        crate::notification::show_notification(title, message)
    }

    // ── 提权 ──
    pub fn is_elevated(&self) -> bool {
        crate::elevation::is_elevated()
    }

    // ── 任务栏 ──
    pub fn update_taskbar(&self, progress: u32, total: u32, color: &str) {
        crate::taskbar::TaskbarProgress::update(progress, total, color);
    }
}
```

`app.rs` 改造后：

```rust
// 窗口创建后 — 一次注册
let hwnd = main_win.raw_handle() as usize;
adapter::PlatformService::set_main_window(hwnd);

// 后续调用 — 不再传递 hwnd
if top_requested {
    adapter::PlatformService::global().set_window_on_top(true);
}

// 简化调用：可以加一个 use 别名
use adapter::PlatformService as PS;
PS::global().open_url("https://...");
```

### 5.3 方案 B：保留自由函数（最小改动）

不在 adapter 侧做改动，继续使用自由函数。hwnd 泄漏问题单独在 window.rs 内部解决（见项目 5 方案 B）。

### 5.4 现有模块如何融入 PlatformService

| 现有模块 | 融入方式 | 备注 |
|---------|---------|------|
| `window.rs` | `PlatformService::set_window_on_top()` 委托到此 | 内部函数仍可保留 hwnd 参数 |
| `dialog.rs` | `PlatformService::open_url/dialog()` 委托到此 | dialog 函数本身不需要改 |
| `taskbar.rs` | `PlatformService::update_taskbar()` 委托到此 | TaskbarProgress 内部管理 hwnd |
| `notification.rs` | `PlatformService::show_notification()` 委托到此 | — |
| `ping.rs` | `PlatformService::check_ping_permission()` 委托到此 | — |
| `elevation.rs` | `PlatformService::is_elevated()` 委托到此 | — |
| `autostart.rs` | `PlatformService::set_autostart()` 委托到此 | — |
| `shell.rs` | `PlatformService::set_shell_integration()` 委托到此 | — |

---

## 6. 项目 4：elevation FLTK dialog 解耦

### 6.1 当前问题

`adapter/src/elevation.rs:218-247` 的 `show_elevation_error()` 函数创建 FLTK dialog：

```rust
fn show_elevation_error(msg: &str) {
    let app = fltk::app::App::default();
    let mut wind = fltk::window::Window::default()
        .with_size(400, 180).with_label("Elevation Failed");
    // ... 创建 TextDisplay + OK 按钮 ...
    app.run().unwrap();  // 阻塞式 FLTK 事件循环
}
```

该函数在 4 条失败路径被调用，且每个路径都 `exit(1)`：

| 调用点 | 条件 | 代码行 |
|--------|------|--------|
| 密码取消 | `xelevate::request_password()` 返回 `None` | 第 76 行 |
| spawn 失败 | `cmd.spawn()` 失败 | 第 136 行 |
| sudo 退出码非零 | `status.success()` 为 false | 第 178 行 |
| wait 失败 | `try_wait()` 返回错误 | 第 188 行 |

### 6.2 方案 A：分离到 elevation_ui 模块（推荐）

```rust
// adapter/src/elevation_ui.rs — 仅 UI 代码
// 条件编译：只在需要时编译
#[cfg(not(any(debug_assertions, target_os = "android", target_os = "ios")))]
pub fn show_error(msg: &str) -> ! {
    use fltk::prelude::*;
    
    let app = fltk::app::App::default();
    let mut wind = fltk::window::Window::default()
        .with_size(400, 180).with_label("Elevation Failed");
    wind.make_modal(true);
    // ... 现有 dialog 代码 ...
    app.run().unwrap();
    std::process::exit(1);
}

// adapter/src/elevation.rs — 改为调用 elevation_ui
// 移除此文件顶部的 use fltk::* 和整个 show_elevation_error 函数
fn elevate_with_sudo(exe_path: &str) -> ! {
    // ...
    let password = match xelevate::request_password() {
        Some(p) => p,
        None => crate::elevation_ui::show_error("Elevation cancelled by user."),
    };
    // ...
}
```

**优点**：
- 改动最小，不影响函数签名（仍返回 `!`）
- UI 代码与业务逻辑物理分离
- 保留失败退出的语义

**缺点**：
- adapter 仍然有 FLTK 依赖（只是换了个模块）
- 没有真正消除 FLTK 耦合

### 6.3 方案 B：返回错误 + app 层展示

```rust
// adapter/src/elevation.rs
#[derive(Debug)]
pub enum ElevationError {
    Cancelled,
    SudoFailed(String),
    SpawnFailed(String),
}

pub fn ensure_elevated() -> Result<(), ElevationError> {
    // 不再内部 exit，返回错误
}

// app/src/main.rs
if let Err(e) = adapter::ensure_elevated() {
    let msg = format!("Elevation failed: {e}");
    // app 层有 FLTK 上下文，可以直接显示 dialog
    fltk::dialog::alert_default(&msg);
    std::process::exit(1);
}
```

**问题**：`ensure_elevated()` 目前返回 `!`（never type），因为 `elevate_with_sudo()` 内部 `exit()`。改为返回 `Result` 需要改动 `main.rs` 和 `app.rs` 的控制流。

### 6.4 推荐方案 A

理由：阶段 B 的改动过大且收益有限（elevation 在程序初始化阶段执行，FLTK dialog 是唯一合理的交互方式）。方案 A 做到 UI/业务逻辑分离即可。

---

## 7. 项目 5：window.rs hwnd 私有化

### 7.1 当前问题

```rust
// adapter/src/window.rs — hwnd 泄漏
pub fn set_window_on_top(hwnd: usize, on_top: bool) { ... }
pub fn hide_window(hwnd: usize) { ... }
pub fn show_window(hwnd: usize) { ... }
```

`app.rs` 中每次调用都需要获取 hwnd：

```rust
// app.rs 第 551 行 — 获取 hwnd
let hwnd = main_win.raw_handle() as usize;

// 第 557 行 — 传给 taskbar
adapter::taskbar::set_main_window_hwnd(hwnd);

// 第 561 行 — 传给 window
adapter::window::set_window_on_top(hwnd, true);

// 第 1147 行 — SettingsSave 中再次获取
let hwnd = win.raw_handle() as usize;
adapter::window::set_window_on_top(hwnd, top_value);
```

### 7.2 方案 A：PlatformService 统一管理（推荐，与项目 3 合并）

PlatformService 持有 hwnd，app 层只需在窗口创建后注册一次：

```rust
// app.rs 第 551-561 行改造后
let hwnd = main_win.raw_handle() as usize;
adapter::PlatformService::set_main_window(hwnd);  // ← 一次注册

// taskbar 也委托到 PlatformService，不需要分别调用 set_main_window_hwnd
if top_requested {
    adapter::PlatformService::global().set_window_on_top(true);  // ← 无 hwnd 参数
}

// 第 1147 行 SettingsSave
fltk::app::awake_callback(move || {
    adapter::PlatformService::global().set_window_on_top(top_value);  // ← 无 hwnd 参数
});
```

### 7.3 方案 B：window.rs 内部静态存储（中间步骤）

如果 PlatformService 暂缓，可在 `window.rs` 内部解决：

```rust
// adapter/src/window.rs — 方案 B
use std::sync::OnceLock;

static MAIN_HWND: OnceLock<usize> = OnceLock::new();

/// 注册主窗口句柄（窗口创建后调用一次）
pub fn set_main_window(hwnd: usize) {
    let _ = MAIN_HWND.set(hwnd);
}

/// 设置窗口置顶（无需传 hwnd）
pub fn set_window_on_top(on_top: bool) {
    let Some(&hwnd) = MAIN_HWND.get() else { return };
    // 现有实现，只是不再从参数取 hwnd
    set_window_on_top_impl(hwnd, on_top)
}

// 内部实现保留 hwnd 参数
fn set_window_on_top_impl(hwnd: usize, on_top: bool) {
    #[cfg(target_os = "windows")] { /* ... */ }
    #[cfg(target_os = "linux")]   { /* ... */ }
}
```

同样，`taskbar.rs` 也可以做类似处理：

```rust
// adapter/src/taskbar.rs
pub fn set_main_window_hwnd(hwnd: usize) {
    // 不再使用自己的 MAIN_HWND，改为与 window.rs 共享
    crate::window::set_main_window(hwnd);
}

pub fn update(progress: u32, total: u32, color: &str) {
    // 从 window.rs 获取 hwnd，不再自己存储
    // 但实际上最简单的方案是把 hwnd 存储提取到 adapter 级别的全局变量
}
```

### 7.4 推荐方案 A

理由：`set_main_window(hwnd)` 只有一个事实来源（FLTK window 创建后），所有消费者（window、taskbar、systray）都应该从同一个地方获取。PlatformService 就是最自然的位置。

---

## 8. 执行建议

### 8.1 建议的执行批次

```
阶段 1（无外部依赖，可完全并行）:
  ├── 1a. PlatformService 实现 ← 项目 3 + 项目 5
  ├── 1b. elevation_ui 分离   ← 项目 4
  └── 1c. icon.rs 移到 adapter ← 为阶段 2 铺路

阶段 2（依赖阶段 1 的输出）:
  ├── 2a. systray 迁移 ← 依赖 PlatformService(TrayConfig) + icon
  └── 2b. tray_ipc 协议提取 ← 项目 2 的阶段 A

阶段 3（高复杂度）:
  └── 3a. tray_helper 完整迁移 ← 项目 2 的阶段 B
       ├── 依赖 systray 迁移完成（可复用 TrayConfig）
       └── 需要验证 --tray-helper 入口在 adapter 中能正常工作
```

### 8.2 依赖关系

```
阶段 1a (PlatformService) ──────────────┐
阶段 1b (elevation_ui)    ─── 独立      ├── 可并行
阶段 1c (icon 搬家)       ──────────────┘
       │
       ▼
阶段 2a (systray 迁移) ←── 依赖 1a + 1c
       │
       ▼
阶段 3a (tray_helper 完整) ←── 依赖 2a
```

### 8.3 工作量估算

| 项目 | 文件变更数 | 估算工时 | 风险 |
|------|-----------|---------|------|
| 1a. PlatformService | ~10 文件（新建 1 + 修改 app.rs 等） | 中 | 低（接口封装，无逻辑变更） |
| 1b. elevation_ui | ~3 文件（新建 1 + 修改 2） | 小 | 低（纯搬运） |
| 1c. icon 搬家 | ~5 文件（新建 1 + 修改 4） | 小 | 中（路径变更需要同步所有引用） |
| 2a. systray 迁移 | ~8 文件（新建 4 + 删除 3 + 修改 1） | 中 | 中（需要验证 Linux + 非 Linux） |
| 2b. tray_ipc 提取 | ~3 文件（新建 1 + 修改 1） | 小 | 低（纯提取，无行为变更） |
| 3a. tray_helper 完整 | ~5 文件（新建 1 + 删除 1 + 修改 3） | 大 | 高（IPC + ksni + FLTK 混合逻辑） |

### 8.4 验收标准

#### 阶段 1 验收
- [ ] `PlatformService::global()` 可被所有调用方使用
- [ ] `app.rs` 中 `set_window_on_top()` 调用不再传递 hwnd
- [ ] `app.rs` 中 `open_url()`、`open_file_manager()` 通过 PlatformService 调用
- [ ] `elevation.rs` 不再直接包含 FLTK Dialog 创建代码
- [ ] `adapter/src/icon.rs` 存在且 `app/src/icon.rs` 仍可正常工作（或移除了引用）
- [ ] `cargo check` 通过

#### 阶段 2 验收
- [ ] `adapter/src/tray/` 包含 linux.rs 和 non_linux.rs
- [ ] `app/src/systray*.rs` 已删除
- [ ] `app/src/` 中 `#[cfg(target_os)]` 数量减少（目标 < 30 处，当前约 53 处）
- [ ] 托盘功能正常（创建、显示/隐藏、退出、Quit 触发 shutdown）
- [ ] `cargo build --release` 编译通过

---

## 9. 决策待办

以下决策需要在实施前确认：

| # | 决策 | 选项 | 推荐 |
|---|------|------|------|
| D1 | `icon.rs` 放 adapter 还是 schema | A: adapter; B: schema | A（icon 是平台资源，不宜放 schema） |
| D2 | `TrayConfig.on_shutdown` 回调签名 | A: `Box<dyn Fn()>`; B: `Box<dyn Fn() + Send + Sync>` | B（确保线程安全） |
| D3 | PlatformService 单例还是实例传递 | A: 全局单例; B: app 持有实例 | A（减少模板代码，适配现有自由函数风格） |
| D4 | elevation_ui 放在 adapter 还是 app | A: adapter/elevation_ui.rs; B: app 层显示 | A（减少 app 层变更，保持错误处理局部性） |
| D5 | tray_helper 阶段 B 的优先级 | A: 立即做; B: 等阶段 2 完成后评估 | B（复杂度高，先积累经验） |
