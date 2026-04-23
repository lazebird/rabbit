# 配置优化遇到的问题记录

本文档记录配置优化过程中遇到的问题及解决方案。

---

## 问题 1：ConfigValue 类型未定义

### 状态：✅ 已解决

在 `rabbit-models/src/config.rs` 中添加 ConfigValue 枚举定义：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> { ... }
    pub fn as_i64(&self) -> Option<i64> { ... }
    pub fn as_bool(&self) -> Option<bool> { ... }
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> { ... }
}
```

---

## 问题 2：ui_state.rs 分散的 load+save

### 状态：✅ 已解决

使用事件通知模式：
- `SettingsUpdate { field, value }` - 全局设置更新
- `ModuleUpdate { module, updates }` - 模块配置更新

---

## 问题 3：UiEvent 事件带参数

### 问题描述

当前所有 UiEvent 都带参数，与 config-structure.md 中设计的"无参数事件"不符。

### 当前设计

```rust
pub enum UiEvent {
    PingStart { target: String, options: String },
    ScanStart { start_ip: String, end_ip: String, options: String },
    HttpToggle { port: u16, options: String, shell: bool },
    ChatToggle { username: String, port: u16, broadcast: String },
    // ...
}
```

### 目标设计

```rust
pub enum UiEvent {
    PingStart,
    ScanStart,
    HttpToggle,
    ChatToggle,
    // ...
}
```

### 挑战

1. **迁移成本**：需要修改所有事件发送方和接收方
2. **向后兼容**：需要考虑渐进式迁移
3. **配置时机**：配置更新和事件发送的顺序

### 解决方案

1. 先实现 ViewModel 的 update_section 方法
2. UI 回调中：先调用 `vm.update_section()`，再发送事件
3. 事件处理器：从 `vm.get_config()` 读取最新配置

### 状态

**待修复** - 计划在阶段 4 完成

---

## 问题 4：section+map 字段映射维护

### 问题描述

使用 section+map 方式需要维护字段映射表，新增或修改字段时需要同步更新。

### 当前实现

```rust
"ping" => {
    let ping = &mut self.config.modules.ping;
    if let Some(v) = updates.get("target") {
        if let Some(s) = v.as_str() { ping.target = s.to_string(); }
    }
    if let Some(v) = updates.get("interval") {
        if let Some(n) = v.as_i64() { ping.interval = n as i32; }
    }
    // ... 更多字段
}
```

### 问题

1. 新增字段需要手动添加映射
2. 字段类型变更需要修改映射逻辑
3. 字符串 key 容易拼写错误

### 解决方案

1. **文档化字段映射**：在 config-partial-update.md 中维护完整的字段表
2. **编译时检查**：考虑使用宏自动生成映射（可选优化）
3. **测试覆盖**：添加单元测试验证映射正确性

### 状态

**设计中** - 通过文档化解决

---

## 问题 5：避免冗余接口

### 设计原则

运行状态（running）是业务配置的一部分，应该：
- **更新**：通过 `update_section("ping", {"running": true})` 一起更新
- **读取**：通过 `get_section("ping")["running"]` 一起读取

而非单独提供 `is_running()` / `set_running()` 等冗余接口。

### 正确示例

```rust
// 更新配置（含运行状态）
vm.update_section("http", HashMap::from([
    ("port", ConfigValue::Integer(8000)),
    ("running", ConfigValue::Boolean(true)),
]))?;

// 读取配置（含运行状态）
let http_section = vm.get_section("http").unwrap();
let running = http_section.get("running").and_then(|v| v.as_bool());

// 设置运行状态（作为配置的一部分）
vm.update_section("ping", HashMap::from([
    ("running", ConfigValue::Boolean(true)),
]))?;
```

### 状态

**设计原则** - 通过文档化接口统一处理

---

## 问题 6：模块专有接口 vs 通用接口

### 问题描述

是否需要保留模块专有方法（如 update_ping, update_http）？

### 当前 view_model.rs

```rust
// 已有专用方法
pub fn update_global(...) -> Result<()>
pub fn update_last_tab(...) -> Result<()>

// 已有通用方法
pub fn update_with<F>(updater: F) -> Result<()>
```

### 方案选择

**推荐方案**：以通用 section+map 为主，专用方法为辅

1. **通用方法**：section+map 用于 UI 层的标准化调用
2. **update_with**：用于复杂场景或批量更新
3. **专用方法**：可保留用于特殊场景，不强求移除

### 状态

**设计中** - 通用方法为主

---

## 相关文档

- [config-structure.md](./config-structure.md) - 整体架构设计
- [config-partial-update.md](./config-partial-update.md) - 局部更新方案
- [config-impl-plan.md](./config-impl-plan.md) - 详细实施计划

---

文档版本：1.0
创建日期：2026-04-23