# Rabbit 项目上下文

Rabbit 是一款使用 Rust 语言和 FLTK GUI 框架开发的跨平台（Windows 和 Linux）网络工具和效率套件。它将多种实用程序集成到一个高性能、低占用、单文件的应用程序中。

## 项目概述

- **核心使命**：提供一个“单文件、无依赖、小体积”的网络工具箱。
- **主要技术栈**：
  - **编程语言**：Rust (Edition 2021)
  - **UI 框架**：[FLTK](https://fltk.org/) (通过 `fltk-rs`)
  - **异步运行时**：[Tokio](https://tokio.rs/)
  - **HTTP 服务器**：Axum / Tower
  - **架构设计**：四层架构（表现层、业务层、数据层、基础设施层）
- **核心功能**：
  - Ping 测试（支持任务栏状态显示）
  - IP 扫描器
  - HTTP 文件服务器（支持视频流播放）
  - TFTP 服务端与客户端
  - 局域网聊天（基于 UDP 广播）
  - 任务计划 / 提醒事项

## 工作区结构

项目作为一个 Rust 工作区组织，包含以下 crate：

- `crates/rabbit-app`：主程序入口和基于 FLTK 的 UI 组件。
- `crates/rabbit-core`：业务逻辑和服务（Ping、HTTP、TFTP 等）。
- `crates/rabbit-models`：共享数据结构和配置模型。
- `crates/rabbit-platform`：平台特定抽象（提权、网络、Shell 集成）。

## 开发指南

### 构建与运行

- **调试构建**：`cargo build`
- **发布构建**：`cargo build --release`（针对体积优化：`opt-level = "z"`，启用 LTO，剥离符号表）
- **运行**：`cargo run`
- **检查**：`cargo check`
- **代码风味检查**：`cargo clippy`
- **代码格式化**：`cargo fmt`

### 测试

项目包含一个统一的测试脚本：

- **快速测试（默认）**：`./test.sh`（编译检查 + 所有测试 + Release 构建）
- **完整测试**：`./test.sh full`（快速测试 + 格式检查 + 警告统计）
- **单元测试**：`./test.sh unit`
- **集成测试**：`./test.sh integration`

### 编码规范

- **错误处理**：应用程序级别使用 `anyhow`，库级别使用 `thiserror`。
- **日志记录**：使用 `tracing` crate。日志在 `main.rs` 中通过 `tracing-subscriber` 初始化。
- **异步编程**：所有 I/O 和并发操作优先使用 `tokio`。
- **配置管理**：通过 `rabbit-models/src/config.rs` 管理，使用 `serde` 和 `config` crate。遵循“双层配置模型”，业务模型通过 `From` trait 从持久化模型转换。
- **UI 设计**：FLTK UI 逻辑分为 `view_model.rs`（状态/逻辑）和 `ui/`（组件/布局）。

## 已知问题与注意事项

- **Windows 启动优化 (防闪现)**：
  - 程序已启用 `#![windows_subsystem = "windows"]` 消除主进程控制台。
  - 所有子进程（reg, powershell）调用均使用 `CREATE_NO_WINDOW` 标志进行底层静默启动。
  - 采用“原子化 UI 初始化”策略，确保窗口首帧渲染即为正确业务状态。
  - 详细技术细节请参考 `doc/windows-optimization.md`。
- **退出保障机制**：
  - 应用程序退出时包含 3 秒的异步清理超时保护。如果 3 秒内未完成正常服务销毁，将强制执行 `std::process::exit(0)`，确保不会产生后台残留进程。
- **权限提升**：
  - 某些功能（如 Ping）需要管理员权限。已通过资源清单（Manifest）实现启动前 UAC 提权，并在 `main` 函数顶端进行校验。

## 相关文档

详细设计方案请参考 `doc/` 目录：
- `doc/architecture.md`：技术选型与高层设计。
- `doc/requirements.md`：功能需求与 UI 规范。
- `doc/config-refactor-design.md`：配置系统重构细节。
- `doc/ui-refactor-plan.md`：UI 优化路线图。
