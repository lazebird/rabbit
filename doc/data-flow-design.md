# Rabbit 数据流与状态同步设计文档

文档版本: 2.2  
最后更新: 2026-04-30  
状态: ✅ 优化完成（完全事件驱动，无轮询）

---

## 1. 当前实现状态

### 1.1 已完成的核心机制

已实现基于 **Channel + awake_callback 的完全事件驱动架构**，彻底淘汰轮询机制：

| 模块 | 状态 | 实现方式 |
|------|------|----------|
| **Ping** | ✅ 完成 | Channel 推送 + `ServiceStatus` 通知 + `awake_callback` 事件驱动 |
| **Scan** | ✅ 完成 | Channel 推送 + `ServiceStatus` 通知 + `awake_callback` 事件驱动 |
| **HTTP** | ✅ 完成 | Channel 推送日志 + `awake_callback` 刷新 |
| **TFTP** | ✅ 完成 | Channel 推送日志 + `awake_callback` 刷新 |
| **Chat** | ✅ 完成 | Channel 推送消息 + `ChatUserList(Vec<String>)` + `awake_callback` 刷新 |
| **Plan** | ✅ 完成 | Channel 推送提醒 + `PlanReminder` 事件驱动 |

### 1.2 核心架构（已实现）

```
┌─────────────┐          ┌─────────────┐
│  Service    │───UiData──▶│  app.rs     │
│  (core)    │   (channel) │  (handle_ui_data)
└─────────────┘          └──────┬──────┘
                                 │
                    ┌───────────┴───────────┐
                    │  ui_state (全局状态)    │
                    └───────────┬───────────┘
                                 │
                    ┌───────────┴───────────┐
                    │  awake_callback (事件驱动) │
                    │  → update_button_state()   │
                    │  → refresh_displays()      │
                    │  → update_window_title()   │
                    └───────────────────────┘
```

**关键改进**：
- Service 不再直接调用 `ui_state`（线程安全）
- UI 更新完全事件驱动（无轮询，零 CPU 空闲消耗）
- 业务状态通过 `UiData::ServiceStatus(Module, bool, Option<String>)` 主动通知
- **窗口标题更新**：`awake_callback` 事件驱动 ✅
- **按钮状态更新**：`awake_callback` + `update_button_state()` 事件驱动 ✅
- **文本框内容更新**：`awake_callback` + `refresh_displays()` 事件驱动 ✅
- **轮询机制**：已完全删除 ✅

**性能提升**：从 100ms 轮询（~100% CPU 忙等待）到完全事件驱动（空闲时 0% CPU）

---

## 2. UiData 权威定义（唯一版本）

> **源码位置**: `rabbit-models/src/lib.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiData {
    // ═══ 通用日志（HTTP/TFTP/Chat/Ping/Scan） ═══
    Log(Module, String),  // (模块, 格式化好的文本)

    // ═══ Ping 专用 ═══
    PingStats(String),        // "Tx 10 Rx 9 Loss 10% Min 1ms Max 5ms Avg 2ms"
    PingState {
        address: String,       // 当前 ping 地址
        progress: u32,        // 已发送次数
        total: u32,           // 总次数（count 或无限大）
        color: String,         // "green" / "red" 用于任务栏
    },

    // ═══ Scan 专用 ═══
    ScanProgress(String),     // "Progress: 50% - Found 5 hosts"

    // ═══ Plan 专用 ═══
    PlanReminder(String),     // 计划到期提醒消息

    // ═══ Chat 专用 ═══
    ChatMessage(String, String),      // (用户名, 消息)
    ChatUserList(Vec<String>),       // 数组形式，避免逗号分隔问题

    // ═══ 服务状态更新（核心：业务状态通知） ═══
    // 第三个参数：None = 普通停止，Some(reason) = 带原因
    ServiceStatus(Module, bool, Option<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}
```

**设计原则**：
1. **显示数据**：Service 直接格式化为 `String`，通过 `Log` 或专用变体发送
2. **业务状态**：通过 `ServiceStatus` 统一通知（启动/停止/异常）
3. **最小化**：不在 `UiData` 中暴露内部业务模型（`PingResult`、`ScanResult` 等已私有化）

---

## 3. 业务状态更新流程

### 3.1 Ping 自动停止（count 达到）

```rust
// rabbit-core/src/ping.rs
// 场景：count > 0 且已发送次数达到 count
if target.count > 0 && current_sent >= target.count {
    // 1. 停止服务
    *service.state.write().await = PingState::Idle;
    
    // 2. 发送状态通知（让 UI 更新按钮和窗口标题）
    if let Some(ref ui_tx) = service.tx {
        // 停止原因：count 达到自动停止
        let reason = Some("count reached".into());
        let _ = ui_tx.send(UiData::ServiceStatus(Module::Ping, false, reason)).await;
    }
    
    // 3. 发送完成日志
    let _ = service.send(UiData::Log(
        Module::Ping, 
        format!("Ping completed: {} packets sent", target.count)
    )).await;
}
```

**UI 端处理**（app.rs:1096-1120）：
```rust
UiData::ServiceStatus(module, running, reason) => {
    // 1. 更新状态
    match module {
        Module::Ping => ui_state::set_ping_running(running),
        Module::Scan => ui_state::set_scan_running(running),
        _ => {}
    }
    
    // 2. 通过 awake_callback 直接更新 UI（在主线程执行）
    fltk::app::awake_callback(move || {
        // 更新窗口标题
        update_window_title(module, running, reason);
        // 更新按钮状态（标签/颜色）
        update_button_state(module, running);
    });
}
```

### 3.2 Scan 完成自动停止

```rust
// rabbit-core/src/scan.rs
// 场景：所有 IP 扫描完成
*state.write().await = ScannerState::Completed;

if let Some(ref tx) = self.tx {
    // 1. 发送状态通知（让按钮从 "Stop" → "Start"）
    let reason = Some("completed".into());
    let _ = tx.send(UiData::ServiceStatus(Module::Scan, false, reason)).await;
    
    // 2. 发送完成日志
    let _ = tx.send(UiData::Log(Module::Scan, "Scan finished.".to_string())).await;
}
```

### 3.3 手动停止（用户点击按钮）

```rust
// rabbit-app/src/ui/ping_tab.rs:108-175
start_btn.set_callback(move |_| {
    let label = start_btn_clone.label();
    if label == "Start" {
        // 启动：发送 ModuleToggle 事件
        send_event(UiEvent::ModuleToggle { module: "ping".into() });
        start_btn.set_label("Stop");  // 立即反馈
    } else {
        // 停止：发送 ModuleToggle 事件
        send_event(UiEvent::ModuleToggle { module: "ping".into() });
        start_btn.set_label("Start");
    }
});

// app.rs 处理 ModuleToggle
// → 调用 service.update() 
// → 停止时发送 ServiceStatus(Module, false, None)  // None = 用户手动停止
```

### 3.4 状态更新时序图

```
┌─────────────┐    ServiceStatus    ┌─────────────┐
│ Service   │════════════════▶│  app.rs    │
│ (core)   │   (Module, bool,  │ (handle_ui│
│           │    Option<String>)│ _data)    │
└─────────────┘                  └─────┬─────┘
                                       │
                          ┌────────────┴────────────┐
                          │  awake_callback (事件驱动) │
                          │  → update_button_state()   │
                          │  → update_window_title()   │
                          └───────────────────────┘
```

### 3.5 文本框内容更新流程（事件驱动）

```rust
// app.rs handle_ui_data 中：
UiData::Log(module, msg) => {
    match module {
        Module::Ping => crate::ui_state::append_ping_output(&msg),
        Module::Http => crate::ui_state::append_http_log(&msg),
        // ... 其他模块
    }
    // 事件驱动：直接调用 refresh_displays()
    fltk::app::awake_callback(|| {
        crate::ui::ui_refresh::refresh_displays();
    });
}
```

---

## 4. 各模块使用方式

### 4.1 PingService

```rust
// 发送日志
let _ = self.send(UiData::Log(
    Module::Ping, 
    format!("Reply from {}: time={:.1}ms TTL={}", target, rtt_ms, ttl)
)).await;

// 发送统计
let _ = self.send(UiData::PingStats(stats_string)).await;

// 发送状态更新（停止时，带原因）
let reason = Some("count reached".into());
let _ = self.tx.send(UiData::ServiceStatus(Module::Ping, false, reason)).await;
```

### 4.2 ScanService

```rust
// 发现主机时发送日志
let _ = tx.send(UiData::Log(
    Module::Scan, 
    format!("Found online host: {} ({})", ip, hostname)
)).await;

// 完成时发送状态更新（带原因）
let reason = Some("completed".into());
let _ = tx.send(UiData::ServiceStatus(Module::Scan, false, reason)).await;
```

### 4.3 HttpService

```rust
// 请求日志（格式化好的字符串）
let text = format!("{} {} {} - {}", timestamp, method, path, status);
let _ = self.send(UiData::Log(Module::Http, text)).await;
```

---

## 5. 已完成优化工作

### 5.1 ✅ 淘汰 ui_refresh 轮询机制（已完成）

**优化前**：`ui_refresh.rs` 使用 100ms 轮询检查 `updated` 标志，CPU 占用率高。

**优化后**：完全事件驱动，零轮询。

| 更新类型 | 实现方式 | 状态 |
|----------|----------|------|
| 窗口标题更新 | `fltk::app::awake_callback` | ✅ 无轮询 |
| 按钮状态更新 | `awake_callback` + `update_button_state()` | ✅ 无轮询 |
| 文本框内容更新 | `awake_callback` + `refresh_displays()` | ✅ 无轮询 |

**迁移步骤**（已完成）：
1. ✅ 已将窗口标题更新改为 `awake_callback`
2. ✅ 已将按钮状态更新改为 `awake_callback`（通过 `update_button_state()` 函数）
3. ✅ 已将文本框内容更新改为事件驱动（通过 `refresh_displays()`）
4. ✅ 已完全删除 `ui_refresh` 轮询逻辑（`start_refresh_loop`、`do_refresh`、`REFRESH_RUNNING` 等已删除）

**代码修订完成**：
- ✅ 修改 `app.rs`：在 `handle_ui_data` 中使用 `awake_callback` 直接更新 UI
- ✅ 修改 `ui_refresh.rs`：删除轮询逻辑，保留 `update_button_state()` 和 `refresh_displays()`
- ✅ 测试验证：所有测试通过，UI 响应及时，空闲时 0% CPU

### 5.2 ✅ UiData 接口优化（已完成）

#### 5.2.1 ChatUserList 从逗号分隔改为数组

**优化前**：`ChatUserList(String)` - 用逗号分隔用户名  
**问题**：用户名可能包含逗号，解析困难且不安全

**优化后**：`ChatUserList(Vec<String>)` - 直接传递数组

#### 5.2.2 ServiceStatus 增加可选原因字符串

**优化前**：`ServiceStatus(Module, bool)` - 无法区分停止原因  
**示例场景**：
- Ping count 达到 → `ServiceStatus(Ping, false, Some("count reached".into()))`
- 用户手动停止 → `ServiceStatus(Ping, false, None)`
- 错误停止 → `ServiceStatus(Ping, false, Some("error: timeout".into()))`

**优化后**：`ServiceStatus(Module, bool, Option<String>)`  
第三个参数：None = 普通停止，Some(reason) = 带原因

**迁移步骤**（已完成）：
- ✅ 修改 `rabbit-models/src/lib.rs`：更新 `UiData` 定义
- ✅ 修改各 Service：传递停止原因（ping.rs, scan.rs）
- ✅ 修改 `app.rs`：处理原因字符串（记录日志）

### 5.3 ✅ 模型精简（rabbit-models 瘦身）（已完成）

| 任务 | 状态 | 说明 |
|------|------|------|
| 删除 `PingResult` 公共定义 | ✅ 完成 | 移至 `rabbit-core` 内部作为私有结构（ping.rs:45） |
| 删除 `ScanResult` 公共定义 | ✅ 完成 | 移至 `rabbit-core` 内部作为私有结构（scan.rs:39） |
| 删除 `HttpAccessLog` 公共定义 | ✅ 完成 | 不存在于公共接口 |
| 删除 `Task`, `Schedule` 公共定义 | ✅ 完成 | 不存在于公共接口 |
| 确认 `UiData` 和 `Module` 在正确位置 | ✅ 完成 | 已在 `rabbit-models` |

---

## 6. 设计决策记录

### 6.1 为什么用 `ServiceStatus(Module, bool, Option<String>)` 而不是 `PingState` 枚举？

| 方案 | 优点 | 缺点 |
|------|------|------|
| **ServiceStatus (当前)** | 统一接口，所有模块通用 | 丢失具体状态（Completed/Error） |
| **PingState 枚举** | 能表达更多状态 | 每个模块需要不同枚举，复杂 |

**决定**：使用 `ServiceStatus` 统一接口 + 具体状态通过日志补充。如果需要更详细状态，可扩展为：
```rust
ServiceStatus { module: Module, status: ServiceStatusType }

enum ServiceStatusType {
    Started, Stopped, Completed, Error(String),
}
```

### 6.2 为什么不用单个 N对1 Channel？

1. **Tokio mpsc 限制**：Receiver 不可克隆，只支持 1对1
2. **数据隔离**：模块间不应互相影响
3. **代码清晰**：每个模块独立处理，路由简单

### 6.3 为什么选择完全事件驱动而不是轮询？

**轮询的问题**：
- 100ms 轮询 = 每秒 10 次空检查，浪费 CPU
- 即使没有数据更新，也要检查 `updated` 标志
- 延迟：最多 100ms 才能看到 UI 更新

**事件驱动的优势**：
- 零 CPU 消耗（空闲时）
- 实时响应（收到数据立即更新 UI）
- 代码更清晰（数据流向明确）

**实现要点**：
- FLTK 组件必须在主线程操作
- `awake_callback` 可以将闭包派发到主线程执行
- 在 `handle_ui_data` 中直接调用 `awake_callback`，无需中间状态标志

---

## 7. 迁移历史

| 日期 | 变更内容 | 状态 |
|------|----------|------|
| 2026-04-20 | 实现 Ping 状态通知 + 任务栏集成 | ✅ 完成 |
| 2026-04-24 | 设计消息驱动架构（本文档初版） | ✅ 部分实现 |
| 2026-04-30 | 修复 scan 按钮状态 + 任务栏标题 | ✅ 完成 |
| 2026-04-30 | 重写文档（本文档 v2.0） | ✅ 完成 |
| 2026-04-30 | 按钮状态改为 awake_callback 事件驱动 | ✅ 完成 |
| 2026-04-30 | ChatUserList 改为 Vec<String> | ✅ 完成 |
| 2026-04-30 | ServiceStatus 增加原因字符串 | ✅ 完成 |
| 2026-04-30 | 模型精简（PingResult/ScanResult 私有化） | ✅ 完成 |
| 2026-04-30 | 文本框更新改为事件驱动 + 删除轮询 | ✅ 完成 |
| 2026-04-30 | 本文档更新到 v2.2 | ✅ 完成 |

---

## 8. 已完成工作（补充）

### 8.1 ✅ Plan 模块重构（已完成）

按照 §8.1 的要求，已完成 Plan 模块重构：
1. ✅ PlanService 内部维护 `Task`/`Schedule` 私有结构（plan.rs:14-73）
2. ✅ 到期提醒通过 `UiData::PlanReminder` 发送（plan.rs:230）
3. ✅ 删除公共的 `Task`, `Schedule` 定义（已私有化）
4. ✅ 改为事件驱动：直接操作 UI 组件，无需通过 `ui_state` 维护 `plan_list`
5. ✅ `app.rs` 中正确处理 `PlanReminder`（显示通知 + 刷新 UI）

### 8.2 ✅ 任务栏进度（已完成）

**实现规则**（遵循需求文档 2.1.4）：
- 全成功 → 绿色（Normal），进度 = 成功次数/5
- 有失败 → 红色（Error），进度 = 失败次数/5
- 取最近 5 次结果，不足 5 次则取实际次数
- 始终显示任务栏进度（无配置开关）

**数据流**：
1. `ping.rs` 计算任务栏状态（progress, total, color）并发送 `UiData::PingState`
2. `app.rs` 接收 `PingState`，调用 `update_taskbar_from_state()` 更新任务栏
3. `taskbar.rs` 仅负责呈现，不计算逻辑

**实现位置**：
- `rabbit-core/src/ping.rs:356-372`：计算并发送 `PingState`
- `rabbit-app/src/app.rs:1164-1177`：处理 `PingState` 并更新任务栏
- `rabbit-platform/src/taskbar.rs:121-145`：`update_taskbar_from_state()` 呈现函数

---

**文档维护**：如发现与实际代码不符，请更新此文档。
