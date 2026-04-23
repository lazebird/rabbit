# 配置优化详细实施计划

## 概述

本文档详细描述配置系统优化的具体实施步骤，按阶段划分，每个阶段包含具体任务和验收标准。

### 方案选择历史

| 方案 | 说明 | 选择 |
|------|------|------|
| 方案 A | HashMap 替换配置模型 | ❌ 破坏性大 |
| 方案 B | 混合模式（struct + 运行时状态） | ❌ 需修改配置模型 |
| **方案 C** | ViewModel 调整（最小改动） | ✅ 采用 |

---

## 任务清单汇总

### 阶段 1：ConfigValue 定义

- [ ] 定义 ConfigValue 枚举（String, Integer, Boolean, Array）
- [ ] 实现辅助方法（as_str, as_i64, as_bool, as_array）
- [ ] 导出 ConfigValue
- [ ] 验证序列化/反序列化

### 阶段 2：section+map 方法

- [ ] 定义 ConfigValue 枚举
- [ ] 添加 as_array 辅助方法
- [ ] 实现 ping.update_section（6字段）
- [ ] 实现 http.update_section（6字段 + dirs数组）
- [ ] 实现 scan.update_section（3字段）
- [ ] 实现 tftpd.update_section（11字段 + work_dirs数组）
- [ ] 实现 tftpc.update_section（7字段）
- [ ] 实现 plan.update_section（6字段）
- [ ] 实现 chat.update_section（4字段）
- [ ] 实现 get_section（所有模块）
- [ ] 实现 get_value
- [ ] 编译验证

### 阶段 3：统一保存逻辑

- [ ] 改造 set_systray
- [ ] 改造 set_top
- [ ] 改造 set_autostart
- [ ] 改造 set_autoupdate
- [ ] 改造 set_language
- [ ] 改造 sync_http_config
- [ ] 改造 sync_tftpd_config
- [ ] 改造 sync_plan_config
- [ ] 改造 sync_scan_config
- [ ] 改造 sync_http_start_config

### 阶段 4：简化事件参数

#### A. 改造为无参数的事件
- [ ] 改造 PingStart
- [ ] 改造 ScanStart
- [ ] 改造 HttpToggle
- [ ] 改造 ChatToggle

#### B. 改造事件（参数通过 ViewModel 保存）
- [ ] 改造 TftpServerToggle
- [ ] 改造 PlanAdd

#### C. 其他事件改造
- [ ] 修改 TftpServerAddDir 事件处理
- [ ] 修改 TftpServerRemoveDir 事件处理

### 阶段 5：验证测试
- [ ] 编译通过
- [ ] 单元测试通过
- [ ] 集成测试通过

---

## 详细步骤

## 阶段 1：定义 ConfigValue 类型

**目标**：在 rabbit-models 中定义 ConfigValue 类型，支持 section+map 方式更新配置

### 1.1 添加 ConfigValue 枚举

**文件**：`rabbit-models/src/config.rs`

**任务**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}
```

**辅助方法**：
```rust
impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> { ... }
    pub fn as_i64(&self) -> Option<i64> { ... }
    pub fn as_bool(&self) -> Option<bool> { ... }
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> { ... }
}
```

**验收标准**：
- [ ] ConfigValue 可序列化/反序列化为 JSON
- [ ] 类型转换方法正确工作

### 1.2 导出 ConfigValue

**文件**：`rabbit-models/src/lib.rs`

**任务**：确保 `pub use config::*;` 导出 ConfigValue

**验收标准**：
- [ ] `use rabbit_models::config::ConfigValue;` 编译通过

---

## 阶段 2：实现 section+map 方法

**目标**：在 view_model.rs 中实现通用的 update_section/get_section/get_value 方法

### 2.1 实现 update_section

**文件**：`crates/rabbit-app/src/view_model.rs`

**任务**：为 7 个模块实现字段映射

| 模块 | 字段数 | 状态 |
|------|--------|------|
| ping | 6 | 待实现 |
| http | 6 | 待实现 |
| scan | 3 | 待实现 |
| tftpd | 11 | 待实现 |
| tftpc | 7 | 待实现 |
| plan | 6 | 待实现 |
| chat | 4 | 待实现 |

**示例**：
```rust
"ping" => {
    let ping = &mut self.config.modules.ping;
    if let Some(v) = updates.get("target") {
        if let Some(s) = v.as_str() { ping.target = s.to_string(); }
    }
    if let Some(v) = updates.get("interval") {
        if let Some(n) = v.as_i64() { ping.interval = n as i32; }
    }
    if let Some(v) = updates.get("count") {
        if let Some(n) = v.as_i64() { ping.count = n as i32; }
    }
    if let Some(v) = updates.get("stoponloss") {
        if let Some(b) = v.as_bool() { ping.stoponloss = b; }
    }
    if let Some(v) = updates.get("taskbar") {
        if let Some(b) = v.as_bool() { ping.taskbar = b; }
    }
    if let Some(v) = updates.get("running") {
        if let Some(b) = v.as_bool() { ping.running = b; }
    }
}
```

**验收标准**：
- [ ] 所有 7 个模块的 update_section 实现完成
- [ ] 编译通过

### 2.2 实现 get_section

**任务**：为 7 个模块实现读取

**验收标准**：
- [ ] 所有 7 个模块的 get_section 实现完成
- [ ] 编译通过

### 2.3 实现 get_value

**验收标准**：
- [ ] 单值读取正确工作

### 2.4 清理现有代码

**任务**：修复 view_model.rs 中的 ConfigValue 引用错误

**当前错误**：
```
error[E0432]: unresolved import `rabbit_models::config::ConfigValue`
 --> crates/rabbit-app/src/view_model.rs:1:40
```

**验收标准**：
- [ ] `cargo build` 编译通过
- [ ] 无 ConfigValue 相关错误

---

## 阶段 3：统一 ui_state.rs 保存逻辑

**目标**：将 ui_state.rs 中的分散 load+save 改为调用 ViewModel

### 3.1 改造全局设置方法

| 方法 | 当前 | 目标 |
|------|------|------|
| `set_systray(value)` | load+save | 调用 `vm.update_global()` |
| `set_top(value)` | load+save | 调用 `vm.update_global()` |
| `set_autostart(value)` | load+save | 调用 `vm.update_global()` |
| `set_autoupdate(value)` | load+save | 调用 `vm.update_global()` |
| `set_language(value)` | load+save | 调用 `vm.update_global()` |

**挑战**：
- ui_state.rs 是静态全局方法
- 需要传递 ViewModel 引用或使用回调

**推荐方案**：改造为接受 ViewModel 引用
```rust
pub fn set_systray(vm: &mut AppViewModel, value: bool) -> Result<()> {
    vm.update_global(
        vm.config.language,
        vm.config.theme,
        value,
        vm.config.top,
        vm.config.autostart,
        vm.config.autoupdate,
    )
}
```

### 3.2 改造同步方法

| 方法 | 当前 | 目标 |
|------|------|------|
| `sync_http_config()` | load+save | 调用 `vm.update_section("http", ...)` |
| `sync_tftpd_config()` | load+save | 调用 `vm.update_section("tftpd", ...)` |
| `sync_plan_config()` | load+save | 调用 `vm.update_section("plan", ...)` |
| `sync_scan_config()` | load+save | 调用 `vm.update_section("scan", ...)` |
| `sync_http_start_config()` | load+save | 调用 `vm.update_section("http", ...)` |

### 3.3 验收标准

- [ ] 所有 10 个方法改造完成
- [ ] 配置保存行为不变
- [ ] 编译通过

---

## 阶段 4：简化 UiEvent 参数

**目标**：将带参数的事件改为无参数，配置从 ViewModel 读取

### 4.1 事件改造列表

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

### 4.2 改造步骤

1. 修改 `ui_events.rs` 中的事件定义
2. 修改事件发送方（UI 回调）
3. 修改事件处理方（app.rs 或各模块）

### 4.3 示例改造

**Before**：
```rust
// UI
send_event(UiEvent::PingStart {
    target: input.value(),
    options: "interval=1000".into(),
});

// Handler
async fn handle_event(event: UiEvent) {
    match event {
        UiEvent::PingStart { target, options } => {
            ping_service.start(target, options).await;
        }
    }
}
```

**After**：
```rust
// UI
vm.update_section("ping", updates)?;
send_event(UiEvent::PingStart);

// Handler
async fn handle_event(event: UiEvent, vm: &AppViewModel) {
    match event {
        UiEvent::PingStart => {
            let ping_config = vm.get_section("ping").unwrap();
            ping_service.start(ping_config).await;
        }
    }
}
```

### 4.4 验收标准

- [ ] 所有 4 个事件改造完成
- [ ] 编译通过
- [ ] 功能验证正常

---

## 阶段 5：验证与测试

### 5.1 编译检查

```bash
cargo build --release
```

**验收标准**：无编译错误

### 5.2 单元测试

```bash
cargo test
```

**验收标准**：
- [ ] 现有测试通过
- [ ] 新增 ConfigValue 相关测试

### 5.3 集成测试

**测试场景**：
1. 修改设置 → 关闭应用 → 重新打开 → 设置保持
2. 修改模块配置 → 关闭应用 → 重新打开 → 配置保持
3. 启动服务 → 关闭应用 → 重新打开 → running 状态正确恢复

---

## 相关文档

- [config-structure.md](./config-structure.md) - 整体架构设计
- [config-issues.md](./config-issues.md) - 遇到的问题记录

---

文档版本：2.0
更新日期：2026-04-23