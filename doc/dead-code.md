# Dead Code 清单

**处理方式**: 在 `rabbit-core` 模块级别添加 `#![allow(dead_code)]`

## rabbit-core (已用 allow 抑制)

| 文件 | 结构 | 字段/变体 | 说明 |
|------|------|-----------|------|
| ping.rs | PingCommand::AddTarget | 预留动态添加目标 |
| http.rs | HttpAccessLog 字段 | 预留访问日志详情 |
| tftpd.rs | TftpOperation | 传输类型 |
| tftpd.rs | TftpTransferState | 传输状态 |
| tftpd.rs | TftpTransfer 字段 | 传输信息 |
| tftpd.rs | TftpLogEntry 字段 | 日志字段 |
| tftpc.rs | TftpTransfer | 配置重复 |

## rabbit-app

| 文件 | 结构 | 字段/变体 | 说明 |
|------|------|-----------|------|
| planner.rs | PlannerState::Snoozed | 预留 snooze 状态 |
| planner.rs | ScheduledTask 字段 | 预留任务触发信息 |
| scanner.rs | ScanResult::username | 预留用户名 |

## rabbit-platform

| 文件 | 结构 | 字段/变体 | 说明 |
|------|------|-----------|------|
| ping.rs | PingResult 字段 | 预留响应时间/端口 |

---

## 剩余警告

| 警告类型 | 数量 | 原因 |
|---------|------|------|
| static_mut_refs | 22 | FLTK UI 框架 (Rust 2024) |
| unreachable | 1 | cfg guard 限制 |
| dead_code | 0 | ✓ 已抑制 |

## 当前状态

编译通过 ✓