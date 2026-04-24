# 统一配置重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现统一配置模块，支持内存缓存、脏检查，并完成业务模块的去中心化适配。

**Architecture:** 采用单例缓存模式。`rabbit-platform` 维护全局配置状态，各业务模块按需通过静态接口拉取。

**Tech Stack:** Rust, `OnceLock`, `RwLock`, `toml`, `rabbit-models`.

---

### Task 1: rabbit-platform 配置单例实现

**Files:**
- Modify: `crates/rabbit-platform/src/config.rs`

- [ ] **Step 1: 引入依赖并定义全局状态**
使用 `OnceLock` 和 `parking_lot::RwLock` (或标准库 RwLock) 定义全局缓存。

- [ ] **Step 2: 实现懒加载和脏检查逻辑**
实现一个内部函数，对比当前内存内容与磁盘内容的差异，仅在变化时写入。

- [ ] **Step 3: 提供静态访问接口**
封装 `get_string`, `get_integer`, `get_bool` 等接口。

### Task 2: rabbit-core 服务重构 (HTTP & Chat)

**Files:**
- Modify: `crates/rabbit-core/src/http.rs`
- Modify: `crates/rabbit-core/src/chat.rs`

- [ ] **Step 1: 修改 HttpService**
移除 `init` 的配置参数，改为在 `start` 内部直接调用 `rabbit_platform::config::get_integer("http", "port")` 等。

- [ ] **Step 2: 修改 ChatService**
同上，自主获取用户名和端口。

### Task 3: rabbit-core 服务重构 (TFTP & Ping)

**Files:**
- Modify: `crates/rabbit-core/src/tftpd.rs`
- Modify: `crates/rabbit-core/src/tftpc.rs`
- Modify: `crates/rabbit-core/src/ping.rs`

- [ ] **Step 1: TFTP 服务内定义私有配置**
将原本在 models 中的 TFTP 结构移入 core 并改为私有，内部完成解析。

- [ ] **Step 2: Ping 服务改造**
将 `PingTarget` 移入 `core` 作为运行时模型。

### Task 4: rabbit-app 适配与模型清理

**Files:**
- Modify: `crates/rabbit-app/src/app.rs`
- Modify: `crates/rabbit-models/src/lib.rs`
- Delete: `crates/rabbit-models/src/http.rs`
- Delete: `crates/rabbit-models/src/chat.rs`
- Modify: `crates/rabbit-models/src/tftp.rs`
- Modify: `crates/rabbit-models/src/ping.rs`

- [ ] **Step 1: 更新 app.rs 调用链**
删除所有配置转换和注入逻辑。

- [ ] **Step 2: 执行模型层大清理**
删除所有冗余结构体和 `From` 实现。

- [ ] **Step 3: 最终编译验证**
运行 `cargo build`。
