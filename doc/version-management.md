# 版本管理与自动升级方案

## 1. 概述

Rabbit 使用统一版本管理方案进行版本检查和自动升级。方案特点：
- **统一版本号**：所有平台共享同一语义化版本号，无冗余
- **多平台支持**：Windows/Linux x64/Linux arm64/macOS
- **自动检查更新**：启动时后台检查（可配置）
- **自动下载升级**：用户确认后自动下载、校验、替换
- **手动检查**：点击版本信息即时检查

版本信息存储在远程 Git 仓库的 `release/versions.json` 文件中。

## 2. 远程版本文件格式

### 2.1 文件位置

```
https://raw.githubusercontent.com/lazebird/rabbit/rewrite/release/versions.json
```

### 2.2 JSON 格式

```json
{
  "version": "1.2.0",
  "release_date": "2026/04/20",
  "release_notes": "Bug fixes and performance improvements",
  "platforms": {
    "windows-x64": {
      "sha256": "abc123def456...",
      "size": 5242880,
      "url": "https://codeup.aliyun.com/.../release/rabbit-1.2.0-windows-x64.exe"
    },
    "linux-x64": {
      "sha256": "789ghi012jkl...",
      "size": 4718592,
      "url": "https://codeup.aliyun.com/.../release/rabbit-1.2.0-linux-x64"
    },
    "linux-arm64": {
      "sha256": "345mno678pqr...",
      "size": 4456448,
      "url": "https://codeup.aliyun.com/.../release/rabbit-1.2.0-linux-arm64"
    },
    "macos-x64": {
      "sha256": "901stu234vwx...",
      "size": 5767168,
      "url": "https://codeup.aliyun.com/.../release/rabbit-1.2.0-macos-x64"
    },
    "macos-arm64": {
      "sha256": "567yza890bcd...",
      "size": 5505024,
      "url": "https://codeup.aliyun.com/.../release/rabbit-1.2.0-macos-arm64"
    }
  }
}
```

### 2.3 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| version | string | 统一语义化版本号 (semver)，所有平台一致 |
| release_date | string | 发布日期 (YYYY/MM/DD) |
| release_notes | string | 更新说明 |
| platforms | object | 各平台下载信息 |
| platforms.{platform}.sha256 | string | 文件 SHA256 校验值 (hex) |
| platforms.{platform}.size | number | 文件大小 (字节) |
| platforms.{platform}.url | string | 下载 URL |

> **设计说明**：版本号只在全局定义一次，各平台不再重复。这假设所有平台同步发布同一版本，适合 Rabbit 的发布模式。

### 2.4 平台标识

| 标识 | 说明 |
|------|------|
| windows-x64 | Windows x64 |
| linux-x64 | Linux x64 (glibc) |
| linux-arm64 | Linux ARM64 |
| macos-x64 | macOS Intel |
| macos-arm64 | macOS Apple Silicon |

## 3. 客户端实现

### 3.1 本地版本

从 `Cargo.toml` 自动注入：

```rust
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
```

### 3.2 版本检查流程

```
启动应用 (若启用自动检查)
    |
    v
后台获取 versions.json
    |
    v
解析 JSON，取全局 version 字段
    |
    v
Semver 比较: remote > local ?
    |
   是 ────────────────── 否 ──> 静默返回
    |
    v
检查是否已忽略该版本
    |
   否 ────────────────── 是 ──> 静默返回
    |
    v
弹窗提示用户
    |
    v
用户选择:
  ├─ Download & Update ──> 下载 -> 校验 -> 替换 -> 提示重启
  ├─ Later ───────────────> 取消
  └─ Skip This Version ──> 加入忽略列表，不再提示
```

### 3.3 版本比较 (Semver)

```rust
use semver::Version;

fn is_newer(remote: &str, current: &str) -> Option<bool> {
    let remote_ver = Version::parse(remote).ok()?;
    let current_ver = Version::parse(current).ok()?;
    Some(remote_ver > current_ver)
}
```

### 3.4 当前平台检测

```rust
fn current_platform() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "windows-x64";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x64";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "linux-arm64";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "macos-x64";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "macos-arm64";
    "unknown"
}
```

## 4. 统一配置文件

### 4.1 配置位置

升级配置合并到统一的 `config.toml` 中，与其他模块配置共存：

```
~/.config/rabbit/config.toml         (Linux)
%APPDATA%\rabbit\config.toml         (Windows)
~/Library/Application Support/rabbit/config.toml (macOS)
```

### 4.2 配置格式示例

```toml
# 全局设置
language = "System"
theme = "System"
systray = true
top = false
autostart = false
autoupdate = true         # 启动时自动检查更新
last_active_tab = 0

# 升级管理配置
[upgrade]
ignored_versions = []

# 其他模块配置...
[modules.ping]
target = "1.1.1.1"
...
```

### 4.3 配置结构

升级相关字段合并到 `AppConfig` 结构体中：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Language,
    pub theme: Theme,
    pub systray: bool,
    pub top: bool,
    pub autostart: bool,
    pub autoupdate: bool,          // 启动时自动检查更新
    pub last_active_tab: usize,
    pub upgrade: UpgradeConfig,    // 升级管理配置
    pub modules: ModuleConfigs,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpgradeConfig {
    /// 已忽略的版本
    pub ignored_versions: Vec<String>,
}

impl UpgradeConfig {
    /// 版本是否已忽略
    pub fn is_ignored(&self, version: &str) -> bool {
        self.ignored_versions.iter().any(|v| v == version)
    }

    /// 忽略版本
    pub fn ignore_version(&mut self, version: &str) {
        if !self.is_ignored(version) {
            self.ignored_versions.push(version.to_string());
        }
    }
}
```

## 5. 跨平台实现

### 5.1 HTTP 请求

使用 `ureq` 库进行同步 HTTP 请求（无需子进程）：

```toml
[dependencies]
ureq = { version = "2", features = ["tls"] }
```

```rust
fn fetch_versions(url: &str) -> Result<String, String> {
    ureq::get(url)
        .timeout(std::time::Duration::from_secs(15))
        .call()
        .map_err(|e| format!("HTTP error: {}", e))?
        .into_string()
        .map_err(|e| format!("Read error: {}", e))
}
```

### 5.2 文件下载

```rust
use std::fs::File;
use std::io::{self, Write};

fn download_file(url: &str, dest: &Path) -> Result<u64, String> {
    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(300))
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;

    let mut file = File::create(dest)
        .map_err(|e| format!("Create file failed: {}", e))?;

    let mut reader = resp.into_reader();
    let mut buffer = Vec::new();
    io::copy(&mut reader, &mut buffer)
        .map_err(|e| format!("Write failed: {}", e))?;

    file.write_all(&buffer)
        .map_err(|e| format!("Write failed: {}", e))?;

    Ok(buffer.len() as u64)
}
```

### 5.3 SHA256 校验

```rust
use sha2::{Sha256, Digest};
use std::fs;

fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    let data = fs::read(path)
        .map_err(|e| format!("Read file failed: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = format!("{:x}", hasher.finalize());

    if result == expected.to_lowercase() {
        Ok(())
    } else {
        Err(format!("SHA256 mismatch: expected {}, got {}", expected, result))
    }
}
```

### 5.4 版本替换策略

各平台替换方式不同：

| 平台 | 替换方式 |
|------|----------|
| Windows | 下载 .exe 到临时目录，创建 .bat 脚本替换旧文件 |
| Linux/macOS | 下载新二进制文件，`chmod +x`，替换原文件 |

**Windows 替换流程:**
1. 下载新版本到 `%TEMP%\rabbit_update\`
2. 校验 SHA256
3. 创建 `update.bat`: 等待旧进程退出 -> 复制新文件覆盖旧文件 -> 启动新版本
4. 执行 .bat 并退出当前进程

```batch
@echo off
timeout /t 2 /nobreak >nul
copy /y "%TEMP%\rabbit_update\rabbit.exe" "%CURRENT_DIR%\rabbit.exe"
start "" "%CURRENT_DIR%\rabbit.exe"
rmdir /s /q "%TEMP%\rabbit_update"
```

**Linux/macOS 替换流程:**
1. 下载新版本到 `/tmp/rabbit_update/`
2. 校验 SHA256
3. 创建 `update.sh`: 等待旧进程退出 -> 复制新文件 (cp -f) -> 设置执行权限 -> 启动新版本
4. 执行 .sh 并退出当前进程

```bash
#!/bin/bash
sleep 2
cp -f /tmp/rabbit_update/rabbit "$(dirname "$0")/rabbit"
chmod +x "$(dirname "$0")/rabbit"
"$(dirname "$0")/rabbit" &
rm -rf /tmp/rabbit_update
```

## 6. UI 交互

### 6.1 设置页布局

```
[Language: English/中文/System]

[Tray] [Top] [AutoStart] [AutoUpdate v]

sRabbit 0.1.0                          [Home] [Profile] [Help]

┌─────────────────────────────────────────────────┐
│ 输出区域 (版本检查结果显示在这里)                 │
│                                                 │
└─────────────────────────────────────────────────┘
```

### 6.2 更新提示弹窗

```
┌─────────────────────────────────────────┐
│           Update Available              │
├─────────────────────────────────────────┤
│                                         │
│  Current: 0.1.0    Latest: 1.2.0        │
│                                         │
│  Release Notes:                         │
│  Bug fixes and performance              │
│  improvements...                        │
│                                         │
│  Size: 5.0 MB                           │
│                                         │
│     [Download & Update]  [Later]        │
│               [Skip This Version]       │
└─────────────────────────────────────────┘
```

### 6.3 下载进度

```
┌─────────────────────────────────────────┐
│           Downloading Update            │
├─────────────────────────────────────────┤
│                                         │
│  ████████████░░░░░░░░░░  45%            │
│  2.2 MB / 5.0 MB                        │
│  ETA: 12 seconds                        │
│                                         │
│              [Cancel]                   │
└─────────────────────────────────────────┘
```

### 6.4 手动检查更新

用户点击版本信息时:
1. 复制版本号到剪贴板
2. 立即发起版本检查
3. 显示结果

## 7. 模块架构

```
crates/app/
├── src/
│   ├── upgrade/
│   │   ├── mod.rs           # 模块入口
│   │   ├── config.rs        # 升级配置管理
│   │   ├── checker.rs       # 版本检查
│   │   ├── downloader.rs    # 文件下载
│   │   ├── installer.rs     # 平台安装逻辑
│   │   └── models.rs        # 版本数据模型
│   └── ...
```

### 7.1 数据模型

```rust
/// 远程版本信息 (统一版本方案)
#[derive(Deserialize)]
pub struct VersionsManifest {
    /// 统一版本号 (所有平台一致)
    pub version: String,
    /// 发布日期
    pub release_date: String,
    /// 更新说明
    pub release_notes: String,
    /// 各平台信息
    pub platforms: HashMap<String, PlatformInfo>,
}

#[derive(Deserialize)]
pub struct PlatformInfo {
    /// 文件 SHA256
    pub sha256: String,
    /// 文件大小 (字节)
    pub size: u64,
    /// 下载 URL
    pub url: String,
}

/// 更新状态
pub enum UpdateStatus {
    /// 已是最新
    UpToDate,
    /// 有可用更新
    UpdateAvailable(VersionsManifest, PlatformInfo),
    /// 检查出错
    CheckError(String),
}
```

### 7.2 事件系统

```rust
pub enum UiEvent {
    // ... existing events ...
    UpgradeCheck,           // 手动触发检查
    UpgradeConfirm,         // 用户确认更新
    UpgradeCancel,          // 用户取消更新
    UpgradeSkipVersion(String), // 跳过某版本
}
```

## 8. 相关 URL

| 用途 | URL |
|------|-----|
| 版本信息 | `https://raw.githubusercontent.com/lazebird/rabbit/rewrite/release/versions.json` |
| 下载页面 | `https://github.com/lazebird/rabbit/tree/rewrite/release` |
| 使用手册 | `https://github.com/lazebird/rabbit/blob/rewrite/doc/manual.md` |

## 9. 依赖新增

```toml
[dependencies]
ureq = { version = "2", features = ["tls"] }  # 同步 HTTP 客户端
sha2 = "0.10"                                  # SHA256 校验
semver = "1"                                   # 语义化版本号
serde_json = { workspace = true }              # JSON 解析
chrono = { workspace = true }                  # 时间处理
```

## 10. 安全考虑

1. **SHA256 校验**: 下载后校验文件完整性，防止篡改
2. **HTTPS**: 所有 HTTP 请求使用 TLS
3. **权限检查**: Linux/macOS 写入前检查文件权限
4. **原子替换**: 尽可能使用原子操作减少失败风险
5. **临时文件清理**: 升级完成后清理临时目录

## 11. 发布脚本

### 11.1 使用方法

```bash
./release.sh <version>
# 示例:
./release.sh 1.2.0
```

### 11.2 脚本功能

1. 编译当前平台的 release 版本二进制文件
2. 复制到 `release/` 目录，命名为 `rabbit-{version}-{platform}`
3. 计算 SHA256 校验值和文件大小
4. 创建或更新 `release/versions.json`
5. 生成 `release/RELEASE_NOTES_{version}.md` 模板

### 11.3 输出示例

```
[INFO] Rabbit Release Script
[INFO] Version: 1.2.0
[INFO] Platform: linux-x64
[INFO] Building release version 1.2.0...
[INFO] Running cargo build --release...
[INFO] Build successful: .../target/release/rabbit
[INFO] Binary copied to release/rabbit-1.2.0-linux-x64
[INFO] SHA256: abc123...
[INFO] Size: 3127304 bytes (2.9 MB)
[INFO] Updating release/versions.json...
[INFO] Release notes generated: release/RELEASE_NOTES_1.2.0.md
[INFO] Release process complete!
```

### 11.4 多平台发布

需要在各平台分别运行脚本，每次运行会添加当前平台的信息到 `versions.json`：

```bash
# Windows 上运行
./release.sh 1.2.0

# Linux x64 上运行
./release.sh 1.2.0

# Linux arm64 上运行
./release.sh 1.2.0

# macOS 上运行
./release.sh 1.2.0
```

### 11.5 依赖

- `cargo` (Rust 构建工具)
- `sha256sum` 或 `shasum` (校验值计算)
- `python3` (JSON 操作)
- `bc` (文件大小计算)

## 12. 错误处理

当 `versions.json` 获取失败时：
- 显示错误信息（网络错误/服务器错误）
- 提示用户检查网络连接
- 提供手动访问下载页面的链接

## 13. 版本方案对比

| 方案 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| 统一版本 (当前) | 简洁，无冗余 | 所有平台需同步发布 | Rabbit 个人工具 |
| 平台独立版本 | 各平台可独立发布 | 版本号冗余/不一致 | 大型项目多团队 |
