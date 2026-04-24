# 消息驱动架构与模型精简设计方案 (2026-04-24)

## 1. 核心思想
遵循“**以消息为唯一契约**”原则。UI 与 Service 之间不再通过共享复杂的业务模型（如 `Task`, `ScanResult`, `HttpAccessLog`）来同步状态，而是通过统一的消息枚举 `UiData` 进行单向数据推送。

## 2. 架构变更

### 2.1 重新定义 `rabbit-models`
`rabbit-models` 将被精简为“纯协议层”，仅保留跨模块通信必须的原子数据和配置。

- **保留**:
    - `AppConfig`, `ConfigValue`: 静态配置定义。
    - `UiData`, `Module`: UI 与 Core 的通信协议（从 `rabbit-core` 迁入）。
    - `ScanRange`: 必要的输入参数。
- **消除 (作为公共模型)**:
    - 所有的业务结果结构体：`PingResult`, `ScanResult`, `TftpTransfer`, `HttpAccessLog`, `ChatMessage` 等。
    - 所有的业务逻辑结构体：`Task`, `Schedule`, `ChatRoom` 等。

### 2.2 内部化与内聚
原有的公共模型将根据其用途处理：
1. **显示数据**: Service 直接将其格式化为 `String`，通过 `UiData::Log` 或专用变体发送给 UI。
2. **业务逻辑**: 如果 Service 运行需要这些结构（如 `Task` 调度），则将其移至 `rabbit-core` 的 Service 内部，变为私有结构。

## 3. 各模块重构细节

| 模块 | 变更内容 |
| :--- | :--- |
| **Common** | 将 `UiData` 和 `Module` 定义从 `rabbit-core/ui_channel.rs` 移至 `rabbit-models/lib.rs`。 |
| **HTTP** | 删除 `HttpAccessLog`。`HttpService` 产生请求时直接格式化字符串发送。 |
| **Ping** | 删除 `PingResult`, `PingSummary`。`PingService` 直接推送格式化好的统计字符串。 |
| **Scan** | 删除 `ScanResult`。`ScanService` 在发现主机时直接推送 Log。 |
| **TFTP** | 删除 `TftpTransfer`, `TftpLogEntry`。进度和日志均以字符串形式推送。 |
| **Plan** | 删除 `Task`, `Schedule` 等公共定义。`PlanService` 内部维护私有 `InternalTask`。 |
| **App** | `AppViewModel` 移除所有 `Vec<Results>` 成员。`app.rs` 的 `handle_ui_data` 仅负责更新文本框和状态位。 |

## 4. 预期收益
- **极度解耦**: 更改业务逻辑或显示格式时，几乎不需要修改模型层。
- **性能提升**: 消除大量结构体克隆（Clone）和序列化开销。
- **代码整洁**: 删除了约 50% 的样板代码（Boilerplate）。

---
文档版本: 1.0
状态: 待执行
