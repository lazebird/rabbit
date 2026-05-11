# 平台适配层重构方案

## 1. 目标

将项目中的所有平台相关代码集中到 `adapter` crate 中，`adapter` 对外提供**平台无关的统一接口**，其他业务模块（尤其是 `service` 层）不感知平台差异。

### 核心原则

- **`adapter` 只做平台适配**：提供平台相关的接口抽象和封装，不含业务逻辑
- **业务代码不直接触碰平台细节**：不出现 `#[cfg(windows)]`、`unsafe` 平台 API、`os::unix` 等
- **接口统一**：`adapter` 对外的 API 是纯平台无关的，不暴露句柄、平台类型等

---

## 2. 现状分析

### 2.1 当前 crate 依赖关系

```
app ─→ adapter, service, schema
                ↑
service ────────┘    ← 问题：service 直接依赖 adapter
adapter ─→ schema
```

### 2.2 平台代码三分散

#### 位置 A: `crates/adapter/` — 已有平台适配

| 模块 | 行数 | 包含业务逻辑？ | 评价 |
|------|------|--------------|------|
| `config.rs` | 127 | **是**（缓存、脏检查、merge_defaults） | ❌ 不应在 adapter |
| `diag.rs` | 35 | 否（纯工具） | ❌ 不应在 adapter |
| `dialog.rs` | 281 | 否 | ✅ 平台适配 |
| `autostart.rs` | 137 | 否 | ✅ 平台适配 |
| `shell.rs` | 110 | 否 | ✅ 平台适配 |
| `taskbar.rs` | 209 | 否 | ✅ 平台适配 |
| `window.rs` | 57 | 否（但暴露 `hwnd: usize`） | ⚠️ 接口需净化 |
| `elevation.rs` | 252 | 否 | ✅ 平台适配 |
| `notification.rs` | 102 | 否 | ✅ 平台适配 |
| `ping.rs` | 71 | 少量（resolve 是通用 DNS） | ⚠️ 需拆分 |
| `network.rs` | 110 | **部分是**（calculate_ip_range） | ⚠️ 需拆分 |

#### 位置 B: `crates/app/` — 仍分散着平台代码

| 文件 | 内容 | 行数 | 应移至 adapter？ |
|------|------|------|----------------|
| `systray_linux.rs` | ksni D-Bus 托盘 | 222 | ✅ |
| `systray_non_linux.rs` | tray-icon 托盘 | 186 | ✅ |
| `systray.rs` | 平台分派 | 21 | ✅ |
| `tray_helper.rs` | Unix socket IPC | ~450 | ✅ |
| `app.rs:70-129` | X11 错误处理器 | 60 | ✅ |
| `app.rs:493-527` | F1/F2/F3 各平台不同命令 | 35 | ✅（改用 adapter 接口） |
| `upgrade/installer.rs` | 安装器平台路径/权限 | ~150 | ⚠️ 评估后决定 |
| `settings_tab.rs:85-109` | 语言/主题平台名称 | 25 | ❌ 纯文本，留在 app |

#### 位置 C: `crates/service/` — 直接调用 adapter 内部

| 文件 | 调用 | 问题 |
|------|------|------|
| `scan.rs:84` | `adapter::config::load_config()` | service 越界依赖平台 |
| `scan.rs:167,285` | `adapter::network::get_mac_from_arp()` | service 越界依赖平台 |
| `ping.rs:225` | `adapter::config::load_config()` | 同上 |
| `http.rs:15` | `adapter::config::{get_bool, get_integer, ...}` | 同上 |
| `tftpd.rs:7,36` | `adapter::config::{get_integer, get_array}` | 同上 |
| `tftpc.rs:7` | `adapter::config::{get_integer, get_string}` | 同上 |
| `chat.rs:4` | `adapter::config::{get_integer, get_string}` | 同上 |
| `lib.rs:76` | `#[from] adapter::PlatformError` | 错误类型耦合 |

---

## 3. 目标架构

### 3.1 依赖关系（目标）

```
app ─→ adapter, service, config
                │
service ────────┤    ← service 不再依赖 adapter
                │
adapter ────────┼──→ schema    （纯平台适配，零业务逻辑）
                │
config ─────────┘──→ schema    （配置管理，独立于 platform）
```

### 3.2 Adapter 的职责边界

```
┌─────────────────────────────────────────────────┐
│                  adapter                         │
│                                                   │
│  public API（平台无关）                            │
│  ┌─────────────────────────────────────────────┐  │
│  │ PlatformService {                          │  │
│  │   open_url(url)                            │  │
│  │   open_file_dialog(...)                    │  │
│  │   open_folder_dialog(...)                  │  │
│  │   set_autostart(bool)                      │  │
│  │   set_shell_integration(...)               │  │
│  │   set_window_on_top(bool)   ← 无 hwnd     │  │
│  │   show_notification(...)                   │  │
│  │   request_elevation(...)                   │  │
│  │   taskbar_progress(...)                    │  │
│  │   check_ping_permission()                  │  │
│  │ }                                          │  │
│  │                                             │  │
│  │ TrayService {                              │  │
│  │   init(...)                                │  │
│  │   remove()                                 │  │
│  │   update(enabled)                          │  │
│  │ }                                          │  │
│  │                                             │  │
│  │ NetworkProvider {                          │  │
│  │   get_mac_from_arp(ip)    ← 纯平台操作     │  │
│  │   get_interfaces()                         │  │
│  │   get_local_ip()                           │  │
│  │ }                                          │  │
│  └─────────────────────────────────────────────┘  │
│                                                   │
│  internal（平台细节，外部不可见）                   │
│  ┌─────────────────────────────────────────────┐  │
│  │ #[cfg(windows)] → Win32 API                │  │
│  │ #[cfg(linux)]   → D-Bus / X11 / sudo       │  │
│  │ #[cfg(macos)]   → Cocoa / osascript        │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

### 3.3 不属于 adapter 的内容

| 功能 | 去向 | 原因 |
|------|------|------|
| `config`（加载/保存/缓存/merge） | 独立 `config` crate | 含业务逻辑（缓存策略、脏检查、merge defaults） |
| `diag` | 独立 `diag` crate 或保留 | 纯调试工具，非平台适配 |
| `network::calculate_ip_range()` | 移到 `schema` 或 `service` | 纯算法，与平台无关 |
| `ping::resolve()` | 保留在 `service` | DNS 解析是通用网络操作 |

---

## 4. 重构步骤

### 步骤 1: 将 `config` 从 adapter 提取到独立 crate

**动机**：`adapter::config` 当前包含缓存策略、脏检查、merge_defaults 等业务逻辑，不属于"平台适配"。

**操作**：
1. 新建 `crates/config/` crate
2. 从 `adapter::config` 移动：`load_config`, `save_config`, `update_config`, `get_*`, 缓存、脏检查、merge_defaults
3. `config` crate 通过 `dirs` crate 获取平台目录（不需要经过 adapter）
4. 定义 `ConfigError` 替代 `adapter::PlatformError`
5. `adapter` 删除 config 模块
6. 更新所有 `adapter::config::*` 引用 → `config::*`

**涉及文件**：
- 新建: `crates/config/Cargo.toml`, `crates/config/src/lib.rs`
- 删除: `crates/adapter/src/config.rs`, `crates/adapter/src/lib.rs` 中 config 的行
- 修改: `service/Cargo.toml`（添加强依赖, 移除adapter依赖）, `app/Cargo.toml`
- 修改: 所有 `use adapter::config::*` → `use config::*`

### 步骤 2: 净化 `adapter` 的公共 API

**动机**：当前 adapter 暴露的是零散函数，且有些接口泄漏了平台类型（如 `hwnd: usize`）。

**操作**：
1. 定义 `PlatformService` 结构体，统一封装：
   - `dialog` → `open_url()`, `open_file()`, `open_folder()`, `open_file_manager()`
   - `autostart` → `set_autostart(enabled: bool)`
   - `shell` → `set_shell_integration(enabled: bool, exe_path: &str)`
   - `taskbar` → `update_taskbar(progress, total, color)`, `clear_taskbar()`
   - `window` → `set_window_on_top(on_top: bool)`（**不再暴露 hwnd**）
   - `notification` → `show_notification(title, message)`
   - `ping` → `check_ping_permission()`, `request_elevation()`
2. 定义 `NetworkProvider` trait：
   - `get_mac_from_arp(ip) -> Option<String>`
   - `get_interfaces() -> Vec<NetworkInterface>`
   - `get_local_ip() -> Option<Ipv4Addr>`
3. 纯计算函数移出 adapter：
   - `network::calculate_ip_range()` → 移到 `schema::scan` 或 `service::scan`
4. `elevation.rs` 中创建 FLTK dialog 的逻辑考虑移到 app 层（或通过回调解耦）

### 步骤 3: 将 app 中的平台代码移到 adapter

**动机**：systray、X11 handler、tray_helper 都是平台相关代码，应集中到 adapter。

**操作**：
1. 创建 `adapter/src/tray/` 模块：
   - `adapter/src/tray/mod.rs` — 平台分派（同现有 `systray.rs`）
   - `adapter/src/tray/linux.rs` — 从 `app/systray_linux.rs` 移植
   - `adapter/src/tray/non_linux.rs` — 从 `app/systray_non_linux.rs` 移植
2. 创建 `adapter/src/tray_helper.rs` — 从 `app/tray_helper.rs` 移植
3. 创建 `adapter/src/x11_diag.rs` — 从 `app/app.rs:70-129` 移植
4. app 中的 F1/F2/F3 快捷键直接调用 `adapter::dialog::open_url()`

**依赖调整**：
- `adapter/Cargo.toml` 添加 `ksni`（Linux）、`tray-icon`（非 Linux）

### 步骤 4: service 层移除对 adapter 的依赖

**动机**：service 层是业务逻辑层，不应感知平台适配的存在。

**操作**：
1. `service/Cargo.toml` 移除 `adapter` 依赖
2. 所有 `adapter::config::*` 调用改为通过 `config` crate
3. `adapter::network::get_mac_from_arp()` 通过 `NetworkProvider` trait 注入
   - 方式：`ScanService` 构造函数接受 `Arc<dyn NetworkProvider>`
   - app 层构建时传入 `adapter::PlatformService` 作为 `NetworkProvider`
4. `adapter::PlatformError` → 移除 `#[from]`，改用 `ServiceError` 本地变体或 `anyhow`

**方案选择**：

| 方案 | 描述 | 复杂度 | 推荐度 |
|------|------|--------|--------|
| **构造注入** | service 构造函数接收配置值和 provider trait | 中 | ⭐⭐⭐⭐⭐ |
| 全局静态 | service 内部通过全局静态取配置 | 低 | ⭐⭐ |
| 回调函数 | 传入闭包读取配置 | 低 | ⭐⭐⭐ |

推荐 **构造注入**：最干净的解耦方式，且便于单元测试。

### 步骤 5: 提取 `diag` 工具

**动机**：`adapter::diag` 是纯调试工具，不是平台适配。

**操作**：
1. 新建 `crates/diag/` crate
2. 从 `adapter::diag` 移动 log 函数
3. 更新所有引用
4. 也可选择保留在现状（影响小，优先级低）

---

## 5. 关键设计决策

### 5.1 配置管理：为什么不能放 adapter

当前 `adapter::config` 包含：
- 配置缓存（`OnceLock<RwLock<AppConfig>>`）— 这是生命周期管理
- 脏检查（`LAST_SAVED_CONTENT`）— 这是 IO 优化策略
- `merge_defaults()` — 这是数据合并逻辑
- `get_*` 访问器 — 这是通用查询

这些全是**业务逻辑**，不是平台适配。平台相关的部分只是"配置文件存哪个目录"，而这个可以通过 `dirs` crate 在 config crate 中直接获取。

### 5.2 `set_window_on_top` 如何消除 `hwnd`

当前：
```rust
// adapter::window
pub fn set_window_on_top(hwnd: usize, on_top: bool) { ... }

// app 调用处
let hwnd = win.raw_handle() as usize;
adapter::window::set_window_on_top(hwnd, on_top);
```

改进后：
```rust
// adapter::PlatformService 内部通过回调/存储持有 hwnd
impl PlatformService {
    pub fn set_main_window_hwnd(&self, hwnd: usize) { /* 内部存储 */ }
    pub fn set_window_on_top(&self, on_top: bool) {
        let hwnd = self.get_stored_hwnd();
        // 平台实际调用
    }
}
```

app 在窗口创建时设置一次 hwnd，之后所有窗口操作都不需要再传 hwnd。

### 5.3 `elevation` 中 FLTK dialog 的处理

当前 `adapter::elevation` 在提权失败时直接创建 FLTK dialog：
```rust
fn show_elevation_error(msg: &str) {
    use fltk::prelude::*;
    // ...创建 FLTK 窗口
}
```

这违反了"adapter 不应包含 UI 逻辑"的原则。改进方案：
1. `PlatformService` 接受一个 `show_error: Box<dyn Fn(&str)>` 回调
2. app 层注入 FLTK 实现
3. 或：elevation 只返回错误，由 app 层决定如何展示

### 5.4 Systray 跨平台抽象

```rust
// adapter 提供的统一 TrayService
pub trait TrayProvider: Send {
    fn init(&mut self, lifecycle: Lifecycle) -> Result<(), String>;
    fn remove(&mut self);
    fn is_active(&self) -> bool;
    fn update(&mut self, enabled: bool);
}

// Linux 实现：ksni
// 非 Linux 实现：tray-icon
```

注意：`Lifecycle` 类型当前在 `app` crate 中。需要将 `Lifecycle` 的定义移到 `adapter` 或抽象为 trait。

---

## 6. 最终文件结构

### 重构后 `crates/adapter/`

```
adapter/src/
├── lib.rs              # 导出 PlatformService, TrayProvider, NetworkProvider
├── platform.rs         # PlatformService 统一入口
├── dialog.rs           # 文件对话框、URL打开
├── autostart.rs        # 开机自启
├── shell.rs            # 右键菜单集成（Windows）
├── taskbar.rs          # 任务栏进度（Windows）
├── window.rs           # 窗口操作（隐藏 hwnd）
├── notification.rs     # 系统通知
├── elevation.rs        # 提权（接口不含 UI）
├── ping.rs             # ping 权限检查
├── network.rs          # 平台网络操作（不含 calculate_ip_range）
├── tray/
│   ├── mod.rs          # 分派
│   ├── linux.rs        # ksni 实现
│   └── non_linux.rs    # tray-icon 实现
├── tray_helper.rs      # Unix IPC helper
└── x11_diag.rs         # X11 错误诊断
```

### 重构后项目结构

```
crates/
├── schema/             # 纯数据模型（无外部依赖）
├── config/             # 配置读写（无业务依赖，使用 dirs）
├── adapter/            # 纯平台适配（零业务逻辑）
├── service/            # 业务服务（无 adapter 依赖）
└── app/                # UI 层（组合所有模块）
```

---

## 7. 迁移风险与应对

| 风险 | 等级 | 应对 |
|------|------|------|
| service 改为构造注入后，需改动所有构造调用点 | 中 | 分模块逐步改，保持向后兼容 |
| systray 依赖 `Lifecycle` 类型跨 crate | 中 | 将 `Lifecycle` 定义移到 adapter 或抽象 |
| tray_helper 的 Unix socket IPC 与 FLTK 主循环耦合深 | 高 | 保留 IPC 逻辑在 adapter，FLTK 回调通过注入 |
| `elevation` 与 FLTK dialog 解耦后功能等价性 | 中 | 先提取接口，保持原实现作为默认 |
| 配置模块提取后路径兼容性 | 低 | 实际文件路径不变（仍是 `~/.config/rabbit/`） |

---

## 8. 实施优先级

```
P0 ─ 步骤 1 (config 提取) + 步骤 4 (service 解耦)
  │    核心收益：service 层不再依赖 adapter，架构分层清晰
  │
P1 ─ 步骤 2 (adapter 接口净化)
  │    收益：所有 adapter 接口不暴露平台类型
  │
P2 ─ 步骤 3 (app 平台代码移入 adapter)
  │    收益：全部平台代码集中到 adapter
  │
P3 ─ 步骤 5 (diag 提取)
      收益：低优先级，可延后
```
