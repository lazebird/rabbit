# 消息驱动重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将项目重构为以 `UiData` 为核心的消息驱动架构，清理 `rabbit-models` 中的所有非必要模型。

---

### Task 1: 通信协议下沉 (rabbit-models)

**Files:**
- Modify: `crates/rabbit-models/src/lib.rs`
- Modify: `crates/rabbit-core/src/ui_channel.rs`

- [ ] **Step 1: 迁移 UiData 定义**
将 `UiData` 和 `Module` 枚举从 `rabbit-core` 移至 `rabbit-models`。

- [ ] **Step 2: 清理 re-exports**
更新 `rabbit-models/src/lib.rs`，移除对即将删除的模块（http, chat 等）的引用。

### Task 2: rabbit-core 服务内聚化重构

**Files:**
- Modify: `crates/rabbit-core/src/*.rs`

- [ ] **Step 1: HTTP & Chat & TFTP 改造**
移除对模型层结果结构的依赖。在产生数据处，直接调用字符串格式化并发送 `UiData::Log`。

- [ ] **Step 2: Ping 改造**
移除 `PingResult` 等结构。`PingService` 内部计算好统计信息后，直接发送格式化字符串。

- [ ] **Step 3: Plan 改造 (重点)**
将 `Task` 结构移入 `plan.rs` 变为私有。修改 `add_task` 接口，接收简单参数（String, i32 等）。

- [ ] **Step 4: Scan 改造**
移除 `ScanResult` 公共定义。

### Task 3: rabbit-app 状态管理瘦身

**Files:**
- Modify: `crates/rabbit-app/src/view_model.rs`
- Modify: `crates/rabbit-app/src/app.rs`

- [ ] **Step 1: 移除 AppViewModel 冗余成员**
删除 `ping_results`, `scan_results`, `chat_messages` 等复杂列表。

- [ ] **Step 2: 简化事件处理**
更新 `app.rs` 中的 `handle_ui_data`。确保它只负责将收到的字符串追加到 `UiState` 或更新简单的状态标识。

### Task 4: 最终清理与验证

- [ ] **Step 1: 删除 rabbit-models 下的源文件**
物理删除 `http.rs`, `chat.rs`, `tftp.rs`, `ping.rs`, `plan.rs`。

- [ ] **Step 2: 编译与冒烟测试**
运行 `cargo build` 并解决所有剩余的导入问题。
