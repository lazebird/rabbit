# 配置简化实施计划

## 目标

简化依赖关系：服务模块直接从配置模块获取配置，去除中间转换结构。

## 当前模式

```
App → ModuleConfigs (HashMap) → From转换 → HttpServerConfig/TftpServerConfig struct → 服务
```

## 目标模式

```
服务 → 直接读取 ModuleConfigs → 自己初始化
```

---

## 实施步骤

### 步骤 1: 分析受影响的服务（当前状态）

| 服务 | 当前初始化方式 | 需要修改 |
|------|----------------|----------|
| HttpService | `init(HttpServerConfig)` | → `init(&ModuleConfigs)` |
| TftpService | `init(TftpServerConfig, TftpClientConfig)` | → `init(&ModuleConfigs)` |
| ChatService | `init(ChatConfig)` | → `init(&ModuleConfigs)` |
| PingService | `init(log_file)` | 无需修改 |
| ScanService | `init(ScannerConfig)` | 无需修改 |
| PlanService | `init()` | 无需修改 |

### 步骤 2: 更新 rabbit-core 服务接口

修改 `rabbit-core` 中的服务 `init` 方法：

```rust
// HTTP 服务 - 新接口
pub async fn init(modules: &ModuleConfigs) -> Result<()> {
    let port = modules.get_integer("http", "port").unwrap_or(8000) as u16;
    let root_path = modules.get_array("http", "dirs")
        .and_then(|dirs| dirs.first().cloned())
        .unwrap_or_default();
    let shell = modules.get_bool("http", "shell").unwrap_or(false);
    let auto_index = modules.get_bool("http", "autoindex").unwrap_or(true);
    let video_play = modules.get_bool("http", "videoplay").unwrap_or(true);
    
    // ... 初始化逻辑
}

// TFTP 服务 - 新接口  
pub async fn init(modules: &ModuleConfigs) -> Result<()> {
    let port = modules.get_integer("tftpd", "port").unwrap_or(69) as u16;
    let root_path = modules.get_array("tftpd", "work_dirs")
        .and_then(|dirs| dirs.first().cloned())
        .unwrap_or_default();
    // ...
}

// Chat 服务 - 新接口
pub async fn init(modules: &ModuleConfigs) -> Result<()> {
    let username = modules.get_string("chat", "username")
        .unwrap_or_else(|| "User@PC".to_string());
    let port = modules.get_integer("chat", "port").unwrap_or(1314) as u16;
    // ...
}
```

### 步骤 3: 更新 app.rs 调用

```rust
// HTTP
http_service.init(&config.modules).await?;

// TFTP
tftp_service.init(&config.modules).await?;

// Chat
chat_service.init(&config.modules).await?;
```

### 步骤 4: 删除转换代码

- 删除 `rabbit-models/src/http.rs` 中的 `impl From<&ModuleConfigs> for HttpServerConfig`
- 删除 `rabbit-models/src/tftp.rs` 中的 `impl From` 转换
- 删除 `rabbit-models/src/chat.rs` 中的 `impl From` 转换

### 步骤 5: 验证编译

```bash
cargo build --release
```

---

## 依赖简化前后对比

| 方面 | 简化前 | 简化后 |
|------|-------|-------|
| 文件依赖 | app 依赖 models (From转换) | 服务直接读取配置 |
| 代码路径 | App → 转换 → 服务 | 服务 → 配置 |
| 结构数量 | 多份结构定义 | 仅 HashMap + 服务内部结构 |
| 配置读取 | 分散在多处 | 集中在服务内部 |

---

## 实施状态

- [ ] 步骤 1: 分析（进行中）
- [ ] 步骤 2: 更新 rabbit-core 接口
- [ ] 步骤 3: 更新 app.rs
- [ ] 步骤 4: 删除转换代码
- [ ] 步骤 5: 验证编译

---

文档版本：1.0
更新日期：2026-04-23