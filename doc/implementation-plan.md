# Rabbit 数据流实施计划

## 1. 最终设计 (基于 doc/data-flow-design.md)

### 1.1 UiData 枚举 (最终版)

```rust
#[derive(Debug, Clone)]
pub enum UiData {
    // 通用日志
    Log(Module, String),
    
    // Ping
    PingStats(String),                        // 统计框
    PingState { address, progress, total, color },
    
    // Scan
    ScanProgress(String),
    
    // Plan
    PlanReminder(String),                     // 提醒消息
    
    // Chat
    ChatMessage(String, String),              // (username, message)
    ChatUserList(String),                     // "," 分隔的用户列表
    
    // 错误
    Error(Module, String),
}

pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}
```

### 1.2 Channel 结构

```rust
pub struct UiChannels {
    ping_tx: mpsc::Sender<UiData>,
    http_tx: mpsc::Sender<UiData>,
    scan_tx: mpsc::Sender<UiData>,
    tftp_tx: mpsc::Sender<UiData>,
    chat_tx: mpsc::Sender<UiData>,
    plan_tx: mpsc::Sender<UiData>,
}
```

---

## 2. 实施步骤

### Phase 1: 核心基础设施

| 步骤 | 文件 | 改动内容 |
|------|------|----------|
| 1.1 | `crates/rabbit-core/src/lib.rs` | 新增 Module 枚举 |
| 1.2 | `crates/rabbit-core/src/ui_channel.rs` (新) | 新增 UiData 枚举, UiChannels 结构 |
| 1.3 | `crates/rabbit-app/src/lib.rs` | 导出 UiData, Module |

### Phase 2: HTTP 改造

| 步骤 | 文件 | 改动内容 |
|------|------|----------|
| 2.1 | `crates/rabbit-core/src/http.rs` | 添加 tx 字段，发送 Log |
| 2.2 | `crates/rabbit-app/src/app.rs` | 创建 channel，启动 receiver，删除轮询 |
| 2.3 | `crates/rabbit-app/src/http_tab.rs` | 无需改动 |

### Phase 3: Ping 改造

| 步骤 | 文件 | 改动内容 |
|------|------|----------|
| 3.1 | `crates/rabbit-core/src/ping.rs` | 添加 tx 字段，发送 Log/PingStats/PingState |
| 3.2 | `crates/rabbit-app/src/app.rs` | 创建 channel，启动 receiver，删除轮询 |

### Phase 4: Scan 改造

| 步骤 | 文件 | 改动内容 |
|------|------|----------|
| 4.1 | `crates/rabbit-core/src/scan.rs` | 添加 tx 字段，发送 Log/ScanProgress |
| 4.2 | `crates/rabbit-app/src/app.rs` | 创建 channel，启动 receiver，删除轮询 |

### Phase 5: TFTP 改造

| 步骤 | 文件 | 改动内容 |
|------|------|----------|
| 5.1 | `crates/rabbit-core/src/tftpd.rs` | 添加 tx 字段，发送 Log |
| 5.2 | `crates/rabbit-app/src/app.rs` | 创建 channel，启动 receiver |
| 5.3 | `crates/rabbit-core/src/tftpc.rs` | 添加 tx 字段，发送 Log |
| 5.4 | `crates/rabbit-app/src/app.rs` | 创建 channel，启动 receiver |

### Phase 6: Chat 改造

| 步骤 | 文件 | 改动内容 |
|------|------|----------|
| 6.1 | `crates/rabbit-core/src/chat.rs` | 添加 tx 字段，发送 ChatMessage/ChatUserList |
| 6.2 | `crates/rabbit-app/src/app.rs` | 创建 channel，启动 receiver，删除轮询 |

### Phase 7: Plan 改造

| 步骤 | 文件 | 改动内容 |
|------|------|----------|
| 7.1 | `crates/rabbit-core/src/plan.rs` | 添加 tx 字段，发送 PlanReminder |
| 7.2 | `crates/rabbit-app/src/app.rs` | 创建 channel，启动 receiver |

---

## 3. 待明确问题

### 问题 3.1: Channel 创建方式

**设计**: app.rs 集中创建所有 channel

**流程**:
1. app.rs::new() 创建 channel (tx, rx)
2. 将 tx 克隆传给各 service 初始化
3. app 保存 rx 在 UiReceivers 结构中
4. app::run() 启动 receiver 任务处理数据

```rust
// app.rs 结构变化
pub struct App {
    // 现有字段
    services: ...,
    // 新增
    ui_receivers: UiReceivers,
}

pub struct UiReceivers {
    http: mpsc::Receiver<UiData>,
    ping: mpsc::Receiver<UiData>,
    scan: mpsc::Receiver<UiData>,
    tftpd: mpsc::Receiver<UiData>,
    tftpc: mpsc::Receiver<UiData>,
    chat: mpsc::Receiver<UiData>,
    plan: mpsc::Receiver<UiData>,
}

impl App::new() async {
    // 创建 channels
    let (http_tx, http_rx) = mpsc::channel(100);
    let (ping_tx, ping_rx) = mpsc::channel(100);
    // ...
    
    // 初始化 services (传入 tx)
    let mut http_service = HttpService::new(http_tx);
    let mut ping_service = PingService::new(ping_tx);
    // ...
    
    // 保存 receivers
    let ui_receivers = UiReceivers {
        http: http_rx,
        ping: ping_rx,
        // ...
    };
    
    App { services, ui_receivers }
}

impl App::run() {
    // 为每个 module 启动 receiver 任务
    let mut rx = self.ui_receivers.http.take();
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            handle_ui_data(data).await;
        }
    });
}
```

### 问题 3.2: Error 处理策略

当前设计: 错误显示在窗口标题

```rust
UiData::Error(module, msg) => {
    // 显示在窗口标题
    if let Some(mut win) = ui_state::get_main_window() {
        fltk::app::awake_callback(move || {
            win.set_label(&format!("Rabbit - Error: {}", msg));
        });
    }
}
```

问题: 窗口标题显示错误会覆盖 ping 地址显示

建议: 使用独立的错误提示区域或状态栏？

### 问题 3.3: 轮询代码删除位置

需要删除的 tokio::spawn 位置 (app.rs):
- 行 546: event_handle (保留)
- 行 866: ping 轮询
- 行 991: scan 轮询
- 行 1049: http 轮询
- 行 1114: chat 轮询

---

## 4. 风险评估

### 风险 4.1: Service 初始化顺序

- 需要先创建 channel，再创建 service
- service 初始化签名需要改变

### 风险 4.2: 内存泄漏

- channel buffer 设置为 100
- receiver 任务需要正确退出

---

## 5. 验收标准

### Phase 1 验收
- [ ] UiData, Module 定义在 rabbit-core
- [ ] 编译通过

### Phase 2+ 验收 (每个 Phase)
- [ ] 对应模块日志实时显示
- [ ] 轮询代码已删除
- [ ] 无编译错误

### 整体验收
- [ ] 所有模块工作正常
- [ ] 原有轮询代码全部删除

---

## 6. 已确认问题 (之前待确认)

1. **Channel 管理方案**: app.rs 集中创建 ✅
2. **Plan 模块**: 需要加入 channel 机制 ✅
3. **Error 处理**: 记录日志但不显示错误框 ✅

---

## 7. 已完成 (Phase 1)

- ✅ UiData / Module 定义在 rabbit-core/src/ui_channel.rs
- ✅ rabbit-core 导出 UiData, Module
- ✅ rabbit-app 导入 UiData, Module
- ✅ 编译通过

---

## 6. 需要你确认的问题

1. **Channel 管理方案**: 方案 A (app.rs 集中创建) 可以吗？
2. **Plan 模块**: 需要加入 channel 机制吗？
3. **Error 处理**: 遇到 channel 错误时，记录日志但不显示错误框，可以吗？

---

版本: 1.0 | 2026-04-24