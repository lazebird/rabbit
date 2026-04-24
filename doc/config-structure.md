# Rabbit 配置数据结构方案

## 概述

Rabbit 应用使用统一的配置管理方案，所有配置通过 `ModuleConfigs` 的 HashMap 结构存储，支持灵活的键值访问。

---

## 数据结构

### AppConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub modules: ModuleConfigs,
}
```

### ModuleConfigs

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfigs {
    pub global: HashMap<String, ConfigValue>,    // 全局配置
    pub ping: HashMap<String, ConfigValue>,      // Ping 模块
    pub scan: HashMap<String, ConfigValue>,     // 扫描模块
    pub http: HashMap<String, ConfigValue>,     // HTTP 服务模块
    pub tftpd: HashMap<String, ConfigValue>,   // TFTP 服务模块
    pub tftpc: HashMap<String, ConfigValue>,   // TFTP 客户端模块
    pub plan: HashMap<String, ConfigValue>,     // 计划任务模块
    pub chat: HashMap<String, ConfigValue>,     // 局域网聊天模块
}
```

### ConfigValue

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}
```

---

## 统一接口

### 读取配置

```rust
// 读取字符串
config.modules.get_string("global", "language")

// 读取整数
config.modules.get_integer("http", "port")

// 读取布尔值
config.modules.get_bool("ping", "stoponloss")

// 读取数组
config.modules.get_array("http", "dirs")
```

### 写入配置

```rust
// 写入值
config.modules.insert("global", "systray", ConfigValue::Boolean(true));
config.modules.insert("http", "port", ConfigValue::Integer(8080));
```

---

## 配置文件

### 文件位置

- **Linux**: `~/.config/rabbit/rabbit.toml`
- **Windows**: `%APPDATA%/rabbit/rabbit.toml`

### 文件格式

```toml
[modules.global]
language = "System"
theme = "System"
systray = true
top = false
autostart = false
autoupdate = true
last_active_tab = 0
window_x = 100
window_y = 100
window_width = 800
window_height = 600

[modules.ping]
target = "1.1.1.1"
interval = 1000
count = -1
stoponloss = false
taskbar = true
log = ""
running = false

[modules.scan]
start_ip = "192.168.1.1"
end_ip = "254"
filter = true

[modules.http]
port = 8000
shell = false
autoindex = true
videoplay = true
dirs = []
running = false

[modules.tftpd]
port = 69
timeout = 200
maxretry = 10
blksize = 512
qsize = 2000
qtout = 1000
override_conflicts = false
fslog = false
work_dirs = []
working_dir_index = 0
running = false

[modules.tftpc]
server_addr = "127.0.0.1"
server_port = 69
local_path = ""
remote_file = ""
timeout = 200
maxretry = 10
blksize = 1024

[modules.plan]
date = ""
time = ""
cycle = 0
unit = "minute"
msg = ""
override_conflicts = false

[modules.chat]
username = "User@PC"
port = 1314
broadcast_addr = "255.255.255.255"
running = false
```

---

## 字段映射表

### global (全局配置)

| Key | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `language` | String | "System" | 语言: System/English/Chinese |
| `theme` | String | "System" | 主题: System/Light/Dark |
| `systray` | Boolean | true | 托盘图标 |
| `top` | Boolean | false | 窗口置顶 |
| `autostart` | Boolean | false | 开机自启 |
| `autoupdate` | Boolean | true | 自动更新 |
| `last_active_tab` | Integer | 0 | 最后活动标签页 |
| `window_x` | Integer | 100 | 窗口 X 坐标 |
| `window_y` | Integer | 100 | 窗口 Y 坐标 |
| `window_width` | Integer | 800 | 窗口宽度 |
| `window_height` | Integer | 600 | 窗口高度 |

### ping (Ping 模块)

| Key | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `target` | String | "1.1.1.1" | 目标地址 |
| `interval` | Integer | 1000 | 间隔(ms) |
| `count` | Integer | -1 | 次数(-1无限) |
| `stoponloss` | Boolean | false | 丢包停止 |
| `taskbar` | Boolean | true | 任务栏状态 |
| `log` | String | "" | 日志 |
| `running` | Boolean | false | 运行状态 |

### http (HTTP 服务)

| Key | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `port` | Integer | 8000 | 端口 |
| `shell` | Boolean | false | Shell 访问 |
| `autoindex` | Boolean | true | 目录列表 |
| `videoplay` | Boolean | true | 视频播放 |
| `dirs` | Array | [] | 共享目录 |
| `running` | Boolean | false | 运行状态 |

### scan (IP 扫描)

| Key | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `start_ip` | String | "192.168.1.1" | 起始 IP |
| `end_ip` | String | "254" | 结束 IP |
| `filter` | Boolean | true | 过滤条件 |

### tftpd (TFTP 服务)

| Key | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `port` | Integer | 69 | 端口 |
| `timeout` | Integer | 200 | 超时(ms) |
| `maxretry` | Integer | 10 | 最大重试 |
| `blksize` | Integer | 512 | 块大小 |
| `qsize` | Integer | 2000 | 队列大小 |
| `qtout` | Integer | 1000 | 队列超时 |
| `override_conflicts` | Boolean | false | 覆盖冲突 |
| `fslog` | Boolean | false | 文件日志 |
| `work_dirs` | Array | [] | 工作目录 |
| `working_dir_index` | Integer | 0 | 当前目录索引 |
| `running` | Boolean | false | 运行状态 |

### tftpc (TFTP 客户端)

| Key | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `server_addr` | String | "127.0.0.1" | 服务器地址 |
| `server_port` | Integer | 69 | 服务器端口 |
| `local_path` | String | "" | 本地路径 |
| `remote_file` | String | "" | 远程文件 |
| `timeout` | Integer | 200 | 超时(ms) |
| `maxretry` | Integer | 10 | 最大重试 |
| `blksize` | Integer | 1024 | 块大小 |

### plan (计划任务)

| Key | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `date` | String | "" | 日期 |
| `time` | String | "" | 时间 |
| `cycle` | Integer | 0 | 周期 |
| `unit` | String | "minute" | 单位 |
| `msg` | String | "" | 消息 |
| `override_conflicts` | Boolean | false | 覆盖冲突 |

### chat (局域网聊天)

| Key | 类型 | 默认值 | 说明 |
|-----|------|--------|------|
| `username` | String | "User@PC" | 用户名 |
| `port` | Integer | 1314 | 端口 |
| `broadcast_addr` | String | "255.255.255.255" | 广播地址 |
| `running` | Boolean | false | 运行状态 |

---

## 加载/保存接口

### rabbit-platform

```rust
// 加载配置
pub fn load_config() -> Result<AppConfig>

// 保存配置
pub fn save_config(config: &AppConfig) -> Result<()>

// 更新配置
pub fn update_config<F>(modifier: F) -> Result<()>
where
    F: FnOnce(&mut AppConfig),
```

---

## 实施状态

- [x] ConfigValue 枚举定义
- [x] ModuleConfigs HashMap 结构
- [x] global 模块统一管理全局配置
- [x] 统一访问接口 (get_string/get_integer/get_bool/get_array/insert)
- [x] 配置文件统一为 rabbit.toml

---

文档版本：4.0
更新日期：2026-04-23
