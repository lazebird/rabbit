# Rabbit 配置局部更新方案

## 需求

配置更新/保存需要支持基于模块或局部更新，避免无关配置被覆盖/重置。

---

## 方案选择

### 方案对比

| 方案 | 优点 | 缺点 |
|------|------|------|
| 专用方法 | 清晰、类型安全 | 模块多时方法变多 |
| 字段更新器 | 灵活、单一方法 | 需闭包处理 |
| **section+map（推荐）** | 通用化、接口统一 | 需要 ConfigValue 类型 |

### 推荐：section+map 通用方法

尽量避免使用模块专有接口，使用通用的 section+map 方式实现配置更新。

---

## ConfigValue 类型定义

### 方案 A：使用 serde_json::Value（推荐）

优点：
- 无需定义新类型
- 自带序列化支持
- 社区广泛使用

缺点：
- 类型转换需要 match 处理

### 方案 B：自定义枚举

```rust
#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}
```

优点：
- 类型更明确
- 可扩展性强

缺点：
- 需要额外定义
- 序列化需手动实现

### 推荐实现

在 `rabbit-models/src/config.rs` 中定义：

```rust
use serde::{Deserialize, Serialize};

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

## 接口设计

### 1. update_section - 通用更新

```rust
pub fn update_section(&mut self, section: &str, updates: HashMap<&str, ConfigValue>) -> Result<()> {
    match section {
        "ping" => {
            let ping = &mut self.config.modules.ping;
            if let Some(v) = updates.get("target") {
                if let Some(s) = v.as_str() { ping.target = s.to_string(); }
            }
            if let Some(v) = updates.get("interval") {
                if let Some(n) = v.as_i64() { ping.interval = n as i32; }
            }
            // ... 其他字段
        }
        "http" => {
            let http = &mut self.config.modules.http;
            if let Some(v) = updates.get("port") {
                if let Some(n) = v.as_i64() { http.port = n as u16; }
            }
            // ... 其他字段
        }
        // ... 其他模块
        _ => {}
    }
    save_config(&self.config)
}
```

### 2. get_section - 通用读取

```rust
pub fn get_section(&self, section: &str) -> Option<HashMap<String, ConfigValue>> {
    match section {
        "ping" => {
            let ping = &self.config.modules.ping;
            let mut map = HashMap::new();
            map.insert("target".into(), ConfigValue::String(ping.target.clone()));
            map.insert("interval".into(), ConfigValue::Integer(ping.interval as i64));
            // ... 其他字段
            Some(map)
        }
        // ... 其他模块
        _ => None,
    }
}
```

### 3. get_value - 单值读取

```rust
pub fn get_value(&self, section: &str, key: &str) -> Option<ConfigValue> {
    self.get_section(section)?.get(key).cloned()
}
```

---

## 字段映射表

### ping 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `target` | ping.target | String |
| `interval` | ping.interval | i32 |
| `count` | ping.count | i32 |
| `stoponloss` | ping.stoponloss | bool |
| `taskbar` | ping.taskbar | bool |
| `running` | ping.running | bool |

### http 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `port` | http.port | u16 |
| `shell` | http.shell | bool |
| `autoindex` | http.autoindex | bool |
| `videoplay` | http.videoplay | bool |
| `dirs` | http.dirs | Vec\<String\> |
| `running` | http.running | bool |

### scan 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `start_ip` | scan.start_ip | String |
| `end_ip` | scan.end_ip | String |
| `filter` | scan.filter | bool |

### tftpd 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `port` | tftpd.port | u16 |
| `timeout` | tftpd.timeout | i32 |
| `maxretry` | tftpd.maxretry | i32 |
| `blksize` | tftpd.blksize | i32 |
| `qsize` | tftpd.qsize | i32 |
| `qtout` | tftpd.qtout | i32 |
| `override_conflicts` | tftpd.override_conflicts | bool |
| `fslog` | tftpd.fslog | bool |
| `work_dirs` | tftpd.work_dirs | Vec\<String\> |
| `working_dir_index` | tftpd.working_dir_index | Option\<usize\> |
| `running` | tftpd.running | bool |

### tftpc 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `server_addr` | tftpc.server_addr | String |
| `server_port` | tftpc.server_port | u16 |
| `local_path` | tftpc.local_path | String |
| `remote_file` | tftpc.remote_file | String |
| `timeout` | tftpc.timeout | i32 |
| `maxretry` | tftpc.maxretry | i32 |
| `blksize` | tftpc.blksize | i32 |

### plan 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `date` | plan.date | String |
| `time` | plan.time | String |
| `cycle` | plan.cycle | i32 |
| `unit` | plan.unit | String |
| `msg` | plan.msg | String |
| `override_conflicts` | plan.override_conflicts | bool |

### chat 模块

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `username` | chat.username | String |
| `port` | chat.port | u16 |
| `broadcast_addr` | chat.broadcast_addr | String |
| `running` | chat.running | bool |

### 全局配置

| ConfigKey | ConfigField | 类型 |
|-----------|-------------|------|
| `language` | language | Language |
| `theme` | theme | Theme |
| `systray` | systray | bool |
| `top` | top | bool |
| `autostart` | autostart | bool |
| `autoupdate` | autoupdate | bool |
| `last_active_tab` | last_active_tab | usize |
| `window_x` | window_x | Option\<i32\> |
| `window_y` | window_y | Option\<i32\> |
| `window_width` | window_width | Option\<i32\> |
| `window_height` | window_height | Option\<i32\> |

> **注意**：`window_*` 字段通过专门方法管理，不通过 section+map 接口。

---

## 特殊类型处理

### Vec\<String\> 数组类型

ConfigValue 的 `Array` 变体用于处理数组字段：

**update_section 中的写入**：
```rust
"http" => {
    let http = &mut self.config.modules.http;
    // dirs 是 Vec<String>
    if let Some(v) = updates.get("dirs") {
        if let Some(arr) = v.as_array() {
            http.dirs = arr.iter()
                .filter_map(|item| item.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
}
```

**get_section 中的读取**：
```rust
"http" => {
    let http = &self.config.modules.http;
    let mut map = HashMap::new();
    // ...
    map.insert("dirs".into(), ConfigValue::Array(
        http.dirs.iter().map(|s| ConfigValue::String(s.clone())).collect()
    ));
    Some(map)
}
```

### Option\<usize\> 类型

tftpd.working_dir_index 是 `Option<usize>`：

```rust
"tftpd" => {
    let tftpd = &mut self.config.modules.tftpd;
    // UI 使用 1-based 索引，配置使用 0-based
    if let Some(v) = updates.get("working_dir_index") {
        if let Some(n) = v.as_i64() {
            tftpd.working_dir_index = if n <= 0 { None } else { Some((n - 1) as usize) };
        }
    }
}
```

---

## 替代方案：字段更新器

对于复杂更新场景，可使用闭包：

```rust
pub fn update_with<F>(&mut self, updater: F) -> Result<()>
where
    F: FnOnce(&mut AppConfig),
{
    updater(&mut self.config);
    save_config(&self.config)
}
```

调用示例：
```rust
vm.update_with(|cfg| {
    cfg.modules.ping.target = "8.8.8.8".to_string();
    cfg.modules.ping.interval = 500;
})?;
```

---

## 实施计划

### 步骤 1：定义 ConfigValue 类型

- [ ] 在 `rabbit-models/src/config.rs` 添加 ConfigValue 枚举
- [ ] 实现 as_str/as_i64/as_bool 辅助方法
- [ ] 验证序列化/反序列���

### 步骤 2：实现 update_section

- [ ] ping 模块字段映射
- [ ] http 模块字段映射
- [ ] scan 模块字段映射
- [ ] tftpd 模块字段映射
- [ ] tftpc 模块字段映射
- [ ] plan 模块字段映射
- [ ] chat 模块字段映射

### 步骤 3：实现 get_section

- [ ] ping 模块读取
- [ ] http 模块读取
- [ ] 其他模块读取

### 步骤 4：验证

- [ ] 编译检查
- [ ] 单元测试
- [ ] 集成测试

---

## 相关文档

- [config-structure.md](./config-structure.md) - 整体架构设计
- [config-impl-plan.md](./config-impl-plan.md) - 详细实施计划
- [config-issues.md](./config-issues.md) - 遇到的问题记录

---

文档版本：1.1
更新日期：2026-04-23