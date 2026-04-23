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
5. **通用接口**：使用 section+map 方式，避免模块专有接口膨胀

### 架构流程

```
UI 点击按钮
    │
    ▼
┌─────────────────────────────────────────────────┐
│  1. 收集输入框值                                 │
│  2. vm.update_config(config)              │ ← 同步更新+保存
│  3. send_event(UiEvent::ModuleToggle)           │ ← 不带参数
└─────────────────────────────────────────────────┘
    │
    ▼
业务模块 handle_event
    │
    ▼
┌─────────────────────────────────────────────────┐
│  4. vm.get_config()                            │ ← 读取缓存配置
│  5. config.modules.get_string("ping", "target")│ ← Map 方式访问
│  6. service.start(ping_config)                 │
└─────────────────────────────────────────────────┘
```

---

## ConfigValue 类型定义

在 `rabbit-models/src/config.rs` 中定义：

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}
```

---

## ModuleConfigs HashMap 结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfigs {
    pub ping: HashMap<String, ConfigValue>,
    pub scan: HashMap<String, ConfigValue>,
    pub http: HashMap<String, ConfigValue>,
    pub tftpd: HashMap<String, ConfigValue>,
    pub tftpc: HashMap<String, ConfigValue>,
    pub plan: HashMap<String, ConfigValue>,
    pub chat: HashMap<String, ConfigValue>,
}

impl ModuleConfigs {
    pub fn get_string(&self, module: &str, key: &str) -> Option<String> {
        self.get_map(module)?.get(key)?.as_str().map(|s| s.to_string())
    }

    pub fn get_integer(&self, module: &str, key: &str) -> Option<i64> {
        self.get_map(module)?.get(key)?.as_integer()
    }

    pub fn get_bool(&self, module: &str, key: &str) -> Option<bool> {
        self.get_map(module)?.get(key)?.as_bool()
    }

    pub fn get_array(&self, module: &str, key: &str) -> Option<Vec<String>> {
        self.get_map(module)?.get(key)?.as_string_array()
    }

    pub fn insert(&mut self, module: &str, key: &str, value: ConfigValue) {
        if let Some(map) = self.get_map_mut(module) {
            map.insert(key.to_string(), value);
        }
    }

    fn get_map(&self, module: &str) -> Option<&HashMap<String, ConfigValue>> { ... }
    fn get_map_mut(&mut self, module: &str) -> Option<&mut HashMap<String, ConfigValue>> { ... }
}
```

---

## ���段映射表

### ping 模块

| ConfigKey | 类型 | 默认值 |
|-----------|------|--------|
| `target` | String | "1.1.1.1" |
| `interval` | Integer | 1000 |
| `count` | Integer | -1 |
| `stoponloss` | Boolean | false |
| `taskbar` | Boolean | true |
| `log` | String | "" |
| `running` | Boolean | false |

### http 模块

| ConfigKey | 类型 | 默认值 |
|-----------|------|--------|
| `port` | Integer | 8000 |
| `shell` | Boolean | false |
| `autoindex` | Boolean | true |
| `videoplay` | Boolean | true |
| `dirs` | Array | [] |
| `running` | Boolean | false |

### scan 模块

| ConfigKey | 类型 | 默认值 |
|-----------|------|--------|
| `start_ip` | String | "192.168.1.1" |
| `end_ip` | String | "254" |
| `filter` | Boolean | true |

### tftpd 模块

| ConfigKey | 类型 | 默认值 |
|-----------|------|--------|
| `port` | Integer | 69 |
| `timeout` | Integer | 200 |
| `maxretry` | Integer | 10 |
| `blksize` | Integer | 512 |
| `qsize` | Integer | 2000 |
| `qtout` | Integer | 1000 |
| `override_conflicts` | Boolean | false |
| `fslog` | Boolean | false |
| `work_dirs` | Array | [] |
| `working_dir_index` | Integer | 0 |
| `running` | Boolean | false |

### tftpc 模块

| ConfigKey | 类型 | 默认值 |
|-----------|------|--------|
| `server_addr` | String | "127.0.0.1" |
| `server_port` | Integer | 69 |
| `local_path` | String | "" |
| `remote_file` | String | "" |
| `timeout` | Integer | 200 |
| `maxretry` | Integer | 10 |
| `blksize` | Integer | 1024 |

### plan 模块

| ConfigKey | 类型 | 默认值 |
|-----------|------|--------|
| `date` | String | "" |
| `time` | String | "" |
| `cycle` | Integer | 0 |
| `unit` | String | "minute" |
| `msg` | String | "" |
| `override_conflicts` | Boolean | false |

### chat 模块

| ConfigKey | 类型 | 默认值 |
|-----------|------|--------|
| `username` | String | "User@PC" |
| `port` | Integer | 1314 |
| `broadcast_addr` | String | "255.255.255.255" |
| `running` | Boolean | false |

---

## UiEvent 简化事件

```rust
pub enum UiEvent {
    ModuleToggle { module: String, running: bool },
    // ...
}

async fn handle_event(event: UiEvent, vm: &AppViewModel) {
    match event {
        UiEvent::ModuleToggle { module, running } => {
            config.modules.insert(&module, "running", ConfigValue::Boolean(running));
            vm.update_config(config);
            // 业务模块从 vm.get_config() 读取最新配置
        }
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
| **通用化** | section+map 避免模块专有接口膨胀 |

---

## 实施状态

### ✅ 已完成

- [x] ConfigValue 枚举定义（String, Integer, Boolean, Array）
- [x] ModuleConfigs 改用 HashMap
- [x] get_string/get_integer/get_bool/get_array/insert 方法
- [x] app.rs 使用 Map 方式读取配置
- [x] ui_state.rs 使用 insert 方法保存配置
- [x] defaults.rs UI 默认值使用 Map 方式
- [x] HttpServerConfig/TftpServerConfig/TftpClientConfig From impl

### ⏳ 待完成

- [ ] 文档完善
- [ ] 集成测试

---

文档版本：3.0
更新日期：2026-04-23