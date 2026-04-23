# AppViewModel 接口文档

## 概述

`AppViewModel` 是 Rabbit 应用的统一配置管理和状态管理入口。

## 结构定义

```rust
pub struct AppViewModel {
    pub config: AppConfig,                         // 配置内存缓存
    pub ping_results: HashMap<String, PingSummary>,  // Ping 结果缓存
    pub ping_running: bool,
    pub http_running: bool,
    pub tftp_server_running: bool,
    pub chat_running: bool,
    pub scan_running: bool,
    pub chat_messages: Vec<ChatMessageView>,
    pub scan_results: Vec<ScanResultView>,
}
```

## 构造方法

### new

```rust
pub fn new(config: AppConfig) -> Self
```

创建 ViewModel 实例。

**参数**
- `config: AppConfig` - 初始配置

**返回**
- `Self` - ViewModel 实例

## 配置方法

### get_config

```rust
pub fn get_config(&self) -> AppConfig
```

获取配置克隆。

### update_config

```rust
pub fn update_config(&mut self, config: AppConfig)
```

更新内存配置（不保存）。

### update_and_save

```rust
pub fn update_and_save(&mut self, config: AppConfig) -> Result<()>
```

更新配置并同步保存到磁盘。

### save_settings

```rust
pub fn save_settings(&mut self) -> Result<()>
```

保存当前配置到磁盘。

### update_global

```rust
pub fn update_global(
    &mut self,
    language: Language,
    theme: Theme,
    systray: bool,
    top: bool,
    autostart: bool,
    autoupdate: bool
) -> Result<()>
```

更新全局配置并保存。

### update_last_tab

```rust
pub fn update_last_tab(&mut self, tab: usize) -> Result<()>
```

更新最后活动标签页并保存。

## 业务状态方法

### Ping

```rust
pub fn is_ping_running(&self) -> bool
pub fn set_ping_running(&mut self, running: bool)
pub fn get_ping_result(&self, target: &str) -> Option<&PingSummary>
pub fn update_ping_result(&mut self, result: PingSummary)
```

### HTTP

```rust
pub fn is_http_running(&self) -> bool
pub fn set_http_running(&mut self, running: bool)
```

### TFTP

```rust
pub fn is_tftp_server_running(&self) -> bool
pub fn set_tftp_server_running(&mut self, running: bool)
```

### Chat

```rust
pub fn get_chat_messages(&self) -> &[ChatMessageView]
pub fn add_chat_message(&mut self, message: ChatMessageView)
pub fn is_chat_running(&self) -> bool
pub fn set_chat_running(&mut self, running: bool)
```

### Scan

```rust
pub fn is_scan_running(&self) -> bool
pub fn set_scan_running(&mut self, running: bool)
pub fn get_scan_results(&self) -> &[ScanResultView]
pub fn update_scan_results(&mut self, results: Vec<ScanResultView>)
```

## 使用示例

```rust
use rabbit_models::config::{AppConfig, Language, Theme};
use rabbit_platform::config::load_config;

fn main() -> anyhow::Result<()> {
    let config = load_config()?;
    let mut vm = AppViewModel::new(config);

    // 更新全局配置
    vm.update_global(Language::Chinese, Theme::Dark, true, false, false, true)?;

    // 更新最后活动标签
    vm.update_last_tab(2)?;

    // 获取配置
    let cfg = vm.get_config();
    
    Ok(())
}
```

---

文档版本：1.0
创建日期：2026-04-23