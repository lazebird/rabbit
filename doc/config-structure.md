# Rabbit 配置数据结构优化方案

## 问题背景

当前配置系统存在以下问题：

1. **配置保存分散**：ui_state.rs、app.rs、view_model.rs 多处独立 load+save
2. **时序问题**：配置更新和事件发送可能不同步
3. **内存缓存缺失**：每次读取都从文件加载
4. **统一入口缺失**：缺少集中的配置管理接口

---

## 统一配置管理方案

### 设计原则

1. **单一数据源**：AppConfig 是唯一内存数据源
2. **同步保存**：先保存到磁盘，再发送事件，无时序问题
3. **内存缓存**：配置常驻内存，避免频繁读文件
4. **统一入口**：ViewModel 负责所有配置操作

### 架构流程

```
UI 点击按钮
    │
    ▼
┌─────────────────────────────────────────────────┐
│  1. 收集输入框值                                 │
│  2. vm.update_and_save(config)             │ ← 同步更新+保存
│  3. send_event(UiEvent::ModuleStart)             │ ← 不带参数
└─────────────────────────────────────────────────┘
    │
    ▼
业务模块 handle_event
    │
    ▼
┌─────────────────────────────────────────────────┐
│  4. vm.get_config()                              │ ← 读取缓存配置
│  5. service.start(config.modules.ping)          │
└─────────────────────────────────────────────────┘
```

### AppViewModel 接口

```rust
pub struct AppViewModel {
    config: AppConfig,                    // 内存缓存
    ping_results: HashMap<String, PingSummary>,  // 运行时数据
}

impl AppViewModel {
    pub fn new(config: AppConfig) -> Self {
        Self { config, ping_results: HashMap::new() }
    }

    // === 基础接口 ===

    pub fn get_config(&self) -> AppConfig {
        self.config.clone()
    }

    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }

    pub fn save_settings(&mut self) -> Result<()> {
        save_config(&self.config)
    }

    // === 通用更新器 ===

    /// 字段更新器 - 通过闭包更新配置
    pub fn update_with<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        updater(&mut self.config);
        save_config(&self.config)
    }

    // === 专用方法 ===

    /// 更新全局设置
    pub fn update_global(&mut self, language, theme, systray, top, autostart, autoupdate) -> Result<()> {
        self.config.language = language;
        self.config.theme = theme;
        self.config.systray = systray;
        self.config.top = top;
        self.config.autostart = autostart;
        self.config.autoupdate = autoupdate;
        save_config(&self.config)
    }

    /// 更新最后活动标签页
    pub fn update_last_tab(&mut self, tab: usize) -> Result<()> {
        self.config.last_active_tab = tab;
        save_config(&self.config)
    }

    // === 通用 section+map 方法 ===

    /// 通过 section+map 更新配置（避免模块专有接口）
    pub fn update_section(&mut self, section: &str, updates: HashMap<&str, ConfigValue>) -> Result<()> {
        // 实现见 config-partial-update.md
        match section {
            "ping" => { /* 字段映射 */ }
            "http" => { /* 字段映射 */ }
            // ...
            _ => {}
        }
        save_config(&self.config)
    }

    /// 读取单个 section 返回 HashMap
    pub fn get_section(&self, section: &str) -> Option<HashMap<String, ConfigValue>> {
        // 实现见 config-partial-update.md
    }

    /// 读取单个值
    pub fn get_value(&self, section: &str, key: &str) -> Option<ConfigValue> {
        self.get_section(section)?.get(key).cloned()
    }

    // === 运行时数据（不持久化）===

    pub fn get_ping_result(&self, target: &str) -> Option<&PingSummary> {
        self.ping_results.get(target)
    }

    pub fn update_ping_result(&mut self, result: PingSummary) {
        self.ping_results.insert(result.target.clone(), result);
    }
}
```

### 事件处理（无参数）

```rust
pub enum UiEvent {
    PingStart,
    PingStop,
    HttpToggle,
    HttpStart,
    HttpStop,
    TftpStart,
    TftpStop,
    ChatStart,
    ChatStop,
    ScanStart,
    ScanStop,
}

async fn handle_event(event: UiEvent, vm: &AppViewModel) {
    match event {
        UiEvent::PingStart => {
            let config = vm.get_config();
            ping_service.init(config.modules.ping.clone()).await;
            ping_service.start().await;
        }
        UiEvent::HttpToggle => {
            let config = vm.get_config();
            if config.modules.http.running {
                http_service.stop().await;
            } else {
                http_service.init(config.modules.http.clone()).await;
                http_service.start().await;
            }
        }
        // ...
    }
}
```

---

## 优点

| 方面 | 说明 |
|------|------|
| **无时序问题** | 同步保存完成后再发送事件 |
| **单一入口** | 只通过 ViewModel 保存 |
| **一致性** | 业务模块总是读取最新配置 |
| **性能** | 内存缓存避免频繁读文件 |
| **代码简化** | 移除分散的 load+save 逻辑 |
| **通用化** | section+map 避免模块专有接口膨胀 |

---

## 需要改造的点

### ui_state.rs 分散的保存点（需统一到 ViewModel）

| 方法 | 当前行为 | 目标行为 |
|------|----------|----------|
| `set_systray(value)` | load+save | 调用 `vm.update_global()` |
| `set_top(value)` | load+save | 调用 `vm.update_global()` |
| `set_autostart(value)` | load+save | 调用 `vm.update_global()` |
| `set_autoupdate(value)` | load+save | 调用 `vm.update_global()` |
| `set_language(value)` | load+save | 调用 `vm.update_global()` |
| `sync_http_config()` | load+save | 调用 `vm.update_section("http", ...)` |
| `sync_tftpd_config()` | load+save | 调用 `vm.update_section("tftpd", ...)` |
| `sync_plan_config()` | load+save | 调用 `vm.update_section("plan", ...)` |
| `sync_scan_config()` | load+save | 调用 `vm.update_section("scan", ...)` |
| `sync_http_start_config()` | load+save | 调用 `vm.update_section("http", ...)` |

### UiEvent 事件参数（需简化）

#### A. 需要改造为无参数的事件

| 当前 | 目标 | 说明 |
|------|------|------|
| `PingStart { target, options }` | `PingStart` | 从 vm.get_config() 读取 |
| `ScanStart { start_ip, end_ip, options }` | `ScanStart` | 从 vm.get_config() 读取 |
| `HttpToggle { port, options, shell }` | `HttpToggle` | 从 vm.get_config() 读取 |
| `ChatToggle { username, port, broadcast }` | `ChatToggle` | 从 vm.get_config() 读取 |

#### B. 需要改造的事件（带参数但不同步配置）

| 当前 | 目标 | 说明 |
|------|------|------|
| `TftpServerToggle { options }` | `TftpServerToggle` | TFTP 参数通过 sync 方法同步 |
| `PlanAdd { date, time, cycle, unit, msg }` | `PlanAdd` | 参数直接通过 ViewModel 保存 |

#### C. 保持原样的事件

| 事件 | 原因 |
|------|------|
| `TftpServerAddDir` | 需要目录选择 UI |
| `TftpServerRemoveDir` | 需要目录选择 UI |
| `TftpClientPut { server, local, remote }` | 每次传输不同参数 |
| `TftpClientGet { server, local, remote }` | 每次传输不同参数 |
| `PlanRemove { id }` | 需要事件 ID |
| `ChatSend { message }` | 消息内容每次不同 |
| `SettingsSave` | **待讨论**：是否需要，或改为无操作 |

#### D. SettingsSave 事件说明

当前 `SettingsSave` 事件的作用：
1. 从磁盘重新加载配置（同步 ui_state.rs 中的修改）
2. 更新 ViewModel
3. 应用各项设置（autostart, systray, top 等）

**新设计中的变化**：
- ui_state.rs 的 sync_* 方法改为调用 ViewModel
- ViewModel 保存配置是同步的
- 事件处理程序直接从 `vm.get_config()` 读取

**可能的改造**：
```rust
// 方案 A：移除 SettingsSave（推荐）
// UI 回调中已通过 ViewModel 保存，无需额外事件

// 方案 B：保留但简化
UiEvent::SettingsSave => {
    // 直接从 ViewModel 获取配置（已是最新的）
    let config = vm.get_config();
    apply_settings(config);
}
```

---

## 实施计划

### 阶段 1：基础接口（已完成）

- [x] `update_and_save()` 方法
- [x] `update_with()` 通用更新器
- [x] `update_global()` 方法
- [x] `update_last_tab()` 方法

### 阶段 2：通用 section+map 方法

- [ ] 定义 ConfigValue 类型（见 config-partial-update.md）
- [ ] 实现 `update_section()` 字段映射（包含 running 状态）
- [ ] 实现 `get_section()` 读取（包含 running 状态）
- [ ] 实现 `get_value()` 单值读取
- [ ] 编译验证

### 阶段 3：移除 ui_state.rs 分散保存

- [ ] 改造 `set_systray()` → 调用 ViewModel
- [ ] 改造 `set_top()` → 调用 ViewModel
- [ ] 改造 `set_autostart()` → 调用 ViewModel
- [ ] 改造 `set_autoupdate()` → 调用 ViewModel
- [ ] 改造 `set_language()` → 调用 ViewModel
- [ ] 改造 `sync_http_config()` → 调用 ViewModel
- [ ] 改造 `sync_tftpd_config()` → 调用 ViewModel
- [ ] 改造 `sync_plan_config()` → 调用 ViewModel
- [ ] 改造 `sync_scan_config()` → 调用 ViewModel
- [ ] 改造 `sync_http_start_config()` → 调用 ViewModel

### 阶段 4：简化事件参数

#### A. 改造为无参数的事件（从 vm.get_config() 读取）

- [ ] 改造 `PingStart` 事件 → 无参数
- [ ] 改造 `ScanStart` 事件 → 无参数
- [ ] 改造 `HttpToggle` 事件 → 无参数
- [ ] 改造 `ChatToggle` 事件 → 无参数

#### B. 改造事件（参数通过 ViewModel 保存）

- [ ] 改造 `TftpServerToggle` 事件 → 无参数
- [ ] 改造 `PlanAdd` 事件 → 无参数（参数通过 ui_state 同步）

#### C. 保持原样的事件

- `TftpServerAddDir` - 需要目录选择 UI
- `TftpServerRemoveDir` - 需要目录选择 UI
- `TftpClientPut/Get` - 每次传输不同参数
- `PlanRemove { id }` - 需要事件 ID
- `ChatSend { message }` - 消息内容每次不同

---

## 相关文档

- [config-partial-update.md](./config-partial-update.md) - 局部更新方案
- [config-impl-plan.md](./config-impl-plan.md) - 详细实施计划
- [config-issues.md](./config-issues.md) - 遇到的问题记录

---

文档版本：1.1
更新日期：2026-04-23
