# 配置优化详细实施计划

## 概述

本文档详细描述配置系统优化的具体实施步骤，按阶段划分，每个阶段包含具体任务和验收标准。

### 方案选择

| 方案 | 说明 | 状态 |
|------|------|------|
| 差异化结构体 | PingConfig, HttpConfig 等独立结构 | ❌ 已移除 |
| **HashMap 统一** | 所有模块使用 HashMap\<String, ConfigValue\> | ✅ 已完成 |

---

## 任务清单汇总

### 阶段 1：ConfigValue 定义 ✅

- [x] 定义 ConfigValue 枚举（String, Integer, Boolean, Array）
- [x] 实现辅助方法（as_str, as_i64, as_bool, as_array, as_string_array）
- [x] 导出 ConfigValue
- [x] 验证序列化/反序列化

### 阶段 2：ModuleConfigs HashMap 改造 ✅

- [x] 将 ping/scan/http/tftpd/tftpc/plan/chat 从结构体改为 HashMap
- [x] 实现 get_string/get_integer/get_bool/get_array 方法
- [x] 实现 insert 方法
- [x] 定义各模块默认配置
- [x] 编译验证

### 阶段 3：代码迁移 ✅

- [x] app.rs 使用 Map 方式读取配置
- [x] ui_state.rs 使用 insert 方法保存配置
- [x] defaults.rs UI 默认值使用 Map 方式
- [x] HttpServerConfig/TftpServerConfig/TftpClientConfig 实现 From impl
- [x] 编译通过

### 阶段 4：验证测试 ✅

- [x] cargo build --release 编译通过
- [x] cargo test 测试通过

---

## 详细实现

### 1. ConfigValue 枚举

```rust
// rabbit-models/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    pub fn as_str(&self) -> Option<&str> { ... }
    pub fn as_i64(&self) -> Option<i64> { ... }
    pub fn as_integer(&self) -> Option<i64> { ... }
    pub fn as_bool(&self) -> Option<bool> { ... }
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> { ... }
    pub fn as_string_array(&self) -> Option<Vec<String>> { ... }
}
```

### 2. ModuleConfigs HashMap

```rust
// rabbit-models/src/config.rs
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
    pub fn get_string(&self, module: &str, key: &str) -> Option<String> { ... }
    pub fn get_integer(&self, module: &str, key: &str) -> Option<i64> { ... }
    pub fn get_bool(&self, module: &str, key: &str) -> Option<bool> { ... }
    pub fn get_array(&self, module: &str, key: &str) -> Option<Vec<String>> { ... }
    pub fn insert(&mut self, module: &str, key: &str, value: ConfigValue) { ... }
}
```

### 3. 服务配置转换

```rust
// rabbit-models/src/http.rs
impl From<&ModuleConfigs> for HttpServerConfig {
    fn from(modules: &ModuleConfigs) -> Self {
        Self {
            enabled: false,
            port: modules.get_integer("http", "port").unwrap_or(8000) as u16,
            root_path: modules.get_array("http", "dirs")
                .and_then(|dirs| dirs.first().cloned())
                .unwrap_or_default(),
            allow_upload: false,
            allow_delete: false,
            shell: modules.get_bool("http", "shell").unwrap_or(false),
            auto_index: modules.get_bool("http", "autoindex").unwrap_or(true),
            video_play: modules.get_bool("http", "videoplay").unwrap_or(true),
        }
    }
}
```

### 4. 业务代码使用示例

```rust
// 读取配置
let target = config.modules.get_string("ping", "target").unwrap_or_default();
let port = config.modules.get_integer("http", "port").unwrap_or(8000) as u16;
let running = config.modules.get_bool("http", "running").unwrap_or(false);

// 保存配置
config.modules.insert("http", "port", ConfigValue::Integer(8080));
config.modules.insert("http", "running", ConfigValue::Boolean(true));
```

---

## 编译验证

```bash
cargo build --release
# ✅ Finished `release` profile [optimized] target(s) in 27.57s
```

---

## 相关文档

- [config-structure.md](./config-structure.md) - 整体架构设计
- [config-issues.md](./config-issues.md) - 遇到的问题记录

---

文档版本：3.0
更新日期：2026-04-23