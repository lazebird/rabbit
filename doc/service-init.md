# 服务统一初始化接口设计

## 设计目标

统一所有服务的初始化接口为无参方法，服务内部自行从配置模块获取所需配置。

## 接口规范

```rust
pub async fn init(&mut self) -> Result<()>
```

### 无参接口优势

| 方面 | 说明 |
|------|------|
| **简化调用** | 调用方无需了解配置细节 |
| **内聚性** | 配置读取逻辑集中在服务内部 |
| **一致性** | 所有服务使用相同接口 |
| **解耦** | 服务与配置模块直接交互 |

## 设计原理

```
App 启动
    │
    ▼
┌─────────────────────────────────────┐
│  service.init()                     │ ← 无参调用
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  内部自行读取 ModuleConfigs:        │
│  - modules.get_string("http", "port")│
│  - modules.get_array("http", "dirs")  │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  初始化完成                         │
└─────────────────────────────────────┘
```

## 服务接口定义

### PingService

```rust
pub async fn init(&mut self) -> Result<()> {
    let log_file = rabbit_platform::config::load_config()
        .and_then(|c| c.modules.get_string("ping", "log"))
        .unwrap_or_default();
    self.init_with_log(log_file).await
}
```

### HttpService

```rust
pub async fn init(&mut self) -> Result<()> {
    let config = rabbit_platform::config::load_config()?;
    let modules = &config.modules;
    let runtime_config = HttpServerConfig {
        enabled: true,
        port: modules.get_integer("http", "port").unwrap_or(8000) as u16,
        root_path: modules.get_array("http", "dirs")
            .and_then(|d| d.first().cloned())
            .unwrap_or_default(),
        // ...
    };
    self.runtime_config = runtime_config;
    Ok(())
}
```

### TftpService

```rust
pub async fn init(&mut self) -> Result<()> {
    let config = rabbit_platform::config::load_config()?;
    let modules = &config.modules;
    // 读取 tftpd 和 tftpc 配置
    self.server_config = server_config;
    self.client_config = client_config;
    Ok(())
}
```

### ChatService

```rust
pub async fn init(&mut self) -> Result<()> {
    let config = rabbit_platform::config::load_config()?;
    let modules = &config.modules;
    // 读取 chat 配置
    self.config = chat_config;
    Ok(())
}
```

### PlanService

```rust
pub async fn init(&mut self) -> Result<()> {
    // 无需配置，启动即可
    Ok(())
}
```

### ScanService

```rust
pub async fn init(&mut self) -> Result<()> {
    let config = rabbit_platform::config::load_config()?;
    // 读取 scan 配置
    self.config = scan_config;
    Ok(())
}
```

## App 调用方式

简化后的调用：

```rust
// 启动服务时
ping_service.init().await?;
http_service.init().await?;
tftp_service.init().await?;
chat_service.init().await?;
plan_service.init().await?;
scan_service.init().await?;
```

对比原来（带参）：

```rust
// 需要手动构建配置
http_service.init(&config.modules).await?;
// 或先获取配置再调用
let config = load_config()?;
http_service.init(config.modules).await?;
```

## 实施清单

- [x] PingService - 无需修改（已无参）
- [x] HttpService - 改为无参，内部读取配置
- [x] TftpService - 改为无参，内部读取配置
- [x] ChatService - 改为无参，内部读取配置
- [x] PlanService - 无需修改
- [ ] ScanService - 待更新
- [ ] app.rs - 更新调用方式

---

文档版本：1.0
更新日期：2026-04-23