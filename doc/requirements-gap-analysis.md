# 需求与实现差异分析

## 概述

本文档详细记录需求文档 `requirements.md` 中定义的功能与当前代码实现之间的差异，包括缺失项和多余项。

**分析日期**: 2026-04-20

---

## 缺失项汇总 (需求有但代码未完全实现)

### 高优先级

| 模块 | 缺失项 | 类型 | 详细说明 | 处理状态 |
|------|--------|------|----------|----------|
| ~~TFTPC 服务器端口配置~~ | ~~UI~~ | ~~需求定义 port 配置项~~ | ✅ 已通过选项字符串兼容 |

### 中优先级

| 模块 | 缺失项 | 类型 | 详细说明 | 处理状态 |
|------|--------|------|----------|----------|
| Ping | 任务栏状态显示 | 功能 | `taskbar` 配置存在，但无 Windows 任务栏进度条集成 | 暂不实现(跨平台) |
| Ping | 日志文件输出 | 功能 | `log` 配置存在，但无文件日志写入逻辑 | 暂不实现 |
| HTTP | 右键菜单集成 | 功能 | `shell` 配置存在，但 Windows 注册表集成待完善 | 保持(未来实现) |

### 低优先级

| 模块 | 缺失项 | 类型 | 详细说明 | 处理状态 |
|------|--------|------|----------|----------|
| TFTPD | 队列大小/超时实际使用 | 功能 | `qsize`/`qtout` 配置存在，但 async-tftp Handler 中未使用 | 保持(内部优化) |
| TFTPD | 文件系统日志 | 功能 | `fslog` 配置存在，但无实际日志输出 | 保持(调试用途) |
| Plan | unit 为 enum 类型 | 类型差异 | 需求定义 unit 为 enum，代码中使用 String 类型 | 保持(灵活性更好) |

---

## 多余项汇总 (代码有但需求中无)

### 配置层多余项

| 模块 | 多余配置项 | 说明 | 必要性 | 处理方案 |
|------|-----------|------|--------|----------|
| Settings | `theme` | 主题配置 (Light/Dark/System) | 必要(用户体验) | **保持** |
| Settings | `last_active_tab` | 最后活跃标签页 | 必要(用户体验) | **保持** |
| Settings | `modules` | 模块配置容器 | 必要(架构需要) | **保持** |
| Ping | `PingTarget.timeout_ms` | 需求中 interval 同时作为超时 | 提供灵活配置 | **保持** |
| Ping | `PingTarget.ip` | 解析后的 IP 缓存 | 内部实现优化 | **保持** |
| Scan | `ScanRange.port` | TCP 端口扫描 | 扩展功能 | **保持** |
| Scan | `ScannerConfig.*` | 扫描器内部调优参数 | 性能优化 | **保持** |
| HTTP | `HttpServerConfig.enabled` | 启用状态 | 内部管理需要 | **保持** |
| HTTP | `HttpServerConfig.allow_upload` | 文件上传开关 | 安全需要 | **保持** |
| HTTP | `HttpServerConfig.allow_delete` | 文件删除开关 | 安全需要 | **保持** |
| TFTPD | `TftpServerConfig.bind_addr` | 绑定地址 | 多网卡支持 | **保持** |
| TFTPD | `TftpServerConfig.window_size` | 窗口扩展 (RFC 7440) | 性能优化 | **保持** |
| TFTPC | `TftpClientConfig.local_port` | 本地监听端口 | 高级配置 | **保持** |
| Plan | `Task.id` | 唯一标识符 | 内部管理需要 | **保持** |
| Plan | `Task.enabled` | 启用状态 | 用户控制需要 | **保持** |
| Plan | `Task.state` | 任务状态 | 状态管理需要 | **保持** |
| Plan | `WeekDay` 枚举 | 每周提醒的星期选择 | 扩展功能 | **保持** |
| Chat | `ChatConfig.multicast_addr` | 组播地址 | 替代广播的方案 | **保持** |

### 模型层多余项

以下为业务逻辑扩展字段，均属于合理的功能增强，无需删除：

| 模块 | 多余字段 | 说明 |
|------|---------|------|
| Plan | `Task.description`, `Task.created_at`, `Task.snooze_until`, `Task.last_triggered` | 任务管理扩展 |
| Chat | `ChatMessage.id`, `ChatMessage.message_type`, `ChatUser.hostname`, `ChatRoom` | 聊天功能扩展 |
| Scan | `ScanResult.mac_address`, `ScanResult.open_ports` | 扫描结果扩展 |

---

## 配置模型重复问题

### 问题描述

项目中存在两套配置模型：

1. **config.rs 中的配置模型** (`PingConfig`, `ScanConfig`, `HttpConfig`, `TftpdConfig`, `TftpcConfig`, `PlanConfig`, `ChatModuleConfig`)
   - 用于持久化和 UI 默认值
   - 字段名贴近需求文档

2. **各模块模型文件中的配置模型** (`PingTarget`, `HttpServerConfig`, `TftpServerConfig`, `TftpClientConfig`, `ChatConfig`)
   - 用于业务逻辑
   - 字段名更语义化

### 已知不一致（已修复）

| 模块 | 配置模型A (config.rs) | 配置模型B (业务层) | 差异 | 状态 |
|------|----------------------|-------------------|------|------|
| Chat | `ChatModuleConfig.port = 1314` | `ChatConfig.port = 1314` | ~~默认端口不一致~~ | ✅ 已统一 |
| Chat | `ChatModuleConfig.broadcast_addr = 255.255.255.255` | `ChatConfig.multicast_addr = 255.255.255.255` | ~~广播/组播地址不同~~ | ✅ 已统一 |

### 解决方案（已实施）

**通过 `From` trait 实现配置模型间的自动转换，删除业务层的 `Default` 实现**，确保默认值唯一来源为 config.rs：

| 模块 | 转换实现 | Default 状态 |
|------|----------|-------------|
| HTTP | `From<&HttpConfig> for HttpServerConfig` | ✅ 已删除 |
| TFTP Server | `From<&TftpdConfig> for TftpServerConfig` | ✅ 已删除 |
| TFTP Client | `From<&TftpcConfig> for TftpClientConfig` | ✅ 已删除 |
| Chat | `From<&ChatModuleConfig> for ChatConfig` | ✅ 已删除 |
| Scan | `From<&ScanConfig> for ScanRange` | 保留（无对应 config.rs 模型） |

**架构原则**：
1. 保留分层配置设计（良好的架构实践）
2. config.rs 负责持久化，业务层负责运行时
3. **业务层不再保留 Default 降级方案**，所有默认值以 config.rs 中定义的为准
4. 核心服务 `new()` 使用内部占位配置（placeholder），`init()` 必须传入从 config.rs 转换的配置

---

## 结论与建议

### 无需删除的多余项

经过分析，所有代码中"多余"的配置项和功能扩展都属于：
1. **用户体验增强** (theme, last_active_tab)
2. **内部管理需要** (enabled, id, state)
3. **性能优化** (timeout_ms, window_size, concurrent)
4. **功能扩展** (TCP 扫描, 文件上传/删除)

**建议：保留所有多余项，不删除任何功能。**

### 已修复的问题

1. ~~Chat 默认值不一致~~ - ✅ 已通过 `From<&ChatModuleConfig> for ChatConfig` 统一
2. ~~配置模型默认值冗余~~ - ✅ 已通过删除 Default + From trait 彻底解决
3. ~~TFTPC 端口配置缺失~~ - ✅ 已通过选项字符串兼容
4. ~~HTTP 视频播放器缺失~~ - ✅ 已通过 Video.js 7.1.0 实现
5. ~~TFTP 客户端事件未连接~~ - ✅ 已连接 UI 事件到后端服务
6. ~~Chat 刷新/通知按钮未实现~~ - ✅ 已实现功能
7. ~~Plan 重复逻辑未实现~~ - ✅ 已实现 cycle + unit 功能
8. ~~Windows 通知未实现~~ - ✅ 已使用 PowerShell 实现
9. ~~HTTP 目录管理未实现~~ - ✅ 已实现添加/删除目录
10. ~~全局快捷键未实现~~ - ✅ 已实现 Esc/Enter/F1/F2/F3
11. ~~TFTP 服务器未实现~~ - ✅ 已使用 async-tftp 库实现
12. ~~DNS 反向查找未实现~~ - ✅ 已使用 dns-lookup 库实现
13. ~~Autostart 未实现~~ - ✅ 已实现跨平台自动启动
14. 文档完善 - 所有架构和进度文档已同步更新

---

## 修复记录

| 日期 | 修复项 | 说明 | 状态 |
|------|--------|------|------|
| 2026-04-20 | Chat 默认值修复 | 统一 ChatConfig 默认值为需求定义的值 (port=1314, multicast_addr=255.255.255.255) | ✅ 已修复 |
| 2026-04-20 | HTTP 视频播放器实现 | 使用 Video.js 7.1.0 实现视频在线播放功能，兼容旧方案 | ✅ 已实现 |
| 2026-04-20 | TFTPC 端口配置 | 通过选项字符串兼容旧方案，TftpcConfig.server_port 已存在 | ✅ 已处理 |
| 2026-04-20 | 配置架构重构 | 删除业务层 Default 实现，From trait 统一转换，占位配置模式 | ✅ 已完成 |
| 2026-04-20 | TFTP 客户端连接 | 将 UI Put/Get 事件连接到后端 upload_to/download_from 服务 | ✅ 已实现 |
| 2026-04-20 | Chat 功能完善 | 实现刷新用户列表和发送通知按钮功能 | ✅ 已实现 |
| 2026-04-20 | Plan 重复逻辑 | 实现 cycle + unit (minute/hour/day) 重复周期功能 | ✅ 已实现 |
| 2026-04-20 | Windows 通知 | 使用 PowerShell 实现 Windows 10+ 气球提示通知 | ✅ 已实现 |
| 2026-04-20 | HTTP 目录管理 | 实现 HTTP 服务器目录添加/删除功能 | ✅ 已实现 |
| 2026-04-20 | 全局快捷键 | 实现 Esc/Enter/F1/F2/F3 快捷键功能 | ✅ 已实现 |
| 2026-04-20 | TFTP 服务器 | 使用 async-tftp 库实现完整的 TFTP 服务器功能 | ✅ 已实现 |
| 2026-04-20 | DNS 反向查找 | 添加 dns-lookup 库实现扫描结果主机名解析 | ✅ 已实现 |
| 2026-04-20 | Autostart 自动启动 | 实现跨平台自动启动 (Windows/Linux/macOS) | ✅ 已实现 |
| 2026-04-20 | Settings 功能 | 实现 autostart/top/systray 配置应用 | ✅ 已实现 |
| 2026-04-20 | 文档完善 | 更新 AGENTS.md, architecture.md, progress.md, 本文档 | ✅ 已完成 |
