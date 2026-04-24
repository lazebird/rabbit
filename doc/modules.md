# Rabbit 模块化结构文档

## 项目架构

```
┌─────────────────────────────────────────┐
│           Presentation Layer             │
│   View (FLTK UI) ← ViewModel (State)     │
├─────────────────────────────────────────┤
│           Business Layer               │
│   PingService / HttpService / ...       │
├─────────────────────────────────────────┤
│           Data Layer                  │
│   Models + Repository (Persistence)    │
├─────────────────────────────────────────┤
│        Infrastructure Layer          │
│   WindowsPlatform / LinuxPlatform    │
└─────────────────────────────────────────┘
```

## 模块结构

```
rabbit/
├── crates/
│   ├── rabbit-app/       # 主入口 + UI
│   │   ├── src/
│   │   │   ├── app.rs        # 主应用入口
│   │   │   ├── view_model.rs # 统一配置管理
│   │   │   ├── ui/          # FLTK 界面
│   │   │   │   ├── mod.rs
│   │   │   │   ├── ping_tab.rs
│   │   │   │   ├── http_tab.rs
│   │   │   │   ├── settings_tab.rs
│   │   │   │   └── ...
│   │   │   └── ui_state.rs   # UI 状态管理
│   │   └── tests/
│   │
│   ├── rabbit-core/       # 业务服务
│   │   ├── src/
│   │   │   ├── ping.rs
│   │   │   ├── http.rs
│   │   │   ├── tftp.rs
│   │   │   ├── scan.rs
│   │   │   ├── chat.rs
│   │   │   └── plan.rs
│   │
│   ├── rabbit-models/    # 数据模型
│   │   ├── src/
│   │   │   ├── config.rs   # 配置模型
│   │   │   ├── ping.rs
│   │   │   ├── http.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   └── rabbit-platform/  # 平台适配
│       ├── src/
│       │   ├── config.rs  # 配置持久化
│       │   ├── lib.rs
│       │   └── platform.rs
│       └── Cargo.toml
│
├── doc/
│   ├── architecture.md
│   ├── config-structure.md
│   └── progress.md
│
├── tests/
│   └── integration_tests.rs
│
└── Cargo.toml
```

## 模块职责

### rabbit-app (Presentation + Business)

| 模块 | 职责 |
|------|------|
| app.rs | 主入口、事件处理、生命周期管理 |
| view_model.rs | 统一配置管理（使用 HashMap） |
| ui/ | FLTK 界面展示 |
| ui_state.rs | UI 状态同步（使用 insert） |

### rabbit-core (Business Services)

| 服务 | 职责 |
|------|------|
| PingService | ICMP Ping 功能 |
| HttpService | HTTP 文件服务器 |
| TftpService | TFTP 传输服务 |
| ScanService | IP 扫描 |
| ChatService | 局域网聊天 |
| PlanService | 定时提醒 |

### rabbit-models (Data Layer)

| 模型 | 职责 |
|------|------|
| AppConfig | 应用配置结构（仅含 modules） |
| ModuleConfigs | 所有模块配置（HashMap 方式） |
| ConfigValue | 通用的配置值类型 |
| HttpServerConfig | HTTP 服务运行时配置 |
| TftpServerConfig | TFTP 服务运行时配置 |

### rabbit-platform (Infrastructure)

| 模块 | 职责 |
|------|------|
| config.rs | 配置加载/保存 |
| platform.rs | 平台特定功能 |

---

## 模块依赖关系

```rust
// rabbit-app
├── rabbit_models   // 数据模型
├── rabbit_core  // 业务服务
└── rabbit_platform // 平台适配

// rabbit_core
└── rabbit_models

// rabbit_platform
└── rabbit_models
```

---

## 配置访问方式

所有模块配置统一使用 HashMap 方式访问：

```rust
// 读取
config.modules.get_string("global", "language")
config.modules.get_integer("http", "port")
config.modules.get_bool("ping", "stoponloss")

// 写入
config.modules.insert("global", "systray", ConfigValue::Boolean(true));
```

---

文档版本：2.0
创建日期：2026-04-16
更新日期：2026-04-23