# AGENTS.md

本文件为 Qoder（qoder.com）在此仓库中处理代码时提供指导。

## 项目概述

Rabbit 是一个跨平台的网络工具集和效率工具合集。它将多种实用程序（Ping、HTTP 服务器、TFTP 服务端/客户端、IP 扫描器、局域网聊天）和任务计划功能集成到单一应用程序中。

**技术栈：** Rust + FLTK GUI 框架

## 构建命令

```bash
# 构建（调试）
cargo build

# 构建（发布）
cargo build --release

# 运行
cargo run

# 运行测试
cargo test

# 检查（不构建）
cargo check

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

要求：
- Rust 1.75+（2024 edition）
- 平台相关的构建工具

## 项目架构

### 四层架构

```
┌─────────────────────────────────────────┐
│             表现层                        │
│   视图（FLTK UI）← 视图模型（状态）         │
├─────────────────────────────────────────┤
│             业务层                        │
│   PingService / HttpService / ...       │
├─────────────────────────────────────────┤
│             数据层                        │
│   模型 + 仓储（持久化）                    │
├─────────────────────────────────────────┤
│             基础设施层                     │
│   WindowsPlatform / LinuxPlatform       │
└─────────────────────────────────────────┘
```

### 模块结构

```
rabbit/
├── crates/
│   ├── app/              # 主程序入口 + UI
│   ├── adapter/          # 平台适配层
│   ├── service/          # 业务服务层
│   ├── schema/           # 数据模型层
│   ├── rabbit-config/    # 配置持久化
│   └── rabbit-diag/      # 诊断日志
├── tests/
├── doc/
│   ├── architecture.md       # 架构设计
│   ├── requirements.md       # 需求规格
│   └── tftp-evaluation.md    # TFTP 库评估
└── AGENTS.md
```

### 功能模块

| 模块 | 描述 |
|--------|-------------|
| Ping | ICMP Ping，支持任务栏状态显示 |
| IP 扫描器 | 网络 IP 扫描 |
| HTTP 服务器 | 简易 HTTP 文件服务器 |
| TFTP 服务端/客户端 | TFTP 文件传输 |
| 任务计划器 | 定时提醒 |
| 局域网聊天 | UDP 广播聊天 |

## 第三方库

| 模块 | 库 | 备注 |
|--------|---------|-------|
| UI 框架 | fltk | 跨平台原生 UI |
| 异步运行时 | tokio | 异步 I/O |
| HTTP 服务器 | axum | 轻量异步 HTTP |
| TFTP | async-tftp | 支持 Handler 的异步 TFTP |
| Ping | surge-ping | ICMP Ping |
| 序列化 | serde | JSON/配置 |

## 发布构建优化

```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # 按体积优化
lto = true           # 链接时优化
panic = "abort"      # 减少二进制体积
strip = true         # 去除符号表
```

## 平台支持

- Windows x64
- Linux x64（glibc）
- Linux arm64

## 文档

- `doc/architecture.md` - 技术选型与架构设计
- `doc/requirements.md` - 详细需求和 UI 规格
- `doc/tftp-evaluation.md` - Rust TFTP 库评估
- `doc/progress.md` - 开发进度跟踪
- `doc/requirements-gap-analysis.md` - 需求与实现差异分析
- `doc/version-management.md` - 版本与发布管理
- `doc/ui-framework-evaluation.md` - UI 框架对比
- `doc/changelog-solution.md` - 更新日志工具对比
