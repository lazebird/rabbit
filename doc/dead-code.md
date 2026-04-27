# Dead Code 清理报告

## 处理方式

已删除 crate-level `#![allow(dead_code)]`

## 配置字段实现

| 文件 | 结构 | 字段 | 实现方式 |
|------|------|------|----------|
| tftpc.rs | ClientConfig | block_size | TFTP包添加blksize选项 |
| tftpc.rs | ClientConfig | timeout_secs | 函数参数传递 |
| scan.rs | ScanResult | response_time_ms | 从ping返回 |
| scan.rs | ScanResult | open_ports | 字段已添加 |
| http.rs | HttpService | log_path | Option<String> 从配置读取 |
| http.rs | ServerConfig | log_path | 从平台配置读取 |

## 变更说明

| 原字段 | 新实现 | 说明 |
|--------|--------|------|
| HttpAccessLog (6字段) | log_path: Option<String> | 原结构删除，改用日志文件路径(字符串或None) |

## 删除的字段 (已处理)

| 文件 | 字段 | 原因 | 状态 |
|------|------|------|------|
| ping.rs | PingCommand::AddTarget | UI不支持运行时添加 | ✓ 已删除 |
| tftpd.rs | TftpOperation | Upload/Download直接用字符串 | ✓ 已删除 |
| tftpd.rs | TftpTransferState | 以实际响应为准 | ✓ 已删除 |
| tftpd.rs | TftpTransfer | 部分字段已删除 | ✓ 已处理 |
| tftpd.rs | TftpLogEntry | 日志为文件路径 | ✓ 已删除 |
| tftpc.rs | TftpTransfer | 配置重复 | ✓ 已删除 |
| plan.rs | TaskState::Snoozed | 未实现 | ✓ 已删除 |
| plan.rs | Task.snooze_until | 依赖Snoozed | ✓ 已删除 |
| plan.rs | Task.id | 未使用 | ✓ 已删除 |

## 剩余警告

| 类型 | 位置 |
|------|------|
| unused_import | ConnectInfo, HashMap, Arc, RwLock |
| unused_variable | timeout_secs x2, start, config |

---

## 编译

```bash
cargo check
```