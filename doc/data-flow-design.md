# Rabbit 数据流与状态同步设计文档

## 1. 当前架构概述

### 1.1 组件分层

```
┌─────────────────────────────────────────────────────────────┐
│                     UI 层 (rabbit-app)                   │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐       │
│  │ ping_tab   │  │ http_tab   │  │ tftpd_tab  │ ...   │
│  └────────────┘  └────────────┘  └────────────┘       │
├─────────────────────────────────────────────────────────────┤
│              状态管理层                                   │
│  ┌──────────────────────┐  ┌──────────────────────┐       │
│  │   ui_state.rs       │  │  view_model.rs      │       │
│  │   (运行时显示状态)   │  │   (配置+运行状态)    │       │
│  └──────────────────────┘  └──────────────────────┘       │
├─────────────────────────────────────────────────────────────┤
│  ┌───────────────────────────────────────────────────────┐  │
│  │                 app.rs (事件中枢)                  │  │
│  │  - 接收 UiEvent                                    │  │
│  │  - 调用 Service.update()                           │  │
│  │  - 手动更新 UI 状态                                │  │
│  └───────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                 业务服务层 (rabbit-core)                  │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐       │
│  │ PingService│  │HttpService │  │TftpdService│       │
│  └────────────┘  └────────────┘  └────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 当前数据流模式

```
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│   Service    │ ───► │    app.rs    │ ───► │  ui_state    │
│  (内部数据)   │      │  (路由转发)   │      │  (缓冲存储)   │
└──────────────┘      └──────────────┘      └──────────────┘
                                               │
                                               ▼
                                        ┌────────���─────┐
                                        │  UI 显示     │
                                        │ (TextDisplay)│
                                        └──────────────┘
```

## 2. 现有问题分析

### 2.1 问题清单

| 问题 | 说明 | 影响 |
|------|------|------|
| **UI 更新分散** | 每个模块在 app.rs 中手动调用 set_*_running() + append_*_log() | 代码重复，维护困难 |
| **状态同步重复** | view_model 和 ui_state 两处都要更新 | 数据不一致风险 |
| **无统一返回** | Service.update() 返回 Result，无法自动更新 UI | 错误处理不统一 |
| **日志格式各异** | 每个模块的日志 append 方式不同 | 难以解析 |
| **无状态订阅** | UI 无法主动感知服务状态变化 | 需要轮询或手动刷新 |

### 2.2 具体示例

**当前 ping 结果显示流程：**

```rust
// app.rs 中的繁琐流程
1. buttons 点击 → UiEvent::PingStart
2. event_handler 解析参数
3. 直接调用 append_ping_output() 写入 ui_state
4. 手动 set_ping_running(true)
5. 手动 set_ping_stats()
6. ping_service.add_target() 添加目标
7. ping results 通过 tokio spawn 异步写入 ui_state

// 问题：
// - 数据流太长，跨越多个文件
// - 异步结果写入需要特殊处理 (awake_callback)
// - 成功/失败都需要手动更新 UI
```

## 3. 优化方案设计

### 3.1 目标

1. **统一接口**：所有服务的 update() 返回结构化结果
2. **自动 UI 更新**：结果自动同步到 ui_state
3. **状态订阅**：UI 可感知服务状态变化
4. **简化路由**：减少 app.rs 中的手动调用

### 3.2 ServiceUpdateResult 设计

```rust
// 定义在 rabbit-core/src/lib.rs

/// Service 操作返回结果
#[derive(Debug, Clone)]
pub enum ServiceUpdateResult {
    /// 服务已启动
    Started { 
        message: String,           // 状态消息
        is_running: bool,       // 当前运行状态
    },
    /// 服务已停止
    Stopped { 
        message: String,
        is_running: bool,
    },
    /// 操作失败
    Error { 
        message: String,
        reason: String,          // 失败原因
        is_running: bool,       // 操作后的状态
    },
    /// 状态无变化
    NoChange { 
        is_running: bool,
        message: String,
    },
}

impl ServiceUpdateResult {
    pub fn is_running(&self) -> bool { ... }
    pub fn message(&self) -> &str { ... }
    pub fn needs_ui_update(&self) -> bool { ... }  // 是否需要刷新 UI
}
```

### 3.3 统一 UI 更新流程

```rust
// app.rs 简化后的代码

async fn handle_service_update<S: Service>(
    service: &mut S,
    ui_log_fn: fn(&str),      // append_*_log
    set_running_fn: fn(bool), // set_*_running
    view_model_fn: fn(bool) // set_*_running in view_model
) -> Result<()> {
    let result = service.update().await;
    
    match result {
        ServiceUpdateResult::Started { message, is_running } => {
            ui_log_fn(&format!("{}\n", message));
            set_running_fn(is_running);
            view_model_fn(is_running);
        }
        ServiceUpdateResult::Stopped { message, is_running } => {
            ui_log_fn(&format!("{}\n", message));
            set_running_fn(is_running);
            view_model_fn(is_running);
        }
        ServiceUpdateResult::Error { message, reason, is_running } => {
            ui_log_fn(&format!("Error: {} - {}\n", message, reason));
            set_running_fn(is_running);
            view_model_fn(is_running);
        }
        ServiceUpdateResult::NoChange { is_running, message } => {
            // 可选：显示提示
        }
    }
}

// 使用示例
match event {
    UiEvent::HttpToggle => {
        handle_service_update(
            &mut *http_service,
            crate::ui_state::append_http_log,
            crate::ui_state::set_http_running,
            |v| view_model.set_http_running(v)
        ).await?;
    }
}
```

### 3.4 事件驱动 UI 更新

```rust
// ui_events.rs 新增事件响应

#[derive(Debug, Clone)]
pub enum UiEvent {
    // ... 现有事件 ...
    
    // 新的统一事件格式
    ServiceToggle {
        module: ModuleName,  // "ping", "http", "tftpd" 等
    },
}

/// 模块名称枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleName {
    Ping,
    Http,
    Tftpd,
    Tftpc,
    Scan,
    Chat,
    Plan,
}

/// 获取对应的服务更新函数
impl ModuleName {
    pub fn update_fn(&self) -> impl Fn(&mut App) -> ... {
        match self {
            ModuleName::Ping => App::toggle_ping,
            ModuleName::Http => App::toggle_http,
            // ...
        }
    }
}
```

## 4. 数据流优化详细设计

### 4.1 统一状态存储结构

```rust
// ui_state.rs 扩展

impl UiState {
    /// 统一的日志缓冲区
    pub fn append_module_log(&mut self, module: ModuleName, msg: &str) {
        let key = match module {
            ModuleName::Ping => "ping_output",
            ModuleName::Http => "http_log",
            ModuleName::Tftpd => "tftpd_log",
            // ...
        };
        
        self.log_buffers
            .entry(key.to_string())
            .or_insert_with(String::new)
            .push_str(msg);
        
        self.updated.insert(key.to_string(), true);
    }
    
    /// 统一的状态更新
    pub fn set_module_running(&mut self, module: ModuleName, running: bool) {
        let key = match module {
            ModuleName::Ping => "ping_running",
            ModuleName::Http => "http_running",
            // ...
        };
        
        self.running_states.insert(key.to_string(), running);
        self.updated.insert(key.to_string(), true);
    }
}
```

### 4.2 服务日志级别

```rust
/// 日志级别
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,   // 详细信息
    Info,    // 一般消息
    Warning, // 警告
    Error,   // 错误
}

/// 扩展 ServiceUpdateResult
pub enum ServiceUpdateResult {
    Started { message: String, logs: Vec<(LogLevel, String)>, is_running: bool },
    Stopped { message: String, logs: Vec<(LogLevel, String)>, is_running: bool },
    Error { message: String, reason: String, logs: Vec<(LogLevel, String)>, is_running: bool },
    NoChange { message: String, logs: Vec<(LogLevel, String)>, is_running: bool },
}
```

### 4.3 异步结果自动刷新

```rust
// ui_refresh.rs 扩展

/// 注册服务的状态变化回调
pub fn register_service_state_callback<F>(module: ModuleName, callback: F)
where
    F: Fn(bool) + Send + Sync + 'static
{
    // 注册回调，在状态变化时自动调用
}
```

## 5. 实现计划

### Phase 1: 基础重构
1. 定义 ServiceUpdateResult 枚举
2. 为每个 Service 实现统一接口
3. 修改 app.rs 事件处理

### Phase 2: UI 层优化
1. 扩展 ui_state 统一方法
2. 简化各 tab 的回调逻辑
3. 实现状态自动刷新

### Phase 3: 高级功能
1. 日志缓冲和历史
2. 状态变化订阅
3. 错误恢复机制

## 6. 文件变更清单

| 文件 | 变更内容 |
|------|----------|
| `rabbit-core/src/lib.rs` | 新增 ServiceUpdateResult |
| `rabbit-core/src/*.rs` | 每个服务实现 update() 返回 ServiceUpdateResult |
| `rabbit-app/src/app.rs` | 简化事件处理，统一调用 |
| `rabbit-app/src/ui_state.rs` | 新增统一状态方法 |
| `rabbit-app/src/ui_events.rs` | 新增统一事件格式 |

---

文档版本：1.0
创建日期：2026-04-24
更新日期：2026-04-24