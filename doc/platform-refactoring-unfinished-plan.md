# 平台适配层重构 — 未完成项实施计划

**基准文档**：`doc/modules.md`（v7.1）、`doc/platform-refactoring-plan.md`、`doc/platform-refactoring-execution-plan.md`（v3）
**生成日期**：2026-05-11
**最后更新**：2026-05-11
**状态**：✅ 全部完成

---

## 1. 架构要求（硬性约束）

以下要求来自 `modules.md` 和 `platform-refactoring-plan.md`，是实施过程中必须满足的目标：

| # | 要求 | 来源 | 说明 |
|---|------|------|------|
| R1 | `adapter` 对外提供**平台无关的统一接口** | `modules.md` §adapter | `PlatformService` 是目标形态 |
| R2 | `adapter` 不暴露平台类型到接口 | `modules.md` §adapter | 如 `hwnd: usize` |
| R3 | `adapter` 不含 UI 逻辑 | `platform-refactoring-plan.md` §5.3 | `elevation` 中的 FLTK dialog 必须移除 |
| R4 | 所有平台代码集中在 `adapter` | `platform-refactoring-plan.md` §3 | `systray`、`tray_helper`、`x11_diag` 从 app 移入 |
| R5 | `Lifecycle` 需移到 adapter 或抽象为 trait | `platform-refactoring-plan.md` §7 | 消除跨 crate 依赖 |
| R6 | 模块依赖单向：`app → service → schema`，`app → adapter → schema` | `modules.md` §依赖关系 | `service` 不依赖 `adapter`（已完成） |

---

## 2. 已完成的步骤

| 步骤 | 状态 | 关键交付 |
|------|------|---------|
| 批次 0 (config + diag 提取) | ✅ | `rabbit-config`、`rabbit-diag` 独立 crate，import 替换完成 |
| Step 2 部分 (NetworkProvider) | ✅ | `schema::network::NetworkProvider` trait 定义 + `adapter::DefaultNetworkProvider` 实现 |
| Step 3 (service 解耦) | ✅ | `service/Cargo.toml` 彻底移除 `adapter` 依赖 |
| x11_diag 移入 adapter | ✅ | `adapter/src/x11_diag.rs` |
| F1/F2/F3 改用 adapter 接口 | ✅ | `adapter::dialog::open_url()` / `open_file_manager()` |
| 批次 1 (Phase 0a+0b): Lifecycle + icon → adapter | ✅ | `adapter/src/lifecycle.rs` + `adapter/src/icon.rs`；`app/src/` 中无 `lifecycle.rs`/`icon.rs` |
| 批次 2 (Phase 1a+1b): PlatformService + elevation 解耦 | ✅ | `adapter/src/platform.rs` 统一 re-export；hwnd 统一 `OnceLock` 存储；`ensure_elevated()` 返回 `Result`；`elevation.rs` 无 `fltk` |
| 批次 3 (Phase 1c): systray → adapter | ✅ | `adapter/src/tray/{mod,linux,non_linux}.rs`；`non_linux.rs` 无 `unsafe`/`static mut`；`app/src/` 无 `systray*.rs` |
| 批次 4 (Phase 2a): tray_helper → adapter | ✅ | `adapter/src/tray_helper.rs`；`app/src/tray_helper.rs` 删除；`main.rs` 调用改为 `adapter::tray_helper` |
| 额外清理：icon.ico 统一 | ✅ | 两份 `resources/icon.ico` 删除，`resources/icon.ico`（工作区根目录）唯一源头；`ico_bytes()` 暴露原始字节 |
| 额外修复：`is_active()` helper 感知 | ✅ | `tray/linux.rs::is_active()` 同时检查本地 ksni 和 helper 状态，修复设置页托盘切换失效 bug |

---

## 3. 项目 A：Lifecycle 移入 adapter ✅

**状态**：已完成。`adapter/src/lifecycle.rs` 存在，`app/src/lifecycle.rs` 已删除。`adapter/src/lib.rs` 中 `pub use lifecycle::Lifecycle;`。`app/src/` 中无 `crate::lifecycle` 引用。`cargo test -p adapter` 包含 3 个 Lifecycle 测试全部通过。

> 详细操作步骤见 §3.3（已执行），此处省略。

---

## 4. 项目 B：icon 移入 adapter ✅

**状态**：已完成。`adapter/src/icon.rs` 存在，包含 `load_app_icon()` 和新增的 `ico_bytes()`。`app/src/icon.rs` 已删除。`app/resources/` 和 `adapter/resources/` 的 `icon.ico` 已合并到工作区根目录 `resources/icon.ico`，两个 crate 均通过 `include_bytes!` 指向该位置。`adapter/src/icon.rs` 的 `ico_bytes()` 暴露原始 `.ico` 字节供 `app.rs` 通过 `IcoImage::from_data()` 设置窗口图标。

> 详细操作步骤见 §4.3（已执行），此处省略。

---

## 5. 项目 C：PlatformService 统一入口 ✅

**状态**：已完成。`adapter/src/platform.rs` 存在，通过 re-export 提供统一入口。`window.rs` 使用 `OnceLock<usize>` 统一存储 `MAIN_HWND`，`set_window_on_top()` 不再接收 hwnd 参数。`taskbar.rs::set_main_window_hwnd()` 委托到 `crate::window::set_main_window`。`adapter/src/lib.rs` 有 `pub mod platform;` + `pub use platform::*;`。

> 详细方案见 §5.3（已实现），此处省略。

---

## 6. 项目 D：elevation FLTK dialog 解耦 ✅

**状态**：已完成。`adapter/src/elevation.rs` 中无 `use fltk` 语句，无 `show_elevation_error` 函数。`ensure_elevated()` 返回 `Result<(), ElevationError>`。`app/src/main.rs` 处理 `Result`，失败时通过 `fltk::dialog::alert_default()` 显示错误。`ElevationError` 有手动 `Display` 实现。`cargo build --release` 验证通过。

> 详细方案见 §6.3（已实现），此处省略。

---

## 7. 项目 E：systray 迁移到 adapter/tray/ ✅

**状态**：已完成。`adapter/src/tray/{mod,linux,non_linux}.rs` 均存在。`app/src/` 中无 `systray*.rs`。`non_linux.rs` 使用 `Mutex<Option<Box<TrayIcon>>>` 替代 `static mut`（无 unsafe）。`is_active()` 在 Linux 上同时考虑本地 ksni 和 helper 状态（§7.4 的待办项已在 tray_helper 迁入 adapter 后合并）。`cargo build --release` 通过。

> 详细方案见 §7.5（已实现），此处省略。

---

## 8. 项目 F：tray_helper 迁移到 adapter ✅

**状态**：已完成。`adapter/src/tray_helper.rs` 存在（从 `app/src/tray_helper.rs` 复制，`adapter::Lifecycle` → `crate::Lifecycle`）。`app/src/tray_helper.rs` 已删除。`main.rs` 中 `--tray-helper` 入口改为 `adapter::tray_helper::run()`。`app.rs` 中所有 `crate::tray_helper::*` 改为 `adapter::tray_helper::*`。`include_bytes!` 路径指向工作区根目录 `resources/icon.ico`。`cargo build --release` + `cargo test -p adapter` 通过。

### 8.1 `is_active()` 的整合

tray_helper 迁入 adapter 后，`adapter/src/tray/linux.rs` 的 `is_active()` 已恢复原始行为——同时检查本地 ksni 和 helper 状态：

```rust
pub fn is_active() -> bool {
    if crate::tray_helper::is_connected() {
        crate::tray_helper::is_tray_visible()
    } else {
        SYSTRAY_ENABLED.load(Ordering::SeqCst)
    }
}
```

这修复了设置页托盘切换失效的 bug（此前 `is_active()` 只查 `SYSTRAY_ENABLED`，当 helper 管理托盘时始终返回 `false`）。

---

## 9. 执行结果

### 9.1 依赖关系（实际执行顺序）

```
Phase 0: Foundation（并行）
  ├── 0a. Lifecycle → adapter
  └── 0b. icon → adapter
        ─┐
         ▼
Phase 1: Implementation — 独立项（并行）
  ├── 1a. PlatformService
  └── 1b. elevation 解耦
        ─┐
         ▼
Phase 2: Implementation — 依赖项
  └── 1c. systray → adapter（依赖 1a 的 window.rs API 变更）
        ─┐
         ▼
Phase 3: 可选 → 实际已执行
  └── 2a. tray_helper → adapter

Phase 4: 额外清理
  └── icon.ico 合并 + bug 修复
```

### 9.2 实际执行批次

```
批次 1: Phase 0a + 0b 并行 ✅
        Lifecycle 搬入 + icon 搬入 + 更新引用 → cargo check 通过

批次 2: Phase 1a + 1b 并行 ✅
        PlatformService（window.rs API 变更 + platform.rs 新建 + hwnd 集中）
        elevation 解耦（删除 FLTK dialog，函数签名变更）
        cargo check 通过

批次 3: Phase 1c ✅
        systray 三个文件搬迁到 adapter/tray/
        non_linux.rs unsafe 修复（static mut → Mutex<Option<Box<TrayIcon>>>）
        cargo build --release 编译通过

批次 4: Phase 2a ✅（原为可选，按用户要求执行）
        tray_helper 全量搬入 adapter
        main.rs + app.rs 调用改为 adapter::tray_helper::
        cargo build --release 通过

批次 5: 额外清理 ✅
        app/src/icon.rs 删除（无主文件）
        app/resources/ + adapter/resources/ 的 icon.ico 删除
        resources/icon.ico 移至工作区根目录（唯一源头）
        adapter::icon::ico_bytes() 新增供 app 层使用
        tray/linux.rs::is_active() 修复（增加 helper 感知）
```

### 9.3 最终文件变更清单

#### 批次 1a：Lifecycle → adapter

| 操作 | 文件 |
|------|------|
| 新建 | `adapter/src/lifecycle.rs` |
| 修改 | `adapter/src/lib.rs` — 添加 `pub mod lifecycle;` |
| 修改 | `app/src/lib.rs` — 移除 `pub mod lifecycle;` |
| 删除 | `app/src/lifecycle.rs` |
| 修改 | `app/src/app.rs` — `crate::lifecycle` → `adapter::Lifecycle` |
| 修改 | `app/src/systray_linux.rs` — 同上 |
| 修改 | `app/src/systray_non_linux.rs` — 同上 |
| 修改 | `app/src/tray_helper.rs` — 同上 |

#### 批次 1b：icon → adapter

| 操作 | 文件 |
|------|------|
| 复制 | `app/resources/icon.ico` → `adapter/resources/icon.ico` |
| 新建 | `adapter/src/icon.rs` |
| 修改 | `adapter/src/lib.rs` — 添加 `pub mod icon;` |
| 修改 | `adapter/Cargo.toml` — 添加 `image = "0.25"` |
| 修改 | `app/src/systray_linux.rs` — `crate::icon` → `adapter::icon` |
| 修改 | `app/src/systray_non_linux.rs` — 同上 |

#### 批次 2a：PlatformService

| 操作 | 文件 |
|------|------|
| 新建 | `adapter/src/platform.rs` — 统一 re-export |
| 修改 | `adapter/src/window.rs` — 增加 `MAIN_HWND` static + 无 hwnd 版本接口 |
| 修改 | `adapter/src/taskbar.rs` — hwnd 存储委托到 `window::set_main_window` |
| 修改 | `adapter/src/lib.rs` — 添加 `pub mod platform;` + `pub use platform::*;` |
| 修改 | `app/src/app.rs` — `set_main_window(hwnd)` 一次注册，消除 hwnd 传参 |

#### 批次 2b：elevation 解耦

| 操作 | 文件 |
|------|------|
| 修改 | `adapter/src/elevation.rs` — `ensure_elevated` 返回 `Result`，删除 FLTK dialog |
| 修改 | `app/src/main.rs` — 处理 `ensure_elevated()` 的 `Result`，显示 FLTK dialog |

#### 批次 3a：systray → adapter

| 操作 | 文件 |
|------|------|
| 新建 | `adapter/src/tray/mod.rs` |
| 新建 | `adapter/src/tray/linux.rs`（从 `app/systray_linux.rs` 移植） |
| 新建 | `adapter/src/tray/non_linux.rs`（从 `app/systray_non_linux.rs` 移植） |
| 修改 | `adapter/src/lib.rs` — 添加 `pub mod tray;` |
| 修改 | `adapter/Cargo.toml` — 添加条件依赖 `ksni`、`tray-icon` |
| 删除 | `app/src/systray.rs` |
| 删除 | `app/src/systray_linux.rs` |
| 删除 | `app/src/systray_non_linux.rs` |
| 修改 | `app/src/lib.rs` — 移除 `pub mod systray;` |
| 修改 | `app/src/app.rs` — `crate::systray` → `adapter::tray` |

---

## 10. 验收标准

### 批次 1 验收
- [x] `cargo check` 通过
- [x] `cargo test -p adapter` 中 `Lifecycle` 的 3 个单元测试通过
- [x] app 中不再有 `crate::lifecycle::Lifecycle` 引用（已改为 `adapter::Lifecycle`）
- [x] `adapter/src/icon.rs` 存在且可被 app 层引用
- [ ] 应用图标在托盘和窗口标题栏正常显示（需运行时验证，无法自动化确认）

### 批次 2 验收（PlatformService + elevation）
- [x] `adapter::set_window_on_top(bool)` **不接收 hwnd 参数**
- [x] `adapter::set_main_window(hwnd)` 在所有 hwnd 消费者之前被调用
- [x] `adapter/src/elevation.rs` 中无 `use fltk` 语句
- [x] `ensure_elevated()` 返回 `Result<(), ElevationError>`
- [x] `cargo check` 通过

### 批次 3 验收（systray）
- [x] `adapter/src/tray/` 包含 mod.rs、linux.rs、non_linux.rs
- [x] `app/src/` 中无 `systray*.rs`
- [x] `non_linux.rs` 中无 `unsafe`/`static mut`（已改为 `Mutex<Option<Box<TrayIcon>>>`）
- [ ] 托盘功能正常：创建/显示/隐藏/退出/Quit 触发 shutdown（需运行时验证）
- [x] `app/src/` 中 `#[cfg(...)]` 数量减少（初始约 53 处 → 当前 32 处，降幅约 40%）
- [x] `cargo build --release` 编译通过

---

## 11. 工作量估算

| 项目 | 文件变更 | 风险 | 说明 |
|------|---------|------|------|
| 0a. Lifecycle 搬家 | ~8 文件 | 低 | 纯文件移动 + import 替换 |
| 0b. icon 搬家 | ~5 文件 | 低 | 文件复制 + import 替换 |
| 1a. PlatformService | ~5 文件 | 低 | 组织性模块 + hwnd 存储集中 |
| 1b. elevation 解耦 | ~2 文件 | 中 | 函数签名变更，需要验证 main.rs 控制流 |
| 1c. systray 搬迁 | ~10 文件 | 中 | 文件最多，需验证 Linux + 非 Linux 托盘 |
| 2a. tray_helper | ~5 文件 | 高 | IPC + ksni + FLTK 混合，待评估 |
