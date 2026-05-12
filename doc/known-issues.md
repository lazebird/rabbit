# Rabbit 开发问题记录

本文档汇总 Rabbit 项目开发过程中遇到的各种问题、根因分析及解决方案。
涵盖 UI 框架迁移、配置架构、提权、系统托盘、窗口管理、跨平台兼容等各个方面。

> **其他专项问题文档：**
> - [配置优化问题](./config-issues.md) — 配置模型相关的细化问题
> - [xelevate 密码对话框问题](./xelevate-password-dialog-issue.md) — 提权密码弹窗不自动关闭
> - [Windows 优化](./windows-optimization.md) — 控制台闪现、退出挂起
> - [static mut 警告分析](./static-mut-ref-analysis.md) — Rust 2024 edition `static mut` 警告
> - [FLTK wait_for poll bug](./fltk-wait-for-poll-bug.md) — `poll()` 返回 `> 1` 导致 `wait_for` 误报

---

## 目录

1. [UI 与框架](#1-ui-与框架)
2. [系统托盘 (Tray)](#2-系统托盘-tray)
3. [提权 (Elevation)](#3-提权-elevation)
4. [配置架构](#4-配置架构)
5. [窗口管理](#5-窗口管理)
6. [跨平台兼容](#6-跨平台兼容)
7. [构建与部署](#7-构建与部署)
8. [代码质量与优化](#8-代码质量与优化)
9. [Rust 语言层面](#9-rust-语言层面)

---

## 1. UI 与框架

### 1.1 Slint → FLTK 迁移

| 项目 | 内容 |
|------|------|
| **问题** | 初始选用 Slint 作为 UI 框架，但 Slint 的 Rust 回调绑定机制复杂，状态管理不便，组件化能力有限 |
| **根因** | Slint 使用声明式 UI + 信号/槽机制，与 Rust 的所有权模型结合时导致大量 `#[allow(unused)]` 和不自然的代码结构；运行时性能也不理想 |
| **解决方案** | 迁移到 FLTK，利用其 `fltk::app::awake_callback` 实现线程安全的事件驱动 UI 更新；使用 `fltk::app::wait_for(0.05)` 实现非阻塞事件循环 |
| **涉及文件** | 全量重写 UI 层 |
| **状态** | ✅ 已解决 (`21a80fd`) |

### 1.2 Slint UI 回调绑定

| 项目 | 内容 |
|------|------|
| **问题** | Slint 声明式 UI 中的回调（callback）与 Rust 代码绑定复杂，信号传播路径不清晰 |
| **根因** | Slint 使用信号/槽机制，事件从 UI → Rust 的传递需要手动桥接，且 Slint 的 Rust 绑定在传递闭包时受限于生命周期和 `Send` 约束 |
| **解决方案** | （最终方案）迁移到 FLTK，使用 `fltk::app::awake_callback` 实现线程安全的跨线程事件投递 |
| **状态** | ✅ 已修复 — 随 Slint→FLTK 迁移一同解决 |

### 1.3 按钮状态不更新 (HTTP)

| 项目 | 内容 |
|------|------|
| **问题** | HTTP 模块的启动/停止按钮在服务运行后不更新状态（仍显示"Start"） |
| **根因** | Phase 1 重构时遗留了 `http_running` 检查，导致状态更新分支被跳过 |
| **解决方案** | 移除 Phase 1 中的 `http_running` 检查，统一使用配置中的 `running` 字段 |
| **涉及文件** | `app.rs` |
| **状态** | ✅ 已修复 (`bee96be`) |

### 1.4 FLTK 事件循环阻塞

| 项目 | 内容 |
|------|------|
| **问题** | 使用 `fltk::app::wait()` 时，窗口隐藏后事件循环立即返回，导致 tray "Hide" 模式下程序退出 |
| **根因** | `wait()` 在没有可见窗口时立即返回，与 tray 隐藏模式的"后台常驻"需求冲突 |
| **解决方案** | 改用 `fltk::app::wait_for(0.05)` 配合 `Lifecycle` 标志位，窗口隐藏时继续等待事件 |
| **涉及文件** | `lifecycle.rs` |
| **状态** | ✅ 已修复 (`14f15c2`) |

### 1.5 FLTK `wait_for` 多 fd 就绪误判为信号中断

| 项目 | 内容 |
|------|------|
| **问题** | Release 构建下以 root 运行，启动后立即崩溃，误报 "interrupted by OS signal"，但实际无信号触发 |
| **根因** | fltk-rs 1.5.10 的 `wait_for()` 用 `as i32` 截断返回值后仅匹配 `0` 和 `1`；但 FLTK 的 `Fl::wait(double)` 直接返回 `poll()` 的结果（就绪 fd 数量），当 X11 display fd 和 awake pipe 同时就绪时返回 `2`，fltk-rs 将其误判为错误 |
| **解决方案** | 绕过 `fltk::app::wait_for()`，通过 raw FFI 调用 `Fl_wait_for()`，将 `> 0` 视为成功（而非仅接受 `== 1`） |
| **涉及文件** | `lifecycle.rs` |
| **详细文档** | [fltk-wait-for-poll-bug.md](./fltk-wait-for-poll-bug.md) |
| **状态** | ✅ 已修复 |

### 1.6 UI 初始化状态跳变

| 项目 | 内容 |
|------|------|
| **问题** | 窗口出现时按钮颜色/文字瞬间切换（如 Start → Stop），视觉上闪烁 |
| **根因** | UI 构建时使用默认值，窗口显示后才通过异步事件同步真实配置状态 |
| **解决方案** | 原子化初始化：组件构建时直接传入 `AppConfig`，构建即最终态；将 `main_win.show()` 挪到所有初始化完毕之后 |
| **涉及文件** | `app.rs`, 各模块 UI 组件 |
| **状态** | ✅ 已修复 |

---

## 2. 系统托盘 (Tray)

### 2.1 Linux 系统托盘框架选择 (GTK → ksni)

| 项目 | 内容 |
|------|------|
| **问题** | 初始使用 `gtk-rs`+`libappindicator` 实现 Linux 系统托盘，但 libappindicator 在 Wayland 下不可靠，且 gtk 初始化开销大 |
| **根因** | libappindicator 已停止维护，依赖 dbus-menu 协议，在 GNOME 42+ 上兼容性差；gtk 初始化需要完整的 GTK 环境 |
| **解决方案** | 替换为 `ksni`（纯 Rust D-Bus StatusNotifierItem 实现），无需 GTK 依赖，跨桌面环境兼容 |
| **涉及文件** | `systray.rs`, `systray_linux.rs` |
| **状态** | ✅ 已解决 (`33608b0`) |

### 2.2 Release 版双击不弹 GUI 窗口

| 项目 | 内容 |
|------|------|
| **问题** | release 构建双击运行时窗口不出现，程序似乎正常启动但无界面 |
| **根因** | 提权后 root 进程尝试通过 D-Bus session bus 创建 ksni tray 图标，但 root 用户的 D-Bus 连接被普通用户的 session bus 拒绝（不同 UID），导致 ksni 初始化挂起 |
| **解决方案** | 引入 **Tray Helper IPC 架构**：以一个非提权的 helper 子进程持有 D-Bus tray， elevated parent 通过 Unix domain socket 发送 HIDE/SHOW 命令控制 tray 显示与隐藏 |
| **涉及文件** | `tray_helper.rs` (新), `main.rs`, `systray_linux.rs` |
| **状态** | ✅ 已修复 |

### 2.3 托盘取消后图标依然存在

| 项目 | 内容 |
|------|------|
| **问题** | 在设置中取消"启用系统托盘"后，tray 图标仍然显示在系统托盘中 |
| **根因** | 此前 `SettingsSave` 调用 `shutdown_helper()` 直接杀掉 helper 子进程。但 root 进程被杀后，非 root 用户无法重新创建 D-Bus tray（root 不能连接用户 D-Bus），导致应用重启后 tray 永久丢失 |
| **解决方案** | 改为 helper 常驻 + IPC 协议：helper 进程不退出，收到 `TAG_TRAY_HIDE` 时 shutdown ksni handle 但进程不退出；收到 `TAG_TRAY_SHOW` 时重新创建 ksni |
| **涉及文件** | `tray_helper.rs`, `systray_linux.rs` |
| **状态** | ✅ 已修复 |

### 2.4 重新设置托盘图标不出现

| 项目 | 内容 |
|------|------|
| **问题** | 先取消托盘，再重新启用，图标不再出现 |
| **根因** | 同上，root 进程在被 kill 后无法重新初始化 D-Bus tray |
| **解决方案** | 同上 — helper 常驻 + HIDE/SHOW 命令协议 |
| **涉及文件** | `tray_helper.rs` |
| **状态** | ✅ 已修复 |

### 2.5 启动时未配置托盘但图标存在

| 项目 | 内容 |
|------|------|
| **问题** | 用户未勾选"启用系统托盘"，但启动时 tray 图标仍然出现一瞬间 |
| **根因** | helper 进程启动后默认创建 tray，此时 parent 尚未读取配置判断是否需要 tray |
| **解决方案** | 无论配置如何都连接 helper，连接成功后立即检查配置，如果未启用则立即发送 `TAG_TRAY_HIDE` 隐藏图标 |
| **涉及文件** | `app.rs` |
| **状态** | ✅ 已修复 |

### 2.6 Tray Helper Shutdown 方法冗余

| 项目 | 内容 |
|------|------|
| **问题** | `shutdown_helper()` 有三种退出方式（socket shutdown + child kill + PID file kill），但前两种是冗余的——socket shutdown 已足够触发 helper 正常退出 |
| **根因** | 早期设计中担心 socket 不可达，保留多重 fallback；实际运行中 socket 可用时其他方法永远不会触发 |
| **解决方案** | 移除 `HELPER_CHILD` 静态变量和 PID 文件读写，仅保留 socket shutdown（方法 1） |
| **涉及文件** | `tray_helper.rs` |
| **状态** | ✅ 已修复 |

---

## 3. 提权 (Elevation)

### 3.1 提权代码不稳定 (polkit)

| 项目 | 内容 |
|------|------|
| **问题** | 双击运行时 polkit 提权经常失败，报 "Request dismissed" (exit code 126) |
| **根因** | polkit 认证代理（polkit-gnome/polkit-kde）可能未完全启动；无重试机制，一次失败直接报错 |
| **解决方案** | (1) 检测 polkit 认证代理是否运行；(2) 添加重试机制（最多 3 次，间隔 1.5s）；(3) 区分"用户取消"与"代理未运行"； |
| **涉及文件** | `elevation.rs` |
| **状态** | ✅ 已修复 (`86a6dfc`) |

### 3.2 提权后窗口概率性不出现

| 项目 | 内容 |
|------|------|
| **问题** | 提权后程序以 root 身份重启，但 GUI 窗口有时不出现 |
| **根因** | `sudo` 的默认行为是将 `HOME` 改为 `/root`，同时 `XAUTHORITY` 环境变量丢失，导致 X 认证失败（默认从 `$HOME/.Xauthority` 读取 cookie，但 `/root/.Xauthority` 不存在） |
| **解决方案** | `XAUTHORITY` 已设置时直接转发；未设置时从 `$HOME/.Xauthority` 推导（而非 `/root/.Xauthority`）；使用 `sudo VAR=value` 语法（而非 `-E`）转发 DISPLAY/XAUTHORITY/DBUS_SESSION_BUS_ADDRESS |
| **涉及文件** | `elevation.rs` |
| **状态** | ✅ 已修复 |

### 3.3 xelevate 密码对话框不关闭

| 项目 | 内容 |
|------|------|
| **问题** | 使用 `xelevate` 提权时，输入密码点击确认后对话框不关闭 |
| **根因** | xelevate 的确认回调中调用 `app::quit()` 但不调用 `wind.hide()`，窗口对象在函数返回前不会被释放，用户感知为"卡住" |
| **解决方案** | 在确认回调中先调用 `wind.hide()` 隐藏窗口再 `app::quit()`；最终决定保持现状等待上游修复，同时将提权方案改为直接使用 `sudo` 而非 xelevate |
| **涉及文件** | `elevation.rs` |
| **详细文档** | [xelevate-password-dialog-issue.md](./xelevate-password-dialog-issue.md) |
| **状态** | ✅ 已绕过（改用 sudo + 环境变量转发） |

### 3.4 环境变量丢失 (DISPLAY/XAUTHORITY)

| 项目 | 内容 |
|------|------|
| **问题** | `sudo` 默认重设环境变量，导致图形界面环境变量丢失 |
| **根因** | `sudo -E` 需要 `SETENV` tag 在 sudoers 中启用，很多系统默认不开启 |
| **解决方案** | 使用 `sudo VAR=value command` 语法，单个变量逐个传递，不依赖 sudoers 配置 |
| **涉及文件** | `elevation.rs` |
| **状态** | ✅ 已修复 |

---

## 4. 配置架构

### 4.1 ConfigValue 类型缺失

| 项目 | 内容 |
|------|------|
| **问题** | 迁移到 HashMap 配置模型后，需要统一的配置值枚举 |
| **解决方案** | 添加 `ConfigValue` 枚举：`String` / `Integer` / `Boolean` / `Array`；实现 `serde` untagged 序列化 |
| **涉及文件** | `schema/src/config.rs` |
| **状态** | ✅ 已解决 |

### 4.2 窗口位置不恢复

| 项目 | 内容 |
|------|------|
| **问题** | 程序关闭后重新打开，窗口位置不回到上次关闭时的位置 |
| **根因** | TOML 解析问题，`window_x`/`window_y` 等字段使用手动解析而非自动反序列化 |
| **解决方案** | 在 `save_config` 中直接修改配置文件，确保窗口位置正确写入和读取 |
| **涉及文件** | `app.rs`, `config.rs` |
| **状态** | ✅ 已修复 |

### 4.3 窗口位置被覆盖

| 项目 | 内容 |
|------|------|
| **问题** | 窗口位置在被正确设置后又被覆盖为默认值 |
| **根因** | `save_config` 在保存配置时覆盖了之前设置的窗口位置 |
| **解决方案** | 在保存配置前先读取现有窗口位置，保存后再恢复 |
| **涉及文件** | `app.rs` |
| **状态** | ✅ 已修复 |

### 4.4 运行状态与配置分离

| 项目 | 内容 |
|------|------|
| **问题** | `running` 运行状态既有单独的 ViewModel 字段，又有配置中的标志位，双份维护不一致 |
| **根因** | 早期设计中将运行状态视为 UI 状态而非业务配置 |
| **解决方案** | 删除 ViewModel 中 5 个独立的 `running` 字段和 10 个 getter/setter；统一从配置中读取 `running` 标志位 |
| **涉及文件** | `view_model.rs`, `app.rs` |
| **状态** | ✅ 已修复 |

### 4.5 配置字段映射维护

| 项目 | 内容 |
|------|------|
| **问题** | 使用 HashMap + section 配置后，新增/修改字段需要手动更新映射代码 |
| **根因** | section+map 方式缺乏编译时类型检查 |
| **解决方案** | 文档化字段映射表；添加单元测试验证映射正确性（未采用宏方案） |
| **涉及文件** | `config.rs` |
| **状态** | ✅ 已通过文档+测试覆盖解决 |

---

## 5. 窗口管理

### 5.1 X 按钮无法真正退出程序

| 项目 | 内容 |
|------|------|
| **问题** | 点击窗口关闭按钮后窗口消失，但进程仍驻留在任务管理器/后台 |
| **根因** | FLTK 的默认窗口关闭行为是 hide 而非 quit，事件循环继续运行 |
| **解决方案** | `main_win.set_callback` 触发 `fltk::app::quit()`，配合 `std::process::exit(0)` 确保进程彻底退出 |
| **涉及文件** | `main.rs`, `app.rs` |
| **状态** | ✅ 已修复 |

### 5.2 异步清理挂起

| 项目 | 内容 |
|------|------|
| **问题** | 退出时某些服务的 cleanup 阻塞（如 HTTP 监听器、Ping 扫描线程） |
| **根因** | 异步 `cleanup().await` 可能因持有锁或等待网络响应而无限挂起 |
| **解决方案** | 引入强制清理超时（3 秒），超时后直接跳过 |
| **涉及文件** | `app.rs` |
| **状态** | ✅ 已修复 |

---

## 6. 跨平台兼容

### 6.1 Windows 控制台闪现

| 项目 | 内容 |
|------|------|
| **问题** | 双击 exe 时出现黑色控制台窗口瞬间闪现 |
| **根因** | Rust 默认编译为控制台子系统，OS 会先分配控制台 |
| **解决方案** | 添加 `#![windows_subsystem = "windows"]` 属性 |
| **涉及文件** | `main.rs` |
| **状态** | ✅ 已修复 (`9d61c4e`) |

### 6.2 Windows 子进程控制台闪现

| 项目 | 内容 |
|------|------|
| **问题** | 调用 `reg.exe` / `powershell.exe` 等子进程时闪现黑色矩形 |
| **根因** | 子进程默认分配控制台窗口 |
| **解决方案** | 使用 `CREATE_NO_WINDOW` 标志启动子进程 |
| **涉及文件** | `platform/windows.rs` |
| **状态** | ✅ 已修复 |

### 6.3 Linux MAC 地址解析

| 项目 | 内容 |
|------|------|
| **问题** | 需要跨平台获取 IP 扫描结果的 MAC 地址 |
| **根因** | Linux 下解析 `/proc/net/arp` 格式不稳定；Windows 无标准 ARP 表文件 |
| **解决方案** | 操作系统原生的 ARP 表访问（Linux `/proc/net/arp` + 解析；Windows `GetIpNetTable2`） |
| **涉及文件** | `adapter/src/network.rs` |
| **状态** | ✅ 已解决 (`b562d70`) |

### 6.4 跨平台通知

| 项目 | 内容 |
|------|------|
| **问题** | 计划任务提醒需要在 Windows/Linux 上工作 |
| **根因** | 各平台通知 API 完全不同 |
| **解决方案** | Windows: PowerShell `[System.Windows.Forms.NotifyIcon]`；Linux: `notify-send` (libnotify)；macOS: `osascript` |
| **涉及文件** | `platform/notification.rs` |
| **状态** | ✅ 已解决 |

### 6.5 Windows Release 版因诊断日志路径崩溃导致窗口不出现

| 项目 | 内容 |
|------|------|
| **问题** | release 版本在 Windows 上双击运行时没有任何窗口出现，进程静默退出，无错误提示 |
| **根因** | `rabbit-diag` 使用硬编码 Unix 路径 `/tmp/rabbit-startup-{pid}.log`。Windows 上没有 `/tmp/` 目录（除非安装了 Git Bash/Cygwin），导致 `.expect("cannot open diagnostic log")` 触发 panic。Release 配置了 `panic = "abort"` + `windows_subsystem = "windows"`，所以 panic 直接静默终止进程——无终端输出、无窗口、无错误对话框 |
| **解决方案** | 改用 `std::env::temp_dir()` 获取平台兼容的临时目录：Unix → `/tmp/`，Windows → `%TEMP%`（如 `C:\Users\<user>\AppData\Local\Temp\`）。此 API 是跨平台标准库函数，在所有平台行为正确 |
| **涉及文件** | `rabbit-diag/src/lib.rs` |
| **测试用例** | `test_log_path_uses_temp_dir` — 验证路径在 temp_dir 下；`test_log_does_not_panic_on_current_platform` — 验证日志创建不会 panic；`test_log_content_format` — 验证文件内容格式正确 |
| **状态** | ✅ 已修复 |

---

## 7. 构建与部署

### 7.1 Release 二进制体积

| 项目 | 内容 |
|------|------|
| **问题** | 默认 release 构建体积过大（>30MB） |
| **根因** | 未启用 LTO、未 strip 符号、包含 panic 字符串 |
| **解决方案** | Cargo.toml 中配置 `opt-level = "z"`、`lto = true`、`panic = "abort"`、`strip = true`，最终体积 9.7MB |
| **涉及文件** | `Cargo.toml` |
| **状态** | ✅ 已解决 |

### 7.2 图标嵌入

| 项目 | 内容 |
|------|------|
| **问题** | tray helper 进程运行时需要图标文件，但提权后 cwd 改变导致找不到图标 |
| **根因** | 图标作为外部文件加载，路径依赖 cwd |
| **解决方案** | 使用 `include_bytes!()` 在编译时嵌入 `.ico` 文件，tray helper 中从内存加载 |
| **涉及文件** | `tray_helper.rs`, `icon.rs` |
| **状态** | ✅ 已解决 (`471e887`) |

### 7.3 Windows Release 版因 IcoImage 加载图标崩溃导致窗口不出现

| 项目 | 内容 |
|------|------|
| **问题** | release 版本在 Windows 上双击运行时没有任何窗口出现，进程静默退出，无错误提示 |
| **根因** | `main_win.set_icon(IcoImage::from_data(ico_bytes()))` 在 FLTK 1.5.10 中调用 C 函数 `Fl_ICO_Image_from_data` 时 crash/abort（SIGABRT）。219KB 的 `.ico` 文件（含 10 个分辨率）触发 FLTK 的 ICO 解析 bug，`panic = "abort"` + `windows_subsystem = "windows"` 导致静默终止 |
| **解决方案** | 改用 `adapter::icon::load_app_icon()` 通过 `image` crate 解码 ICO 为原始 RGBA 像素，然后用 `RgbImage::new(rgba, w, h, Rgba8)` 加载。`image` crate 的 ICO 解析器更健壮，`RgbImage` 直接使用像素数组，绕过 FLTK 的 C 层 ICO 解析 |
| **涉及文件** | `adapter/src/icon.rs`（已有 `load_app_icon()`）, `app/src/app.rs`（修改图标加载逻辑） |
| **状态** | ✅ 已修复 |

---

## 8. 代码质量与优化

### 8.1 static mut 警告 (Rust 2024 Edition)

| 项目 | 内容 |
|------|------|
| **问题** | Rust 2024 Edition 认为 `static mut` 是 unsafe，产生 4 个警告 |
| **根因** | 代码中大量使用 `static mut` 存储全局 UI 组件引用（21 处 `ui_refresh.rs`、2 处 `ui_state.rs`、1 处 `ui_events.rs`） |
| **解决方案** | 替换为 `Mutex<Option<T>>` / `OnceLock` 等线程安全包装 |
| **涉及文件** | `ui_refresh.rs`, `ui_state.rs`, `ui_events.rs` |
| **详细文档** | [static-mut-ref-analysis.md](./static-mut-ref-analysis.md) |
| **状态** | ✅ 已解决 (`c53b54d`) |

### 8.2 Dead Code 积累

| 项目 | 内容 |
|------|------|
| **问题** | 多次重构后大量未使用代码未清理 |
| **根因** | 重构时关注业务逻辑变更，忽略了死代码标记 |
| **解决方案** | 删除 crate-level `#![allow(dead_code)]`，逐段清理所有死代码；删除 `PingCommand::AddTarget`、简化的 `TftpOperation`、无用的 `TftpTransferState` 等 |
| **涉及文件** | 多处 |
| **状态** | ✅ 已解决 (`cfd26ab`, `e65164a`, `a2639bc`) |

### 8.3 代码重复（各服务的构造函数）

| 项目 | 内容 |
|------|------|
| **问题** | 各服务（Ping/HTTP/Scan/Chat/Plan）都有高度重复的 `new()` 和 `with_channel()` 构造函数 |
| **根因** | 初始开发时各自独立实现，未抽象公共模式 |
| **解决方案** | 使用 `Default` trait + builder 模式；提取公共 `send_ui` 函数消除重复代码 |
| **涉及文件** | `service/*.rs` |
| **状态** | ✅ 已解决 (`ac25cd6`) |

### 8.4 未处理错误静默忽略

| 项目 | 内容 |
|------|------|
| **问题** | 多处 `Result` 返回值被 `ok()` 静默忽略 |
| **根因** | 开发阶段图省事使用 `.ok()` 忽略错误 |
| **解决方案** | 审查所有 `.ok()` 调用，合理处改为 `?` 传播、`if let Err(e) = ...` 记录日志、或保留但有明确注释 |
| **涉及文件** | `scan.rs`, `chat.rs` 等 |
| **状态** | ✅ 已解决 |

### 8.5 扫描模块 DNS 解析阻塞

| 项目 | 内容 |
|------|------|
| **问题** | `get_mac_from_arp` 中的 DNS 反向查找阻塞 tokio 运行时 |
| **根因** | 同步 DNS 查询在异步运行时中执行，阻塞了整个线程 |
| **解决方案** | 使用 `tokio::task::spawn_blocking` 将阻塞操作移到阻塞线程池 |
| **涉及文件** | `scan.rs` |
| **状态** | ✅ 已解决 |

### 8.6 编译警告过多

| 项目 | 内容 |
|------|------|
| **问题** | 早期开发中产生大量编译器警告（未使用变量、未使用导入、dead code 等） |
| **根因** | 快速原型开发阶段优先关注功能正确性，未及时清理警告 |
| **解决方案** | 使用 `cargo fix` 自动修复大部分警告；随后逐模块手动修复剩余警告；后续将 clippy 纳入日常开发流程 |
| **涉及文件** | 全局 |
| **状态** | ✅ 已解决 |

### 8.7 IP Scanner 输出顺序混乱

| 项目 | 内容 |
|------|------|
| **问题** | Scan 结果日志中"Scan finished"出现在部分"Found online host"之前，日志顺序混乱 |
| **根因** | MAC/DNS 查找使用 `tokio::spawn` 以 fire-and-forget 方式运行，这些任务在 `join_set` 完成后仍可能未完成。`join_set.join_next()` 循环结束后立即发送"Scan finished"，而部分 MAC/DNS 后台任务仍在运行 |
| **解决方案** | 收集所有 `tokio::spawn` 返回的 `JoinHandle`（通过 `Arc<Mutex<Vec<JoinHandle>>>` 跨闭包共享），在向 `join_set` 等待完成后、发送"Scan finished"之前，逐一 await 这些 handle |
| **涉及文件** | `service/src/scan.rs` |
| **状态** | ✅ 已修复 |

### 8.8 DNS 反向查找返回 "bogon" 未过滤

| 项目 | 内容 |
|------|------|
| **问题** | 所有在线主机均显示 "(bogon)" 作为主机名，因为许多消费级路由器 / ISP 的 DNS 服务器对没有 PTR 记录的 IP 返回默认名称 "bogon" |
| **根因** | `resolve_hostname()` 只过滤空字符串，不检查实际返回的 DNS 名称内容 |
| **解决方案** | 添加 `is_bogus_hostname()` 函数，过滤 `bogon`、`*.localdomain`、`localhost` 等已知无意义 PTR 记录 |
| **涉及文件** | `service/src/scan.rs` |
| **状态** | ✅ 已修复 |

### 8.9 Windows MAC 地址全为 "not found"

| 项目 | 内容 |
|------|------|
| **问题** | Windows 平台扫描结果中所有主机的 MAC 地址均显示"MAC not found (no ARP entry)" |
| **根因** | 使用 `SendARP` API 发送原始 ARP 请求，但可能被防火墙、虚拟网卡（VMware/WSL/Hyper-V）拦截，或对跨子网 IP 调用时返回错误。`SendARP` 还要求发送原始网络流量，在某些 Windows 配置下权限不足 |
| **解决方案** | V1: 改为解析 `arp -a` 命令输出读取系统内核 ARP 缓存。V2: 改用 [`GetIpNetTable`] (`iphlpapi.dll`) 内核 API 直接查询系统 ARP 缓存，无需子进程，无控制台窗口闪烁（`arp -a` 在并发扫描时产生 200+ 个控制台窗口）。输出格式为 `xx:xx:xx:xx:xx:xx` |
| **涉及文件** | `adapter/src/network.rs` |
| **状态** | ✅ 已修复 |

---

## 9. Rust 语言层面

### 9.1 模块名与变量名冲突

| 项目 | 内容 |
|------|------|
| **问题** | `ChatConfig` 字段名与模块名冲突，导致编译错误 |
| **根因** | Rust 中 `use chat::ChatConfig` 和变量名 `chat` 在同一作用域冲突 |
| **解决方案** | 重命名冲突变量或使用完整路径限定 |
| **状态** | ✅ 已修复 |

### 9.2 Ping 模块 borrow 冲突

| 项目 | 内容 |
|------|------|
| **问题** | 同时持有 `Arc<RwLock<>>` 的可变引用时出现 borrow 冲突 |
| **根因** | 在持有读锁的情况下尝试获取写锁（死锁），或 `RwLock` 作用域重叠 |
| **解决方案** | 缩小锁作用域，确保读锁在获取写锁前释放；使用独立的 `AtomicBool` 替代部分锁 |
| **状态** | ✅ 已修复 |

### 9.3 计划模块 weekday 方法缺失

| 项目 | 内容 |
|------|------|
| **问题** | `chrono::Datelike` trait 未引入，`weekday()` 方法不可用 |
| **根因** | 缺少 `use chrono::Datelike` |
| **解决方案** | 添加 trait 导入 |
| **状态** | ✅ 已修复 |

---

## 附录：问题分布热力图

| 领域 | 问题数 | 严重程度 |
|------|--------|----------|
| 系统托盘 (Tray) | 6 | 🔴 高（影响核心 UX） |
| 提权 (Elevation) | 4 | 🔴 高（影响程序启动） |
| 配置架构 | 5 | 🟡 中（影响维护性） |
| UI 与框架 | 5 | 🟡 中 |
| 窗口管理 | 2 | 🟡 中 |
| 跨平台兼容 | 5 | 🔴 高（影响程序启动） |
| 构建与部署 | 3 | 🟢 低 |
| 代码质量 | 9 | 🟢 低 |
| Rust 语言层面 | 3 | 🟢 低 |

---

## 附录：相关文档索引

| 文档 | 内容 |
|------|------|
| [progress.md](./progress.md) | 开发进度总览 |
| [config-issues.md](./config-issues.md) | 配置模型相关问题详解 |
| [xelevate-password-dialog-issue.md](./xelevate-password-dialog-issue.md) | xelevate 密码对话框不关闭 |
| [windows-optimization.md](./windows-optimization.md) | Windows 控制台闪现、退出挂起 |
| [elevation.md](./elevation.md) | 跨平台提权方案与问题排查 |
| [static-mut-ref-analysis.md](./static-mut-ref-analysis.md) | `static mut` 警告分析 |
| [code-optimization-analysis.md](./code-optimization-analysis.md) | 代码优化分析 |
| [dead-code.md](./dead-code.md) | 死代码清理记录 |
| [running-state-refactor.md](./running-state-refactor.md) | 运行状态重构 |

---

文档版本：1.3
创建日期：2026-05-12
