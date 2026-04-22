# 跨平台提权方案

## 概述

本项目需要在启动时获取管理员权限，用于绑定 TFTP 默认端口 69。本文档描述跨平台提权的实现方案。

## 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                     main.rs                             │
│  ensure_elevated() ───────────────────────────────────┐ │
└───────────────────────────────────────────────────────┘ │
                                                          ▼
┌─────────────────────────────────────────────────────────┐
│                   elevation.rs                          │
├─────────────────────────────────────────────────────────┤
│  ensure_elevated()                                      │
│    ├── is_elevated()      // 检查当前权限               │
│    ├── restart_with_elevation()  // 提权重启            │
│    └── show_error_dialog()  // 错误提示                 │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│               elevated-command crate                    │
│  (跨平台提权库)                                          │
│  - Windows: UAC                                         │
│  - Linux: pkexec (polkit)                               │
└─────────────────────────────────────────────────────────┘
```

## 平台支持

| 平台    | 提权方式                   | 依赖                      |
| ------- | -------------------------- | ------------------------- |
| Windows | UAC (User Account Control) | elevated-command + winapi |
| Linux   | pkexec (polkit)            | elevated-command + polkit |

## 依赖配置

### Cargo.toml

```toml
[dependencies]
elevated-command = "1.1.2"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winuser"] }
```

### Linux 系统要求

```bash
# Debian/Ubuntu
sudo apt install policykit-1

# Fedora
sudo dnf install polkit
```

## 核心实现

### 1. 权限检查

```rust
fn is_elevated() -> bool {
    ElevatedCommand::is_elevated()
}
```

- **Windows**: 检查进程是否有管理员令牌
- **Linux**: 检查 uid 是否为 0 (root)

### 2. 提权重启

```rust
fn restart_with_elevation() -> Result<(), String> {
    // For AppImage, use APPIMAGE env var to get the real path
    let exe_path = std::env::var("APPIMAGE")
        .ok()
        .or_else(|| std::env::args().next())
        .ok_or_else(|| "Failed to get executable path".to_string())?;

    ElevatedCommand::new(StdCommand::new(&exe_path))
        .output()
        .map_err(|e| format!("Failed to restart with elevated privileges: {}", e))?;

    Ok(())
}
```

**关键点：**

- **AppImage 支持**: 优先使用 `APPIMAGE` 环境变量获取真实路径
- **回退机制**: 若无 `APPIMAGE`，则使用 `std::env::args().next()`
- **Windows**: 弹出 UAC 对话框，用户确认后以管理员身份重启
- **Linux**: 弹出 pkexec 密码对话框，认证后以 root 身份重启

### 3. 错误提示

#### Linux

```rust
fn show_error_dialog(title: &str, message: &str) {
    let msg = format!("{}: {}", title, message);
    for (cmd, args) in [
        ("zenity", vec!["--error", "--title", title, "--text", message]),
        ("kdialog", vec!["--error", message, "--title", title]),
    ] {
        if StdCommand::new(cmd).args(&args).status().is_ok() {
            return;
        }
    }
    let _ = StdCommand::new("xmessage").arg("-center").arg(&msg).status();
    eprintln!("\n[ERROR] {}\n", msg);
}
```

**对话框工具优先级：**

1. `zenity` - GNOME 桌面环境
2. `kdialog` - KDE 桌面环境
3. `xmessage` - X11 通用回退
4. `stderr` - 终端输出兜底

#### Windows

```rust
fn show_error_dialog(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK};

    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    let msg_wide: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(std::ptr::null_mut(), msg_wide.as_ptr(), title_wide.as_ptr(), MB_OK | MB_ICONERROR);
    }
}
```

### 4. 主入口函数

```rust
pub fn ensure_elevated() {
    #[cfg(debug_assertions)]
    {
        return;
    }

    #[cfg(not(debug_assertions))]
    {
        if is_elevated() {
            return;
        }

        if let Err(e) = restart_with_elevation() {
            let hint = if cfg!(target_os = "linux") {
                "\nHint: Install polkit (pkexec) for privilege elevation."
            } else {
                "\nHint: Run as Administrator."
            };
            show_error_dialog("Privilege Elevation Failed", &format!("{}{}", e, hint));
            exit(1);
        }
        exit(0);
    }
}
```

## 工作流程

```
应用启动
    │
    ▼
┌─────────────────┐
│ ensure_elevated │
└────────┬────────┘
         │
         ▼
    ┌──────────┐    否    ┌─────────────────────┐
    │ 已提权?  │─────────▶│ restart_with_elevation │
    └────┬─────┘          └──────────┬──────────┘
         │ 是                        │
         ▼                           ▼
    ┌──────────┐              ┌──────────────┐
    │ 继续运行  │              │ 用户确认/输入密码 │
    └──────────┘              └───────┬──────┘
                                      │
                       ┌──────────────┼──────────────┐
                       ▼              ▼              ▼
                   ┌───────┐    ┌─────────┐    ┌─────────┐
                   │ 成功  │    │ 用户取消 │    │  失败   │
                   └───┬───┘    └────┬────┘    └────┬────┘
                       │              │              │
                       ▼              ▼              ▼
                  新进程启动      exit(0)      show_error
                                               exit(1)
```

## 常见错误处理

| 错误                                         | 原因                      | 解决方案                   |
| -------------------------------------------- | ------------------------- | -------------------------- |
| `pkexec not found`                           | Linux 未安装 polkit       | `apt install policykit-1`  |
| `Failed to restart with elevated privileges` | 用户取消或认证失败        | 提示用户重试               |
| `Failed to get executable path`              | 程序路径获取失败          | 检查程序完整性             |
| AppImage 无响应                              | `APPIMAGE` 环境变量未设置 | 确保通过 AppImage 正常启动 |
| **"Request dismissed" exit code 126**        | polkit 认证代理未运行/用户取消 | 详见问题排查章节       |
| **exit code 127**                            | 未授权或认证错误          | 可重试                     |

## 问题排查与优化

### 问题：双击文件启动时 "Request dismissed" (exit code 126)

**现象**：双击文件时大多数时候报错 "Request dismissed"，少数情况能弹出授权窗口。

**根因分析**：
1. polkit 认证代理（polkit-gnome/polkit-kde）可能未完全启动
2. 双击启动时环境变量（DISPLAY/XAUTHORITY）可能丢失
3. 无重试机制，一次失败直接报错

**解决方案**（已实现）：

#### 1. polkit 认证代理检测与等待

```rust
#[cfg(target_os = "linux")]
fn is_polkit_agent_running() -> bool {
    // 检测 polkit 认证代理进程
    let agents = ["polkit-gnome-authentication-agent", "polkit-kde-authentication-agent", "polkitd"];
    for agent in agents {
        if StdCommand::new("pgrep").arg("-x").arg(agent).output()?.status.success() {
            return true;
        }
    }
    // 检查 dbus 注册
    StdCommand::new("dbus-send")...
        .arg("org.freedesktop.PolicyKit1.AuthenticationAgent.GetDefaultAgent")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn wait_for_polkit_agent(max_wait_secs: u64) -> bool {
    // 等待代理启动（轮询检测）
    std::thread::sleep(Duration::from_millis(500))
}
```

#### 2. 重试机制

- 最多重试 3 次，每次间隔 1.5 秒
- 给用户时间在 polkit 弹窗中输入密码

#### 3. 区分用户取消与代理未运行

| 条件 | 判断 | 处理 |
|------|------|------|
| exit 126 + 代理运行中 + "Request dismissed" | 用户点击取消 | 报错退出，不重试 |
| exit 126 + 代理未运行 | 代理崩溃/未启动 | 等待后重试 |
| exit 127 或 "Not authorized" | 可恢复错误 | 等待后重试 |

#### 4. 环境变量传递

```rust
cmd.env("RABBIT_ELEVATED", "1");
if let Ok(d) = std::env::var("DISPLAY") {
    cmd.env("DISPLAY", d);
}
if let Ok(x) = std::env::var("XAUTHORITY") {
    cmd.env("XAUTHORITY", x);
}
```

### 跨平台兼容性

| 平台 | 认证代理检测 | 取消/拒绝识别 | 可重试错误 |
|------|-------------|--------------|-----------|
| Linux | polkit 代理进程检测 | exit 126 + 代理运行 + "Request dismissed" | exit 127, "Not authorized" |
| Windows | UAC 始终可用 | - | exit code 5 (拒绝) |
| Other | false | - | - |

### Windows UAC 退出码说明

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 5 | 拒绝/访问被阻止 |
| 1223 | 用户取消 |

如果遇到 Windows 实际退出码与预期不符，请调整代码中的 `code == 5` 条件。

## 相关文件

- `crates/rabbit-platform/src/elevation.rs` - 提权模块实现
- `crates/rabbit-app/src/main.rs` - 主程序入口，调用 `ensure_elevated()`
- `crates/rabbit-platform/Cargo.toml` - 依赖配置
