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
5. **通用接口**：使用 section+map 方式，避免模块专有接口膨胀

### 架构流程

```
UI 点击按钮
    │
    ▼
┌─────────────────────────────────────────────────┐
│  1. 收集输入框值                                 │
│  2. vm.update_section("ping", updates)    │ ← 同步更新+保存
│  3. send_event(UiEvent::PingStart)            │ ← 不带参数
└─────────────────────────────────────────────────┘
    │
    ▼
业务模块 handle_event
    │
    ▼
┌─────────────────────────────────────────────────┐
│  4. vm.get_section("ping")                     │ ← 读取缓存配置
│  5. service.start(ping_config)                 │
└─────────────────────────────────────────────────┘
```

---

## ConfigValue 类型定义

在 `rabbit-models/src/config.rs` 中定义：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}
```

---

## AppViewModel 接口

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

    /// 字段更新器 - 通过闭包更新配置（复杂场景使用）
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

    /// 通过 section+map 更新配置
    pub fn update_section(&mut self, section: &str, updates: HashMap<&str, ConfigValue>) -> Result<()> {
        match section {
            "ping" => {
                let ping = &mut self.config.modules.ping;
                if let Some(v) = updates.get("target") {
                    if let Some(s) = v.as_str() { ping.target = s.to_string(); }
                }
                if let Some(v) = updates.get("interval") {
                    if let Some(n) = v.as_i64() { ping.interval = n as i32; }
                }
                // ... 其他字段
            }
            "http" => { /* ... */ }
            // ...
            _ => {}
        }
        save_config(&self.config)
    }

    /// 读取单个 section 返回 HashMap
    pub fn get_section(&self, section: &str) -> Option<HashMap<String, ConfigValue>> {
        match section {
            "ping" => { /* ... */ }
            "http" => { /* ... */ }
            // ...
            _ => None,
        }
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

---

## 字段映射表

### ping 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `target` | ping.target | String |
| `interval` | ping.interval | i32 |
| `count` | ping.count | i32 |
| `stoponloss` | ping.stoponloss | bool |
| `taskbar` | ping.taskbar | bool |
| `running` | ping.running | bool |

### http 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `port` | http.port | u16 |
| `shell` | http.shell | bool |
| `autoindex` | http.autoindex | bool |
| `videoplay` | http.videoplay | bool |
| `dirs` | http.dirs | Vec\<String\> |
| `running` | http.running | bool |

### scan 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `start_ip` | scan.start_ip | String |
| `end_ip` | scan.end_ip | String |
| `filter` | scan.filter | bool |

### tftpd 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `port` | tftpd.port | u16 |
| `timeout` | tftpd.timeout | i32 |
| `maxretry` | tftpd.maxretry | i32 |
| `blksize` | tftpd.blksize | i32 |
| `qsize` | tftpd.qsize | i32 |
| `qtout` | tftpd.qtout | i32 |
| `override_conflicts` | tftpd.override_conflicts | bool |
| `fslog` | tftpd.fslog | bool |
| `work_dirs` | tftpd.work_dirs | Vec\<String\> |
| `working_dir_index` | tftpd.working_dir_index | Option\<usize\> |
| `running` | tftpd.running | bool |

### tftpc 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `server_addr` | tftpc.server_addr | String |
| `server_port` | tftpc.server_port | u16 |
| `local_path` | tftpc.local_path | String |
| `remote_file` | tftpc.remote_file | String |
| `timeout` | tftpc.timeout | i32 |
| `maxretry` | tftpc.maxretry | i32 |
| `blksize` | tftpc.blksize | i32 |

### plan 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `date` | plan.date | String |
| `time` | plan.time | String |
| `cycle` | plan.cycle | i32 |
| `unit` | plan.unit | String |
| `msg` | plan.msg | String |
| `override_conflicts` | plan.override_conflicts | bool |

### chat 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `username` | chat.username | String |
| `port` | chat.port | u16 |
| `broadcast_addr` | chat.broadcast_addr | String |
| `running` | chat.running | bool |

### 全局配置

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `language` | language | Language |
| `theme` | theme | Theme |
| `systray` | systray | bool |
| `top` | top | bool |
| `autostart` | autostart | bool |
| `autoupdate` | autoupdate | bool |
| `last_active_tab` | last_active_tab | usize |

> **注意**：`window_*` 字段通过专门方法管理，不通过 section+map 接口。

---

## 特殊类型处理

### Vec\<String\> 数组类型

ConfigValue 的 `Array` 变体用于处理数组字段：

**update_section 中的写入**：
```rust
"http" => {
    let http = &mut self.config.modules.http;
    if let Some(v) = updates.get("dirs") {
        if let Some(arr) = v.as_array() {
            http.dirs = arr.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
}
```

**get_section 中的读取**：
```rust
"http" => {
    let http = &self.config.modules.http;
    map.insert("dirs".into(), ConfigValue::Array(
        http.dirs.iter().map(|s| ConfigValue::String(s.clone())).collect()
    ));
}
```

### Option\<usize\> 类型

tftpd.working_dir_index 是 `Option<usize>`：

```rust
"tftpd" => {
    let tftpd = &mut self.config.modules.tftpd;
    // UI 使用 1-based 索引，配置使用 0-based
    if let Some(v) = updates.get("working_dir_index") {
        if let Some(n) = v.as_i64() {
            tftpd.working_dir_index = if n <= 0 { None } else { Some((n - 1) as usize) };
        }
    }
}
```

---

## 事件处理（无参数）

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
            let config = vm.get_section("ping");
            ping_service.init(config).await;
            ping_service.start().await;
        }
        UiEvent::HttpToggle => {
            let http = vm.get_section("http").unwrap();
            let running = http.get("running").and_then(|v| v.as_bool()).unwrap_or(false);
            if running {
                http_service.stop().await;
            } else {
                http_service.start(http).await;
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
| `PingStart { target, options }` | `PingStart` | 从 vm.get_section("ping") 读取 |
| `ScanStart { start_ip, end_ip, options }` | `ScanStart` | 从 vm.get_section("scan") 读取 |
| `HttpToggle { port, options, shell }` | `HttpToggle` | 从 vm.get_section("http") 读取 |
| `ChatToggle { username, port, broadcast }` | `ChatToggle` | 从 vm.get_section("chat") 读取 |

#### B. 需要改造的事件

| 当前 | 目标 | 说明 |
|------|------|------|
| `TftpServerToggle { options }` | `TftpServerToggle` | TFTP 参数通过 sync 方法同步 |
| `PlanAdd { date, time, cycle, unit, msg }` | `PlanAdd` | 参数通过 ViewModel 保存 |

#### C. 保持原样的事件

| 事件 | 原因 |
|------|------|
| `TftpServerAddDir` | 需要目录选择 UI |
| `TftpServerRemoveDir` | 需要目录选择 UI |
| `TftpClientPut { server, local, remote }` | 每次传输不同参数 |
| `TftpClientGet { server, local, remote }` | 每次传输不同参数 |
| `PlanRemove { id }` | 需要事件 ID |
| `ChatSend { message }` | 消息内容每次不同 |
| `SettingsSave` | **待讨论**：推荐移除 |

---

## 实施计划

### 阶段 1：基础接口 + ConfigValue 定义 ✅

- [x] `update_and_save()` 方法
- [x] `update_with()` 通用更新器
- [x] `update_global()` 方法
- [x] `update_last_tab()` 方法
- [x] 定义 ConfigValue 枚举（String, Integer, Boolean, Array）
- [x] 实现 as_str/as_i64/as_bool/as_array 辅助方法
- [x] 验证序列化/反序列化

### 阶段 2：通用 section+map 方法 ✅

- [x] 实现 `update_section()` 字段映射（7个模块）
- [x] 实现 `get_section()` 读取（7个模块）
- [x] 实现 `get_value()` 单值读取
- [x] 编译验证

### 阶段 3：移除 ui_state.rs 分散保存 ✅

- [x] ui_state.rs 方法改为发送事件（SettingsUpdate/ModuleUpdate）
- [x] app.rs 统一处理配置更新
- [x] 10 个方法改造完成

### 阶段 4：简化事件参数

#### A. 改造为无参数的事件

- [ ] 改造 `PingStart` 事件 → 无参数
- [ ] 改造 `ScanStart` 事件 → 无参数
- [ ] 改造 `HttpToggle` 事件 → 无参数
- [ ] 改造 `ChatToggle` 事件 → 无参数

#### B. 改造事件（参数通过 ViewModel 保存）

- [ ] 改造 `TftpServerToggle` 事件 → 无参数
- [ ] 改造 `PlanAdd` 事件 → 无参数

### 阶段 5：验证与测试

- [ ] 编译通过
- [ ] 单元测试通过
- [ ] 集成测试通过

---

## 相关文档

- [config-impl-plan.md](./config-impl-plan.md) - 详细实施计划
- [config-issues.md](./config-issues.md) - 遇到的问题记录

---

文档版本：2.0
更新日期：2026-04-23
