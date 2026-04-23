# Rabbit 配置数据结构优化方案

## 问题背景

当前配置系统存在以下问题：

1. **配置保存分散**：ui_state.rs、app.rs、view_model.rs 多处独立 load+save
2. **时序问题**：配置更新和事件发送可能不同步
3. **内存缓存缺失**：每次读取都从文件加载
4. **统一入口缺失**：缺少集中的配置管理接口

---

## 统一配置管理方案

### 设计原则

1. **单一数据源**：AppConfig 是唯一内存数据源
2. **同步保存**：先保存到磁盘，再发送事件，无时序问题
3. **内存缓存**：配置常驻内存，避免频繁读文件
4. **统一入口**：ViewModel 负责所有配置操作

### 架构流程

```
UI 点击按钮
    │
    ▼
┌─────────────────────────────────────────────────┐
│  1. 收集输入框值                                 │
│  2. vm.update_and_save(config)             │ ← 同步更新+保存
│  3. send_event(UiEvent::ModuleStart)             │ ← 不带参数
└─────────────────────────────────────────────────┘
    │
    ▼
业务模块 handle_event
    │
    ▼
┌─────────────────────────────────────────────────┐
│  4. vm.get_config()                              │ ← 读取缓存配置
│  5. service.start(config.modules.ping)          │
└─────────────────────────────────────────────────┘
```

### AppViewModel 接口

```rust
pub struct AppViewModel {
    config: AppConfig,                    // 内存缓存
}

impl AppViewModel {
    pub fn new(config: AppConfig) -> Self {
        Self { config, .. }
    }

    pub fn get_config(&self) -> AppConfig {
        self.config.clone()
    }

    pub fn update_config(&mut self, config: AppConfig) {
        self.config = config;
    }

    pub fn update_and_save(&mut self, config: AppConfig) -> anyhow::Result<()> {
        self.config = config;
        save_config(&self.config)
    }

    pub fn save_settings(&mut self) -> anyhow::Result<()> {
        save_config(&self.config)
    }

    pub fn update_global(&mut self, language, theme, systray, top, autostart, autoupdate) -> anyhow::Result<()> {
        self.config.language = language;
        self.config.theme = theme;
        self.config.systray = systray;
        self.config.top = top;
        self.config.autostart = autostart;
        self.config.autoupdate = autoupdate;
        save_config(&self.config)
    }

    pub fn update_last_tab(&mut self, tab: usize) -> anyhow::Result<()> {
        self.config.last_active_tab = tab;
        save_config(&self.config)
    }
}
```

### 事件处理（无参数）

```rust
pub enum UiEvent {
    PingStart,
    PingStop,
    HttpToggle,
    HttpStart,
    HttpStop,
    TftpStart,
    TftpStop,
    ChatStart,
    ChatStop,
    ScanStart,
    ScanStop,
}

async fn handle_event(event: UiEvent, vm: &AppViewModel) {
    match event {
        UiEvent::PingStart => {
            let config = vm.get_config();
            ping_service.init(config.modules.ping.clone()).await;
            ping_service.start().await;
        }
        UiEvent::HttpToggle => {
            let config = vm.get_config();
            if config.modules.http.running {
                http_service.stop().await;
            } else {
                http_service.init(config.modules.http.clone()).await;
                http_service.start().await;
            }
        }
        // ...
    }
}
```

---

## 优点

| 方面 | 说明 |
|------|------|
| **无时序问题** | 同步保存完成后再发送事件 |
| **单一入口** | 只通过 ViewModel 保存 |
| **一致性** | 业务模块总是读取最新配置 |
| **性能** | 内存缓存避免频繁读文件 |
| **代码简化** | 移除分散的 load+save 逻辑 |

---

## 需要改造的点

| 当前实现 | 目标实现 |
|----------|----------|
| `ui_state.rs::set_systray()` | 移除，改为 `vm.update_global()` |
| 每个UI设置都 load+save | 只更新 ViewModel + 统一保存 |
| 事件带参数（port, options等） | 事件不带参数，读取 vm.get_config() |
| 分散的配置读取 | 统一从 ViewModel 获取 |

---

## 实施计划

### 阶段 1：添加统一接口（已完成）

- [x] `update_and_save()` 方法
- [x] `update_global()` 方法  
- [x] `update_last_tab()` 方法

### 阶段 2：移除重复保存逻辑

- [ ] 移除 `ui_state.rs` 中的独立 save
- [ ] 改为统一调用 ViewModel

### 阶段 3：简化事件参数

- [ ] 将 `UiEvent::HttpToggle { port, options }` 改为 `UiEvent::HttpToggle`
- [ ] 业务处理时读取 `vm.get_config()`

---

文档版本：1.0
创建日期：2026-04-23