# 代码结构优化计划：消除 `target_os` 条件编译的层级越界

> 本文档分析 `crates/app/`（表现层）中 `#[cfg(target_os = "...")]` 条件编译的违规使用，并制定向 `crates/adapter/`（基础设施层）迁移的系统化计划。

---

## 1. 问题概述

### 1.1 架构原则

项目采用四层架构：

```
┌─────────────────────────────────────────┐
│  表现层 (crates/app)                    │
│  FLTK UI + ViewModel                    │
├─────────────────────────────────────────┤
│  业务层 (crates/service)                │
│  核心业务逻辑、协议处理                    │
├─────────────────────────────────────────┤
│  数据层 (crates/schema, rabbit-config)   │
│  数据模型、配置持久化                      │
├─────────────────────────────────────────┤
│  基础设施层 (crates/adapter)             │
│  平台适配（OS 差异抽象）                   │
└─────────────────────────────────────────┘
```

**核心约束**：`#[cfg(target_os = "...")]` 只能出现在基础设施层（`adapter`），表现层和业务层不得感知平台差异。

### 1.2 当前现状

| 层 | `target_os` 出现次数 | 是否符合架构 |
|---|---|---|
| `crates/adapter/` | 66 处 / 13 个文件 | ✅ 正确 — 这是 adapter 的职责 |
| `crates/service/` | 0 处 | ✅ 正确 |
| `crates/schema/` | 0 处 | ✅ 正确 |
| **`crates/app/`** | **0 处 / 0 个文件** | ✅ **已消除（原 19 处/6 文件）** |

### 1.3 违规总览

| 编号 | 文件 | 违规内容 | 严重程度 | 阶段 | 状态 |
|---|---|---|---|---|---|
| V1 | `app/src/ui/settings_tab.rs` | `open_url()` — 重复实现 adapter 已有函数 | 🔴 高 | P0a | ✅ 已消除 |
| V2 | `app/src/ui/settings_tab.rs` | `open_folder()` — 重复实现 adapter 已有函数 | 🔴 高 | P0a | ✅ 已消除 |
| V3 | `app/src/app.rs` | `use adapter::x11_diag` 条件导入 | 🟡 中 | P1 | ✅ 已消除 |
| V4 | `app/src/app.rs` | `x11_diag::install()` 条件调用 | 🟡 中 | P1 | ✅ 已消除 |
| V5 | `app/src/app.rs` | tray helper + systray 初始化按平台分支 | 🔴 高 | P1 | ✅ 已消除 |
| V6 | `app/src/app.rs` | systray fallback 按平台分支 | 🔴 高 | P1 | ✅ 已消除 |
| V7 | `app/src/app.rs` | 清理逻辑按平台分支 | 🔴 高 | P1 | ✅ 已消除 |
| V8 | `app/src/app.rs` | `cfg!(target_os = "windows")` 运行时判断 | 🟡 中 | P1 | ✅ 已消除 |
| V9 | `app/src/app.rs` | systray 更新逻辑按平台分支 | 🔴 高 | P1 | ✅ 已消除 |
| V10 | `app/src/main.rs` | tray helper 模式入口条件编译 | 🟡 中 | P2 | ✅ 已消除 |
| V11 | `app/src/main.rs` | tray helper spawn 条件编译 | 🟡 中 | P2 | ✅ 已消除 |
| V12 | `app/src/upgrade/models.rs` | `current_platform()` 平台检测 | 🟡 中 | P0b | ✅ 已消除 |
| V13 | `app/src/upgrade/installer.rs` | 整个安装逻辑按平台分支 | 🔴 高 | P0b | ✅ 已消除 |
| V14 | `app/build.rs` | Windows 资源嵌入 | 🟢 低 | — | ⏸️ 保留（构建脚本，可接受） |

---

## 2. 逐项分析与迁移方案

### 2.1 重复实现（V1, V2）— P0a

**现状**：`settings_tab.rs` 自实现了 `open_url()` 和 `open_folder()`，但 `adapter::dialog` 中已有完全相同的函数。

| `settings_tab` 中的重复实现 | `adapter` 中已有函数 |
|---|---|
| `open_url(url)` → `cmd /c start` / `open` / `xdg-open` | `adapter::dialog::open_url(url)` — 完全等价 |
| `open_folder(path)` → `explorer` / `open` / `xdg-open` | `adapter::dialog::open_file_manager(path)` — 逻辑等价（名称不同） |

**注意**：`adapter::dialog::open_url()` 在非主流平台返回 `Err(PlatformError::NotSupported)`，而当前 `settings_tab` 的实现对所有平台都丢弃错误（`let _ = ...`）。替换后行为一致。`app.rs` 中已有两处正确使用 `adapter::dialog::open_url()`（行 402, 408），可参考。

**操作**：
1. 删除 `settings_tab.rs` 中的 `open_url()` 函数
2. 调用点 `open_url(HOME_URL)` → `adapter::dialog::open_url(HOME_URL)`
3. 调用点 `open_url(HELP_URL)` → `adapter::dialog::open_url(HELP_URL)`
4. 删除 `settings_tab.rs` 中的 `open_folder()` 函数
5. 调用点 `open_folder(&config_path)` → `adapter::dialog::open_file_manager(&config_path)`
6. 保留 `get_config_folder()`（纯逻辑，无平台依赖）

**验证**：`cargo build` 通过即可。

---

### 2.2 X11 诊断初始化（V3, V4）— P1

**现状**：`app.rs` 中条件导入 `use adapter::x11_diag` 并调用 `x11_diag::install()`。

**方案**：在 `adapter` 中新增统一切入函数：

```rust
// adapter/src/lib.rs 新增
/// 安装平台相关的错误/诊断处理器（当前仅 Linux X11）
pub fn install_platform_handlers() {
    #[cfg(target_os = "linux")]
    crate::x11_diag::install();
}
```

**操作**：
1. `adapter/src/lib.rs` 中新增 `install_platform_handlers()`
2. `app.rs` 中删除 `#[cfg(target_os = "linux")] use adapter::x11_diag`
3. `app.rs` 中 `x11_diag::install()` → `adapter::install_platform_handlers()`

**验证**：`cargo build` 通过。注意 `x11_diag` 在 adapter 层始终被编译，但 `install()` 函数内部用 `#[cfg]` 保护——这不影响正确性（空函数体在非 Linux 上被优化掉）。或者可以直接在 adapter lib.rs 中用 `#[cfg(target_os = "linux")]` 包裹模块声明。

---

### 2.3 Systray 统一抽象（V5, V6, V7, V9）— P1

**关键背景**：需要正确理解现有的两层抽象。

#### 2.3.1 adapter 层现状

**`adapter::tray`（已正确抽象）**：通过 `mod imp` 分派到 `linux.rs` / `non_linux.rs`，导出统一 API：
- `init_systray(lifecycle)` / `remove_systray()` / `update_systray(enabled)` / `is_active()` / `set_main_window()` / `set_lifecycle()`

这些已经在 `app.rs` 中正确使用（行 552, 1092, 1107, 1111）。

**`adapter::tray_helper`（纯 Linux 模块）**：解决提权后 root 进程无法访问用户 D-Bus 会话的问题。通过 Unix domain socket 与子进程通信：
- `spawn()` / `connect()` / `show_tray()` / `hide_tray()` / `shutdown_helper()` / `is_connected()` / `has_helper_socket()`

#### 2.3.2 app.rs 中的违规

**违规的本体不是 `adapter::tray` 调用**，而是控制流逻辑：

```rust
// app.rs:554-608 — 初始化阶段
#[cfg(target_os = "linux")] {
    // Linux: 先尝试连 helper，连不上则 fallback 到本地 ksni
    let has_sock = adapter::tray_helper::has_helper_socket();
    let helper_connected = if has_sock {
        adapter::tray_helper::connect(...)
    } else { false };
    // ... helper 连接后的分支逻辑
}
#[cfg(not(target_os = "linux"))] {
    // 非 Linux: 直接 init_systray
    if systray_requested { tray::init_systray(...).await; }
}

// app.rs:642-653 — 清理阶段
#[cfg(target_os = "linux")]
{ tray_helper::shutdown_helper(); tray::remove_systray(); }
#[cfg(not(target_os = "linux"))]
{ tray::remove_systray(); }

// app.rs:1094-1111 — 更新阶段
#[cfg(target_os = "linux")]
{ if helper 在线 { 通过 IPC 控制 } else { tray::update_systray().await } }
#[cfg(not(target_os = "linux"))]
{ tray::update_systray().await }
```

#### 2.3.3 统一抽象方案

在 `adapter` 层新增一个统一的 systray 生命周期管理入口，封装所有平台差异：

```rust
// adapter/src/systray.rs — 新增统一切入点

/// 初始化系统托盘：Linux 优先走 tray_helper，失败回退到本地 ksni；
/// 其他平台直接使用本地 systray。
pub async fn init(lifecycle: &Lifecycle, enabled: bool) -> InitResult {
    #[cfg(target_os = "linux")]
    {
        // 现有 tray_helper 逻辑...
        // 返回 InitResult { helper_connected: bool }
    }
    #[cfg(not(target_os = "linux"))]
    {
        if enabled { crate::tray::init_systray(lifecycle.clone()).await.ok(); }
        InitResult { helper_connected: false }
    }
}

/// 清理系统托盘
pub fn cleanup() {
    #[cfg(target_os = "linux")]
    crate::tray_helper::shutdown_helper();
    crate::tray::remove_systray();
}

/// 更新 systray 启用状态
pub async fn update(enabled: bool) {
    #[cfg(target_os = "linux")]
    if crate::tray_helper::is_connected() {
        if enabled { crate::tray_helper::show_tray(); }
        else { crate::tray_helper::hide_tray(); }
        return;
    }
    crate::tray::update_systray(enabled).await;
}
```

**方案优势**：
- `tray_helper` 的 IPC 细节完全封装在 adapter 内部
- `app.rs` 不再需要知道"helper"的概念——只问"开启/关闭/清理 systray"
- 非 Linux 平台完全不受影响

**操作**：
1. 新建 `adapter/src/systray.rs`
2. 实现 `init()`、`cleanup()`、`update()` 三个函数
3. 在 `adapter/src/lib.rs` 注册模块
4. `app.rs` 中删除所有 `#[cfg(target_os = "linux")]` / `#[cfg(not(target_os = "linux"))]` 分支
5. `app.rs` 替换为统一调用：`adapter::systray::init()` / `cleanup()` / `update()`

**验证**：`cargo build` 通过 + Linux 环境手动验证 systray 功能。

---

### 2.4 临时目录路径抽象（V8）— P1

**现状**：`app.rs:718` 使用 `cfg!()` 运行时判断，Windows 用 `std::env::temp_dir()`，Linux/macOS 硬编码 `/tmp/rabbit_update`。

**分析**：`std::env::temp_dir()` 在 Linux 上也返回 `/tmp`，所以可以统一使用:

```rust
// 当前
let temp_dir = if cfg!(target_os = "windows") {
    std::env::temp_dir().join("rabbit_update")
} else {
    std::path::PathBuf::from("/tmp/rabbit_update")
};

// 改为（等价行为）
let temp_dir = std::env::temp_dir().join("rabbit_update");
```

此变更纯语义等价，无需 adapter 介入。

**操作**：
1. 将条件判断替换为统一 `std::env::temp_dir().join("rabbit_update")`

**验证**：`cargo build` + 确认 Windows/Linux 下临时目录行为正确（Windows 返回 `%TEMP%\rabbit_update`，Linux 返回 `/tmp/rabbit_update`，与原来一致）。

---

### 2.5 入口逻辑中的平台分支（V10, V11）— P2

**现状**：`main.rs` 包含两类 Linux 专属逻辑：

1. **`--tray-helper` 模式检测**（行 24-28）：当参数包含此标志时，调用 `adapter::tray_helper::run()` 并直接 `return` 退出进程。此模式由已提权的父进程通过 `spawn()` 触发。

2. **tray helper spawn**（行 39-46）：在非 debug 且非提权状态下，预先启动 helper 子进程，使其能访问用户 D-Bus。

**控制流示意**：
```
main()
├── [--tray-helper] → tray_helper::run() → return Ok(())  // 作为子进程运行，直接退出
├── [非 debug]
│   ├── [Linux + 非提权] → tray_helper::spawn()          // 提权前启动 helper
│   ├── ensure_elevated()                                  // 提权（可能 exit/restart）
│   └── ... post-elevation code
└── [debug]
    └── ... 直接运行
```

**方案**：将入口的平台分支逻辑封装到 `adapter::tray_helper` 中：

```rust
// adapter/src/tray_helper.rs 新增
/// 检查是否以 --tray-helper 模式运行。如果是，执行 helper 循环并返回 true。
pub fn maybe_run_as_helper() -> bool {
    #[cfg(target_os = "linux")]
    if std::env::args().any(|a| a == "--tray-helper") {
        // 直接运行 helper 并退出
        std::process::exit(match run() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("Tray helper failed: {e}");
                1
            }
        });
    }
    false
}

/// 在提权前执行平台相关的预初始化
pub fn pre_init() {
    #[cfg(all(target_os = "linux", not(debug_assertions)))]
    if !crate::is_elevated() {
        let _ = spawn();
    }
}
```

**操作**：
1. `tray_helper.rs` 新增 `maybe_run_as_helper()` 和 `pre_init()`
2. `main.rs` 简化为：

```rust
fn main() -> anyhow::Result<()> {
    rabbit_diag::log("entering main()");
    // ... env 诊断日志 ...

    adapter::tray_helper::maybe_run_as_helper();  // ← 统一入口，内部 exit 或返回

    #[cfg(not(debug_assertions))]
    {
        adapter::tray_helper::pre_init();          // ← 统一入口
        // ... ensure_elevated() ...
    }

    // ... 后续初始化 ...
}
```

**注意**：`maybe_run_as_helper()` 使用 `std::process::exit()` 而非 `return`，避免控制流复杂化。原因是 helper 模式下进程应完全退出，不共享任何初始化路径。当前代码也以 `return Ok(())` 退出 main，改用 `exit()` 后行为一致（进程终止），且调用方可省略后续检查。

**验证**：Linux 环境手动测试 elevation + tray 功能完整。

---

### 2.6 平台检测函数迁移（V12）— P0b

**现状**：`upgrade/models.rs` 中 `current_platform()` 函数检测 OS + 架构，返回 `"linux-x64"` 等字符串。被 `VersionsManifest::for_current_platform()` 内部调用。

**更正**：不应简单"移走函数"而破坏调用链。更优方案是**在 adapter 提供新函数 + 原处委托**：

```rust
// adapter/src/platform.rs 新增
/// 返回当前平台的标识字符串，例如 "linux-x64", "windows-arm64"
pub fn current_platform() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))] { "windows-x64" }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))] { "windows-arm64" }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))] { "linux-x64" }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))] { "linux-arm64" }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))] { "macos-x64" }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))] { "macos-arm64" }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))] { "unsupported" }
}
```

```rust
// upgrade/models.rs — 保持原位但委托
fn current_platform() -> &'static str {
    adapter::platform::current_platform()
}
```

这样不破坏 `VersionsManifest::for_current_platform()` 和 `format_summary()` 等现有调用链。

**操作**：
1. `adapter/src/platform.rs` 新增 `current_platform()`
2. `upgrade/models.rs` 中 `current_platform()` 改为委托调用
3. 可选：在所有第三方调用点逐步替换为直接使用 `adapter::platform::current_platform()`

**验证**：`cargo build` 通过。

---

### 2.7 Upgrade 安装器模块迁移（V13）— P0b

**现状**：`upgrade/installer.rs` 整体包含按平台分支的安装逻辑。需要用到的外部依赖：

| 依赖 | 当前在 `app/Cargo.toml` | 是否需要移至 `adapter/Cargo.toml` |
|---|---|---|
| `sha2` | ✅ `sha2 = "0.10"` | ✅ 需要 |
| `std::os::unix` | 标准库 | 标准库，无需声明 |

**方案**：整体移至 `crates/adapter/src/installer.rs`

**操作**：
1. 复制 `installer.rs` → `adapter/src/installer.rs`
2. `adapter/Cargo.toml` 添加 `sha2 = "0.10"`
3. `adapter/src/lib.rs` 添加 `pub mod installer;`
4. `app/src/upgrade/mod.rs` 删除 `pub mod installer;` 和 `pub use installer::*;`
5. `app/src/app.rs` 中 `crate::upgrade::install_update(...)` → `adapter::installer::install_update(...)`
6. 删除原 `app/src/upgrade/installer.rs`

**验证**：`cargo build` 通过。

---

### 2.8 构建脚本（V14）

**分析**：`build.rs` 中的 `#[cfg(target_os = "windows")]` 用于资源嵌入。属于构建基础设施，不属于运行时四层架构。

**结论**：✅ **无需处理**。

---

## 3. 阶段划分与执行计划

### 3.1 依赖关系图

```
          第一阶段 ─── 可以完全并行
          ├── P0a: V1, V2（settings_tab 清理重复函数）
          │   验证: cargo build
          │
          ├── P0b: V12, V13（platform检测 + installer 模块移动）
          │   验证: cargo build
          │   注意: V13 需要修改 adapter/Cargo.toml 添加 sha2
          │
          第二阶段 ─── 依赖 P0a 完成
          ├── P1: V3-V9（systray 统一 + X11初始化 + temp_dir）
          │   验证: cargo build + Linux 功能测试
          │
          第三阶段 ─── 依赖 P1 完成
          └── P2: V10, V11（main.rs 入口简化）
              验证: cargo build + Linux 完整流程测试
```

### 3.2 第一阶段：P0a — 清理重复函数（安全重构）✅ 已完成

| 步骤 | 文件 | 操作 | 风险 | 状态 |
|---|---|---|---|---|
| 1.1 | `settings_tab.rs` | 删除 `open_url()`，调用点替换为 `adapter::dialog::open_url()` | 🟢 极低 | ✅ |
| 1.2 | `settings_tab.rs` | 删除 `open_folder()`，调用点替换为 `adapter::dialog::open_file_manager()` | 🟢 极低 | ✅ |
| 1.3 | `settings_tab.rs` | 保留 `get_config_folder()`（纯逻辑，无平台依赖） | 🟢 无需操作 | ✅ |

**预计改动**：约 20 行删除，6 行替换。可独立提交（不 push）。

### 3.3 第一阶段：P0b — 模块移动（中等风险）✅ 已完成

| 步骤 | 文件 | 操作 | 风险 | 状态 |
|---|---|---|---|---|
| 2.1 | `adapter/src/platform.rs` | 新增 `current_platform()` 函数 | 🟢 低 | ✅ |
| 2.2 | `upgrade/models.rs` | `current_platform()` 改为委托 `adapter::platform::current_platform()` | 🟢 低 | ✅ |
| 2.3 | `adapter/Cargo.toml` | 添加 `sha2 = "0.10"` | 🟢 低 | ✅ |
| 2.4 | `adapter/src/installer.rs` | 新建，内容来自 `app/src/upgrade/installer.rs` | 🟡 中 | ✅ |
| 2.5 | `adapter/src/lib.rs` | 添加 `pub mod installer;` | 🟢 低 | ✅ |
| 2.6 | `app/src/upgrade/mod.rs` | 删除 `pub mod installer;` 和 `pub use installer::*;` | 🟢 低 | ✅ |
| 2.7 | `app/src/app.rs` | 调用点改为 `adapter::installer::install_update()` | 🟢 低 | ✅ |
| 2.8 | `app/src/upgrade/installer.rs` | 删除原文件 | 🟢 低 | ✅ |

**注意**：步骤 2.3 和 2.4 有顺序依赖（先加依赖再写文件），其余可与 P0a 完全并行。

### 3.4 第二阶段：P1 — 核心抽象重构（较高风险）✅ 已完成

| 步骤 | 文件 | 操作 | 风险 | 状态 |
|---|---|---|---|---|
| 3.1 | `adapter/src/systray.rs` | 新建文件，实现 `init()`/`cleanup()`/`update()` | 🔴 高 | ✅ |
| 3.2 | `adapter/src/lib.rs` | 添加 `pub mod systray;` | 🟢 低 | ✅ |
| 3.3 | `adapter/src/lib.rs` | 新增 `install_platform_handlers()` | 🟢 低 | ✅ |
| 3.4 | `app/src/app.rs` | systray 初始化调用 `adapter::systray::init()` | 🟡 中 | ✅ |
| 3.5 | `app/src/app.rs` | systray 清理调用 `adapter::systray::cleanup()` | 🟡 中 | ✅ |
| 3.6 | `app/src/app.rs` | systray 更新调用 `adapter::systray::update()` | 🟡 中 | ✅ |
| 3.7 | `app/src/app.rs` | X11 初始化改为 `adapter::install_platform_handlers()` | 🟢 低 | ✅ |
| 3.8 | `app/src/app.rs` | temp_dir 改为统一 `std::env::temp_dir().join("rabbit_update")` | 🟢 低 | ✅ |
| 3.9 | `app/src/app.rs` | 删除所有 `#[cfg(target_os = "linux")]` 分支 | 🟢 低 | ✅ |

**风险控制**：
- 步骤 3.1 需仔细实现：`systray.rs` 应完整复制现有 app.rs 中的控制流逻辑，不改变行为
- 步骤 3.4-3.6 需在 Linux 上验证：systray 初始化、显示/隐藏、退出流程
- 建议步骤 3.7 和 3.8 可独立提前执行（与 P0a 并行）

### 3.5 第三阶段：P2 — 入口简化（中等风险）✅ 已完成

| 步骤 | 文件 | 操作 | 风险 | 状态 |
|---|---|---|---|---|
| 4.1 | `adapter/src/tray_helper.rs` | 新增 `maybe_run_as_helper()` | 🟡 中 | ✅ |
| 4.2 | `adapter/src/tray_helper.rs` | 新增 `pre_main_init()` | 🟡 中 | ✅ |
| 4.3 | `app/src/main.rs` | 简化入口，删除 `#[cfg]` 分支 | 🟡 中 | ✅ |

**风险控制**：
- `maybe_run_as_helper()` 使用 `process::exit()` 替代 `return`，确保 helper 模式彻底退出
- `pre_main_init()` 内部保持 `#[cfg]` 保护，非 Linux 为空操作
- 验证 elevation 流程：`cargo build --release && sudo ./target/release/rabbit`

---

## 4. 预期效果

### 4.1 指标变化

| 指标 | 当前 | P0a 后 | P0b 后 | P1 后 | P2 后 | **实际** |
|---|---|---|---|---|---|---|---|
| `app/` 中 `target_os` 次数 | 19 | 13 | 10 | 0 | 0 | **0** ✅ |
| `app/` 中条件编译行数 | ~85 | ~60 | ~45 | 0 | 0 | **0** ✅ |
| 重复 adapter 函数 | 2 处 | 0 处 | 0 处 | 0 处 | 0 处 | **0** ✅ |
| 可编译性 | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |
| app/Cargo.toml 条件依赖 | 3 项 | 3 项 | 3 项 | 3 项 | 3 项 | **0** ✅ |

> **额外成果**：P2 后进一步清理了 `app/Cargo.toml` 中的平台条件依赖（`ksni`/`libc`/`windows-sys`），全部移至 adapter 层。这超出了原始计划的指标范围。

### 4.2 架构改进

| 维度 | 当前 | 优化后 |
|---|---|---|
| **单一职责** | `app.rs` 知道 systray helper、D-Bus、进程提权等平台细节 | `app.rs` 只调 `adapter::systray::init()`，不关心底层 |
| **平台隔离** | 平台分支散落在表现层各处 | 所有 `#[cfg(target_os)]` 集中在 `adapter` |
| **可测试性** | 无法 mock 平台行为测试 UI | adapter 层可独立测试 |
| **代码量** | 重复函数 + 条件分支约 100 行 | 干净架构，约减少 80 行 |
| **新增平台支持** | 需修改 6 个 app 文件 | 只改 adapter |

### 4.3 迁移后架构示意

```
迁移前:

app/                                adapter/
├── main.rs                         ├── tray/          ← 已有统一 API
│   #[cfg(linux)] 2x                │   ├── mod.rs (dispatch)
├── app.rs                          │   ├── linux.rs
│   #[cfg(linux)] 5x                │   └── non_linux.rs
│   #[cfg(not(linux))] 3x           ├── x11_diag.rs
│   cfg!(windows)                   ├── dialog.rs
├── ui/settings_tab.rs              │   open_url()
│   #[cfg] 6x                       │   open_file_manager()
├── upgrade/                        ├── tray_helper.rs  ← 纯 Linux, 但 IPC 控制流在 app
│   ├── models.rs #[cfg]            └── ...
│   └── installer.rs #[cfg]
└── build.rs [cfg] ← OK

迁移后:

app/                                adapter/
├── main.rs                         ├── systray.rs      ← 新增: 统一入口
├── app.rs                          ├── tray/           ← 维持不变
├── ui/settings_tab.rs              │   ├── mod.rs
├── upgrade/                        │   ├── linux.rs
│   └── models.rs (委托调用)         │   └── non_linux.rs
└── build.rs [cfg] ← OK            ├── installer.rs    ← 从 app 移入
                                    ├── platform.rs     ← 新增: current_platform()
                                    ├── tray_helper.rs  ← 维持不变
                                    ├── x11_diag.rs     ← 维持不变
                                     └── dialog.rs
```

---

### 4.4 执行结果

| 维度 | 计划目标 | 实际成果 |
|---|---|---|
| P0a 清理重复 | 删除 `open_url()`/`open_folder()`，改用 `adapter::dialog` | ✅ 完成 |
| P0b 模块移动 | `current_platform()` 委托 + `installer.rs` 移至 adapter | ✅ 完成，+额外: `getuid()` 也封装至 `adapter::platform` |
| P1 systray 统一 | 创建 `adapter::systray`，删除 app.rs 全部 `#[cfg]` | ✅ 完成 |
| P2 入口简化 | `maybe_run_as_helper()` + `pre_main_init()`，main.rs 零条件编译 | ✅ 完成，+额外: C4 一并解决 |
| 依赖清理 | 未列入原始计划 | ✅ 超额: 删除 app/Cargo.toml 中 `ksni`/`libc`/`windows-sys` 三项条件依赖 |
| `app/` 中 `target_os` | 0 处 | ✅ 0 处 |
| `app/Cargo.toml` 条件依赖 | 未列入原始指标 | ✅ 0 项 |
| 编译验证 | `cargo build` + `cargo clippy` | ✅ 均通过，无新警告 |

**核心差异**：实际执行比原始计划多覆盖了 `app/Cargo.toml` 条件依赖清理（原始计划未考虑此维度）。

---

## 5. 风险与验证策略

### 5.1 各阶段验证方法

| 阶段 | 验证命令 | 验证内容 | 预期耗时 | 状态 |
|---|---|---|---|---|---|
| P0a | `cargo build` | 编译通过 | < 1 分钟 | ✅ 通过 |
| P0b | `cargo build` | 编译通过，无警告 | < 1 分钟 | ✅ 通过 |
| P1 | `cargo build` + Linux 手动测试 | systray 显示/隐藏/退出正常 | ~ 10 分钟 | ✅ 通过 |
| P2 | `cargo build --release` + elevation 测试 | sudo 启动后 tray 正常工作 | ~ 15 分钟 | ✅ 通过 |

### 5.2 已知风险

| 风险 | 等级 | 涉及阶段 | 缓解措施 |
|---|---|---|---|
| systray 统一抽象可能引入回归 | 🔴 高 | P1 | 每个步骤后编译验证 + Linux 完整功能测试 |
| tray_helper IPC 控制流被破坏 | 🔴 高 | P1, P2 | 特别关注 `connect`/`show_tray`/`hide_tray` 调用顺序 |
| `main.rs` helper 退出路径变更 | 🟡 中 | P2 | `process::exit()` 确保与当前 `return Ok(())` 行为一致 |
| installer 模块缺少 sha2 依赖 | 🟡 中 | P0b | 先加 adapter/Cargo.toml 依赖再移动文件 |
| 非 Linux 平台无法测试 | 🟡 中 | 全部 | 代码审查确认 `#[cfg(not(target_os = "linux"))]` 路径正确 |

### 5.3 回退方案

| 场景 | 操作 |
|---|---|
| 某步骤编译失败 | 立即修复，不累积未验证的变更 |
| P1 systray 功能异常 | 回退 P1 所有变更，保留 P0 成果 |
| P2 elevation 流程中断 | 回退 P2，保留 P0+P1 |
| 全局性问题 | 整个阶段 `git checkout` 相关文件 |

### 5.4 注意事项

- **增量提交**：每个逻辑单元完成后 commit（但不 push），便于 isolate 问题
- **不引入行为变更**：所有重构只移动/抽象代码，不改变运行时行为
- **主开发环境为 Linux**：systray helper + elevation 路径务必完整测试
- **跨平台检查**：在 Linux 开发时，通过 `#[cfg]` 静态检查确认 Windows/macOS 路径语义正确
- **保留过渡导出**：第一阶段可为 adapter 模块添加 `pub use` 兼容路径，方便逐步迁移外部调用

---

## 附录 A：违规分布（执行前 → 执行后）

```
执行前: 19 处 / 6 个文件         执行后: 0 处 / 0 个文件 ✅

crates/app/src/main.rs          → P2: 封装到 adapter::tray_helper
  24: #[cfg(target_os = "linux")]     ✅ 已消除 → adapter::tray_helper::maybe_run_as_helper()
  39: #[cfg(target_os = "linux")]     ✅ 已消除 → adapter::tray_helper::pre_main_init()

crates/app/src/app.rs           → P1: 统一 adapter::systray 封装
  65: #[cfg(target_os = "linux")]     ✅ 已消除 → adapter::install_platform_handlers()
  266: #[cfg(target_os = "linux")]    ✅ 已消除 → adapter::install_platform_handlers()
  554: #[cfg(target_os = "linux")]    ✅ 已消除 → adapter::systray::init()
  603: #[cfg(not(target_os = "linux"))] ✅ 已消除 → adapter::systray::init()
  642: #[cfg(target_os = "linux")]    ✅ 已消除 → adapter::systray::cleanup()
  649: #[cfg(not(target_os = "linux"))] ✅ 已消除 → adapter::systray::cleanup()
  718: cfg!(target_os = "windows")    ✅ 已消除 → std::env::temp_dir()
  1094: #[cfg(target_os = "linux")]   ✅ 已消除 → adapter::systray::update()
  1110: #[cfg(not(target_os = "linux"))] ✅ 已消除 → adapter::systray::update()

crates/app/src/ui/settings_tab.rs → P0a: 使用 adapter::dialog
  85: #[cfg(target_os = "windows")]   ✅ 已消除 → adapter::dialog::open_url()
  89: #[cfg(target_os = "macos")]     ✅ 已消除 → adapter::dialog::open_url()
  93: #[cfg(target_os = "linux")]     ✅ 已消除 → adapter::dialog::open_url()
  101: #[cfg(target_os = "windows")]  ✅ 已消除 → adapter::dialog::open_file_manager()
  105: #[cfg(target_os = "macos")]    ✅ 已消除 → adapter::dialog::open_file_manager()
  109: #[cfg(target_os = "linux")]    ✅ 已消除 → adapter::dialog::open_file_manager()

crates/app/src/upgrade/models.rs → P0b: 委托 adapter::platform
  94: #[cfg(target_os = "windows")]   ✅ 已消除 → adapter::platform::current_platform()
  101: #[cfg(target_os = "linux")]    ✅ 已消除 → adapter::platform::current_platform()
  108: #[cfg(target_os = "macos")]    ✅ 已消除 → adapter::platform::current_platform()

crates/app/src/upgrade/installer.rs → P0b: 整体移至 adapter::installer
  40: #[cfg(target_os = "windows")]   ✅ 已消除 → 文件移至 adapter
  45: #[cfg(any(linux, macos))]       ✅ 已消除 → 文件移至 adapter
  56: #[cfg(target_os = "windows")]   ✅ 已消除 → 文件移至 adapter
  94: #[cfg(any(linux, macos))]       ✅ 已消除 → 文件移至 adapter

crates/app/build.rs
  2: #[cfg(target_os = "windows")]   → 无需处理（构建基础设施）
```

> 此外，`app/Cargo.toml` 中的 3 项平台条件依赖（`ksni`/`libc`/`windows-sys`）已全部删除 — 这些依赖在 app 中已无直接使用，移至 adapter 层统一管理。

## 附录 B：相关但合规的 adapter 中 `#[cfg]` 使用（参考示例）

以下文件展示了 adapter 层正确使用 `#[cfg]` 的模式，可作为实现参考：

| 文件 | 模式 | 说明 |
|---|---|---|
| `adapter/src/tray/mod.rs` | `#[cfg(target_os = "linux")] mod imp;` + `pub use` | 编译时分发不同实现 |
| `adapter/src/shell.rs` | `#[cfg] mod windows;` + `#[cfg] pub use` | 按平台选择模块 |
| `adapter/src/notification.rs` | 单一 `pub fn` + 内部 `#[cfg]` 分派 | 统一入口，内部平台分支 |
| `adapter/src/autostart.rs` | `#[cfg] mod` + `pub use` | 按平台选择实现 |

---

## 附录 C：额外发现的优化机会（adapter 层内部）

以下问题不违反"app 层不得有 `#[cfg]`"原则（它们都在 adapter 层内），但属于代码质量改进机会，可在核心迁移完成后逐步处理。

### C1 `adapter/src/tray/non_linux.rs` 中 `show_main_window`/`hide_main_window` 的 `#[cfg]` 冗余

**位置**：行 142-178

**问题**：`show_main_window()` 和 `hide_main_window()` 内部使用函数体 `#[cfg]` 在 Windows/非 Windows 间分派：

```rust
fn show_main_window() {
    // ...
    fltk::app::awake_callback(move || {
        #[cfg(target_os = "windows")]
        { crate::window::show_window(); }
        #[cfg(not(target_os = "windows"))]
        { win.show(); }
    });
}

fn hide_main_window() {
    // ... 同上模式
}
```

**分析**：`adapter::window` 已经提供了 `show_window()` 和 `hide_window()`，且在非 Windows 平台上定义为空操作（行 195-199）：

```rust
#[cfg(not(target_os = "windows"))]
pub fn hide_window() {}
#[cfg(not(target_os = "windows"))]
pub fn show_window() {}
```

所以此处可以直接简化为无条件调用：

```rust
fn show_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            fltk::app::awake_callback(move || {
                crate::window::show_window();  // ← 无条件调用，非 Windows 为空操作
            });
        }
    }
}
```

**改进**：删除 2 组（共 4 个）`#[cfg]`，减少 8 行模板代码。

**优先级**：🟢 低 — 可独立于主计划执行。

---

### C2 `adapter/src/elevation.rs` 中的 android/ios 死代码

**位置**：行 38-42、行 58-60、行 93-96、行 206-209

**问题**：`elevation.rs` 中多处使用 `target_os = "android"` 和 `target_os = "ios"` 条件编译：

```rust
pub fn is_elevated() -> bool {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    { let _ = true; false }
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    { xelevate::is_elevated() }
}
```

**分析**：此项目仅面向 **Windows / Linux / macOS** 桌面平台。Android 和 iOS 从未出现在构建目标中，`Cargo.toml` 也无相关配置。这些分支属于从 `xelevate` 库复制来的模板代码，从未被执行。

**建议**：删除所有 `target_os = "android"` 和 `target_os = "ios"` 条件分支，将 `elevation.rs` 简化为纯粹的桌面平台实现。

**影响范围**：

| 位置 | 当前代码 | 简化后 |
|---|---|---|
| `is_elevated()` 行 38-47 | android/ios 分支 + 兜底 | 直接 `xelevate::is_elevated()` |
| `ensure_elevated()` 行 58-61 | debug + android/ios | 仅保留 `debug_assertions` |
| `elevate_with_sudo_inner()` cfg 行 93-96 | 复杂的组合条件 | 简化为 `#[cfg(target_os = "linux")]` |
| （行 206-209 的复杂 cfg） | 同上 | 简化 |

**优先级**：🟢 低 — 代码清理，无行为影响。

---

### C3 `adapter/src/network.rs` 中 `get_interfaces_unix` 内部的嵌套 `#[cfg]`

**位置**：行 45-58

**问题**：`get_interfaces_unix()` 函数用 `#[cfg(any(target_os = "linux", target_os = "macos"))]` 守卫，但内部仍用 `#[cfg]` 区分 Linux/macOS：

```rust
#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn get_interfaces_unix() -> Result<Vec<NetworkInterface>> {
    #[cfg(target_os = "linux")]
    let _output = Command::new("ip").args(["-j", "addr", "show"]).output()?;

    #[cfg(target_os = "macos")]
    let output = Command::new("ifconfig").output()?;
    // ... macOS 的后续解析也用不到 linux 分支的变量
}
```

**分析**：此模式存在变量作用域问题 — `_output`（Linux）和 `output`（macOS）在不同的条件分支中定义，导致解析代码无法共享。且两个平台使用完全不同的命令和输出格式。

**建议**：拆分为独立函数 `get_interfaces_linux()` 和 `get_interfaces_macos()`，使用模块级 `#[cfg]` 分发：

```rust
#[cfg(target_os = "linux")]
async fn get_interfaces_linux() -> Result<Vec<NetworkInterface>> { ... }
#[cfg(target_os = "macos")]
async fn get_interfaces_macos() -> Result<Vec<NetworkInterface>> { ... }
```

**优先级**：🟢 低 — 当前代码编译正常，但可读性差。建议与 network 模块的功能增强一并处理。

---

### C4 `main.rs` 中外层的 `#[cfg(not(debug_assertions))]` 冗余

**位置**：`app/src/main.rs` 行 37-57

**问题**：`main.rs` 用一个 `#[cfg(not(debug_assertions))]` 块包裹了整个 elevation 流程。但 `adapter::ensure_elevated()` 内部已经用 `#[cfg(any(debug_assertions, ...))]` 处理了 debug 模式：

```rust
// main.rs
#[cfg(not(debug_assertions))]  // ← 外层检查
{
    #[cfg(target_os = "linux")]
    if !adapter::is_elevated() {
        adapter::tray_helper::spawn();
    }
    adapter::ensure_elevated()?;  // ← 内部也有 debug 检查
}

// elevation.rs
pub fn ensure_elevated() -> Result<(), ElevationError> {
    #[cfg(any(debug_assertions, target_os = "android", target_os = "ios"))]
    { return Ok(()); }  // ← debug 模式直接返回
    // ...
}
```

**分析**：当前双重保护不会导致 bug（只是外层提前跳过），但容易造成"如果 `ensure_elevated()` 的逻辑变了，外层可能不同步"的维护问题。P2 阶段将 `pre_init()` 封装到 adapter 后，外层 `#[cfg(not(debug_assertions))]` 可以删除，由 adapter 内部统一处理。

**建议**：在 P2 阶段一并处理 — `pre_init()` 使用内部 `#[cfg]` 包裹，`main.rs` 不再使用 `#[cfg(not(debug_assertions))]`。

**优先级**：🟡 中 — 随 P2 处理。

**状态**：✅ 已解决 — `adapter::tray_helper::pre_main_init()` 内部使用 `#[cfg(all(target_os = "linux", not(debug_assertions)))]` 包裹，`main.rs` 不再使用 `#[cfg(not(debug_assertions))]`。

---

### C5 `cfg(windows)` 与 `cfg(target_os = "windows")` 命名不一致

**位置**：`adapter/Cargo.toml` 使用 `cfg(windows)`，而源代码中统一使用 `cfg(target_os = "windows")`

**说明**：这不是 bug。`cfg(windows)` 是 `cfg(target_os = "windows")` 的缩写形式，二者语义完全等价（`cfg(unix)` 涵盖 Linux + macOS）。Cargo 的 `[target.'cfg(...)'.dependencies]` 节中两种写法都常用。

**建议**：无需处理。仅作为背景信息记录。

**优先级**：⚪ 无需处理。

---

### C6 汇总：额外优化项优先级

| 编号 | 位置 | 问题 | 优先级 | 建议时机 |
|---|---|---|---|---|
| C1 | `adapter/src/tray/non_linux.rs:142-178` | `show_main_window`/`hide_main_window` 内部 `#[cfg]` 冗余 | 🟢 低 | 可随时独立处理 |
| C2 | `adapter/src/elevation.rs` 多处 | android/ios 死代码 | 🟢 低 | 与 P1/P2 并行 |
| C3 | `adapter/src/network.rs:45-58` | `get_interfaces_unix` 嵌套 `#[cfg]` 应拆分为独立函数 | 🟢 低 | 与 network 功能增强一并处理 |
| C4 | `app/src/main.rs:37` | 外层 `#[cfg(not(debug_assertions))]` 冗余 | 🟡 中 | 随 P2 处理 — ✅ 已解决 |
| C5 | `adapter/Cargo.toml:26-29` | `cfg(windows)` vs `cfg(target_os = "windows")` 不一致 | ⚪ 无需 | 不处理 |
