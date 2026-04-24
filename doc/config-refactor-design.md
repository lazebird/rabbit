# 统一配置重构设计方案 (2026-04-24)

## 1. 目标
- **去中心化**: 业务服务直接通过配置模块获取参数，不再通过 `init` 注入。
- **清理冗余**: 彻底删除 `rabbit-models` 中用于中间转换的配置结构体。
- **性能优化**: 实现内存缓存（Memory Cache）和脏检查（Dirty Check）以减少磁盘 I/O。

## 2. 核心架构

### 2.1 配置单例 (rabbit-platform)
在 `rabbit-platform` 中维护一个全局配置单例，支持并发访问。

```rust
// 伪代码参考
static CONFIG_CACHE: RwLock<AppConfig>;
static LAST_SAVED_HASH: AtomicU64; // 用于脏检查
```

### 2.2 统一接口
提供静态辅助函数，简化调用方代码：
- `config::get_string(module, key) -> Option<String>`
- `config::get_integer(module, key) -> Option<i64>`
- `config::get_bool(module, key) -> Option<bool>`
- `config::set_value(module, key, value)`
- `config::sync() -> Result<()>` // 脏检查并持久化

## 3. 详细变更说明

### 3.1 rabbit-models (瘦身)
- **删除**: `HttpServerConfig`, `TftpServerConfig`, `TftpClientConfig`, `ChatConfig`, `PingTarget` (配置部分)。
- **保留**: 纯业务模型如 `PingResult`, `TftpTransfer` 等。
- **清理**: 移除所有 `impl From<&ModuleConfigs>` 实现。

### 3.2 rabbit-platform (增强)
- 引入 `OnceLock` 管理全局配置。
- 实现 `dirty_check`: 在保存前对比内容哈希或序列化字符串。

### 3.3 rabbit-core (自给自足)
- 修改 `HttpService`, `TftpService`, `ChatService`, `PingService`。
- `init()` 或 `start()` 不再接收配置参数。
- 在逻辑内部通过 `rabbit_platform::config::get_*` 获取所需配置。

### 3.4 rabbit-app (解耦)
- 仅负责服务的生命周期管理。
- 启动服务时不再准备复杂的配置对象。

## 4. 实施步骤
1.  **基础设施**: 在 `rabbit-platform` 实现缓存单例和脏检查逻辑。
2.  **服务重构**: 逐个修改 `rabbit-core` 下的服务，改用新接口获取配置。
3.  **App 适配**: 修改 `app.rs` 以适应无参的初始化调用。
4.  **最终清理**: 删除 `rabbit-models` 中所有冗余结构和转换代码。
5.  **验证**: 编译并运行测试，确保配置读写正常。

---
文档版本: 1.0
状态: 待执行
