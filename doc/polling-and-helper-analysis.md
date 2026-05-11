# Helper 进程与轮询代码分析报告

生成时间：2026-05-09
分析范围：rabbit 项目全部模块

## 一、Helper 进程分析

### 1.1 概述

**文件**: `crates/app/src/tray_helper.rs`

Helper 进程是 **Linux 特有的** 权限提升 workaround。

### 1.2 存在的必要性

**根本原因**：Linux D-Bus 安全限制

- 当程序使用 `sudo` 提升到 root 权限后，root 用户无法访问普通用户的 D-Bus session bus
- D-Bus daemon 会拒绝来自不同 UID 的连接
- 系统托盘（StatusNotifierItem）需要通过 D-Bus 注册

**解决方案**：
1. 在 `sudo` 提升权限 **之前**，以原用户身份 spawn helper 进程
2. Helper 进程拥有用户的 D-Bus 访问权限，可以创建系统托盘
3. 主进程（root）通过 Unix domain socket 与 helper 通信

**结论**：Helper 进程是**必需的**，但存在替代方案需要评估。

### 1.2.1 深入理解 D-Bus Session 隔离

**问题本质**：

```
用户登录会话 (UID=1000):
┌──────────────────────────────────────────────────────────────────────┐
│  D-Bus Session Bus: unix:path=/run/user/1000/bus                     │
│                                                                       │
│  ┌──────────────┐         ┌──────────────────┐         ┌──────────┐ │
│  │  应用程序     │◀──────▶│ StatusNotifier   │◀────────│  桌面环境  │ │
│  │ (托盘图标)    │         │ Watcher / Host   │         │(GNOME/KDE)│ │
│  └──────────────┘         └──────────────────┘         └──────────┘ │
│                                                                       │
│  关键：所有组件必须连接到同一个 D-Bus Session Bus 才能通信            │
└──────────────────────────────────────────────────────────────────────┘

sudo 提升到 root 后 (UID=0):
┌──────────────────────────────────────────────────────────────────────┐
│  root 进程尝试连接 /run/user/1000/bus                                 │
│                                                                       │
│  D-Bus daemon 检查：                                                   │
│  - 连接者 UID: 0 (root)                                                │
│  - Bus 所有者 UID: 1000 (用户)                                        │
│  - 结果：拒绝连接！(NoReply 或 Connection reset by peer)              │
│                                                                       │
│  原因：dbus-daemon 不认为 root 拥有特殊权限                           │
│        "dbus-daemon is the component enforcing access there,          │
│         and it does not consider 'root' as a user that has           │
│         god-like privs by default"                                    │
└──────────────────────────────────────────────────────────────────────┘
```

### 1.2.2 替代方案分析

| 方案 | 描述 | 优点 | 缺点 | 推荐度 |
|------|------|------|------|--------|
| **A. Helper 进程 (当前实现)** | sudo 前以用户身份 spawn helper，通过 Unix socket 通信 | 清晰分离、不需要 hack、已验证工作 | 额外进程、IPC 开销 | ⭐⭐⭐⭐⭐ |
| **B. seteuid 方案** | 连接 D-Bus 前临时 seteuid(用户 UID) | 不需要额外进程 | 需要修改 ksni 库、复杂、D-Bus 回调时也需要正确 UID | ⭐⭐⭐ |
| **C. 修改系统配置** | `/etc/dbus-1/session-local.conf` 添加 allow root | 简单 | 需要 root 权限修改系统配置、安全风险、用户体验差 | ⭐⭐ |
| **D. 无托盘** | 放弃系统托盘功能 | 简单 | 功能缺失 | ⭐ |

### 1.2.3 方案 B (seteuid) 详细评估

**技术可行性**：
```rust
// Stack Overflow 推荐方案
let user_uid = std::env::var("SUDO_UID")?.parse()?;

// 1. 临时切换有效用户
unsafe { libc::seteuid(user_uid) };

// 2. 连接用户的 D-Bus
let dbus_addr = format!("unix:path=/run/user/{}/bus", user_uid);
std::env::set_var("DBUS_SESSION_BUS_ADDRESS", dbus_addr);

// 3. 注册托盘图标 (ksni 内部操作)
// ...

// 4. 恢复 root 权限
unsafe { libc::seteuid(0) };
```

**问题**：
1. **ksni 库的异步回调**：D-Bus 消息在异步运行时中到来，此时 euid 可能已经恢复为 0
2. **需要深度修改**：需要 fork 或 patch ksni 库
3. **维护成本高**：外部库修改难以维护

### 1.2.4 架构决策：选择 Helper 进程方案

**决策日期**：2026-05-09

**选择理由**：

| 维度 | Helper 进程方案 |
|------|----------------|
| **可靠性** | ✅ 已验证、稳定 |
| **可维护性** | ✅ 清晰分离、不依赖外部库修改 |
| **安全性** | ✅ 不修改系统配置、最小权限原则 |
| **调试难度** | ✅ 独立进程、独立日志 |

**架构优势**：
```
┌─────────────────────────────────────────────────────────────────────┐
│                        进程权限分离架构                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────────────────┐          ┌───────────────────────┐      │
│  │   Helper 进程         │          │   主进程 (root)       │      │
│  │   (UID = 用户)        │          │   (UID = 0)           │      │
│  │                       │          │                       │      │
│  │  ┌─────────────────┐  │          │  ┌─────────────────┐  │      │
│  │  │ D-Bus / ksni    │  │◀───────▶│  │  业务逻辑        │  │      │
│  │  │ 系统托盘        │  │  Unix    │  │  (需要权限操作)  │  │      │
│  │  └─────────────────┘  │  Socket  │  └─────────────────┘  │      │
│  │                       │          │                       │      │
│  └───────────────────────┘          └───────────────────────┘      │
│                                                                     │
│  职责分离：                                                          │
│  - Helper: 用户界面相关 (D-Bus、托盘、显示)                         │
│  - 主进程: 需要权限的操作 (ping、网络扫描等)                         │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.3 Helper 进程职责划分

| 职责 | 所属进程 | 实现方式 |
|------|----------|----------|
| 创建/销毁系统托盘图标 | Helper 进程 | ksni crate |
| 处理托盘点击事件 | Helper 进程 | ksni callback |
| 托盘隐藏/显示 | Helper 进程 | 响应 socket 命令 |
| 主窗口显示/隐藏 | 主进程 | 响应 helper 事件 |
| 程序退出 | 主进程 | 统一管理 |

### 1.4 IPC 通信协议

使用 Unix domain socket: `/tmp/rabbit-tray-{UID}.sock`

**命令格式**：4字节小端整数

| 方向 | 命令 | 值 | 含义 |
|------|------|-----|------|
| 主→Helper | TAG_TRAY_HIDE | 0x10 | 隐藏托盘 |
| 主→Helper | TAG_TRAY_SHOW | 0x11 | 显示托盘 |
| Helper→主 | TAG_ACTIVATE | 0x01 | 托盘图标被点击 |
| Helper→主 | TAG_MENU_SHOW | 0x02 | 菜单"Show" |
| Helper→主 | TAG_MENU_HIDE | 0x03 | 菜单"Hide" |
| Helper→主 | TAG_MENU_QUIT | 0x04 | 菜单"Quit" |

---

## 二、轮询代码全面分析

### 2.1 分类体系

```
轮询代码
├── ✅ 可接受的轮询
│   ├── 业务需要的周期性任务
│   ├── GUI 事件循环的阻塞等待
│   └── 阻塞式 I/O 读取
│
└── ❌ 低效的轮询（需要优化）
    ├── 循环 + sleep 检查标志位
    ├── 非阻塞 accept + sleep
    └── try_wait + sleep
```

### 2.2 ✅ 可接受的轮询（无需修改）

#### 2.2.1 Ping 服务的周期性发送

**文件**: `crates/service/src/ping.rs` (第193-217行)

```rust
let mut ping_interval = interval(Duration::from_millis(current_interval_ms));

loop {
    tokio::select! {
        _ = ping_interval.tick() => {
            // 周期性发送 ping 包
            service.ping_all().await;
        }
        Some(cmd) = rx.recv() => {
            // 响应停止命令
        }
    }
}
```

**分析**：
- Ping 本质上是**周期性任务**，需要定期发送 ICMP 包
- 使用 `tokio::time::interval` 是标准做法
- `tokio::select!` 确保不会阻塞其他事件

**结论**：无需修改，这是正确的事件驱动实现。

#### 2.2.2 Plan 服务的定时触发

**文件**: `crates/service/src/plan.rs` (第148-197行)

```rust
loop {
    let (next_instant, due_tasks) = Self::calc_next(&tasks).await;
    
    tokio::select! {
        _ = sleep_until(instant) => {
            // 触发到期任务，重新计算下次时间
        }
        _ = timer_notify.notified() => {
            // 任务列表变化，立即重新计算
        }
    }
}
```

**分析**：
- 使用 `tokio::time::sleep_until` 睡到精确的触发时间
- 使用 `tokio::sync::Notify` 响应任务列表变化
- **完全是事件驱动**，没有任何忙等

**结论**：这是一个优秀的实现模式，无需修改。

#### 2.2.3 HTTP/TFTP/Chat 服务的 select 模式

**文件**: 
- `crates/service/src/http.rs` (第379-389行)
- `crates/service/src/tftpd.rs` (第127-134行)
- `crates/service/src/chat.rs` (第120-169行)

都是类似模式：
```rust
tokio::select! {
    result = server_future => { /* 服务器自然结束 */ }
    _ = shutdown_rx.recv() => { /* 收到关闭信号 */ }
}
```

**分析**：纯事件驱动，没有轮询。

#### 2.2.4 FLTK 主事件循环

**文件**: `crates/app/src/lifecycle.rs` (第104-142行)

```rust
loop {
    let raw: f64 = Fl_wait_for(0.05);  // 最多等 50ms
    
    if self.is_shutdown_requested() {
        break;
    }
}
```

**背景**：
- FLTK 的 `wait()` 在所有窗口隐藏时会立即返回
- 为了支持"最小化到托盘"功能，必须保持事件循环运行
- `wait_for(0.05)` 是官方推荐的 workaround

**分析**：
- 50ms 超时 = 每秒 20 次检查
- 这是 FLTK 框架的限制，不是代码问题
- CPU 开销极小（每次检查只是几个原子操作）

**结论**：可以接受，无需修改（框架限制）。

#### 2.2.5 阻塞式 Socket 读取

**文件**: `crates/app/src/tray_helper.rs`
- 第176-222行: Helper 命令读取
- 第360-394行: 主进程事件监听

```rust
loop {
    match cmd_reader.read_exact(&mut cmd_buf) {
        Ok(()) => { /* 处理命令 */ }
        Err(e) => { /* 连接关闭，退出 */ }
    }
}
```

**分析**：
- `read_exact()` 是**阻塞调用**，不是轮询
- 没有数据时线程会被操作系统挂起
- 这是正确的 socket 处理方式

**结论**：无需修改。

#### 2.2.6 Downloader 的读取循环

**文件**: `crates/app/src/upgrade/downloader.rs` (第36-53行)

```rust
loop {
    let bytes_read = reader.read(&mut buffer)?;
    if bytes_read == 0 { break; }
    // 处理数据...
}
```

**分析**：标准的阻塞 I/O 读取模式，不是轮询。

---

### 2.3 ❌ 低效的轮询（需要优化）

#### 2.3.1 窗口大小保存的专用轮询线程

**文件**: `crates/app/src/app.rs` (第512-544行)

**当前实现**：
```rust
std::thread::spawn(move || {
    loop {
        std::thread::sleep(Duration::from_millis(100));  // 每次睡 100ms
        
        if pending && now.saturating_sub(last) >= 500 {
            // 检查标志位，保存窗口位置
        }
    }
});
```

**问题分析**：
1. 永久占用一个系统线程
2. 即使没有 resize 发生，线程也每秒醒来 10 次
3. 这是典型的"标志位轮询"反模式

**优化方案**：使用 FLTK 的 timeout 机制

```rust
// 替代方案：在 resize callback 中调度一次性 timeout
resize_callback(|w, x, y, nw, nh| {
    // 设置 500ms 后执行保存（如果已有则重设）
    fltk::app::repeat_timeout3(Duration::from_millis(500), |_handle| {
        save_window_position();
    });
});
```

**收益**：
- 消除永久轮询线程
- 只在实际 resize 发生时调度保存
- 更符合 FLTK 的事件驱动模型

---

#### 2.3.2 权限提升的 try_wait 轮询

**文件**: `crates/adapter/src/elevation.rs` (第149-164行)

**当前实现**：
```rust
let exit_status = loop {
    match child.try_wait() {
        Ok(Some(status)) => break Some(status),
        Ok(None) => {
            if timeout { break None; }
            std::thread::sleep(Duration::from_millis(200));  // 轮询等待
        }
        Err(e) => break None,
    }
};
```

**问题分析**：
- `try_wait()` + `sleep()` 是低效的轮询模式
- 实际上 `std::process::Child::wait()` 就是阻塞等待的！

**优化方案**：使用带 timeout 的阻塞等待

```rust
// 方案 1: 使用 wait_timeout crate（如果允许添加依赖）
use wait_timeout::ChildExt;

match child.wait_timeout(Duration::from_millis(5000)) {
    Ok(Some(status)) => status,
    Ok(None) => {
        // 超时，但 sudo 可能已经成功提升权限
        info!("sudo still running after 5s, assuming success");
        std::process::exit(0);
    }
    Err(_) => ...
}

// 方案 2: 重构逻辑，直接使用 wait()
// 实际上当前的逻辑是：
// - sudo 成功 → 会一直运行（因为 elevated rabbit 还在跑）
// - sudo 失败 → 立即退出返回错误码
// 
// 所以更好的方式是：
// 1. 等待一小段时间看是否立即失败
// 2. 如果没有失败，认为授权成功，直接 exit(0)
```

**更深入的分析**：

当前逻辑：
```
sudo -S ... ./rabbit
        ↓
┌───────┴───────┐
↓               ↓
成功           失败
(一直在运行)    (立即退出)
```

实际上：
- 如果 sudo 密码错误或权限不足，sudo 会立即失败退出
- 如果 sudo 成功，它会一直运行直到被提升的进程退出

所以**更好的方案**是：
```rust
// 只等待 1 秒，如果这期间没退出，就是成功了
const INITIAL_WAIT_MS: u64 = 1000;

let deadline = Instant::now() + Duration::from_millis(INITIAL_WAIT_MS);

loop {
    match child.try_wait() {
        Ok(Some(status)) => {
            // 这段时间内退出了 = 失败
            return handle_sudo_failure(child, status);
        }
        Ok(None) => {
            if Instant::now() > deadline {
                // 1秒没退出 = 成功提升权限
                info!("sudo running normally, assuming elevation success");
                std::process::exit(0);
            }
            std::thread::sleep(Duration::from_millis(50));  // 缩短轮询间隔
        }
        Err(e) => ...
    }
}
```

但这**仍然是轮询**。

**最优方案**：使用 `wait_timeout` crate，或者使用 nix 的 waitpid 带 WNOHANG + pselect/ppoll。

但考虑到这个代码只在程序启动时执行一次（且执行时间很短），可以做一个更轻量的优化：

**当前实际需求分析**：
1. sudo 失败（密码错、无权限）→ 立即返回非零退出码
2. sudo 成功 → 子进程会一直运行

我们只需要判断：**sudo 是立即失败了，还是成功运行了？**

所以最简单的优化：
```rust
// 不使用轮询，直接使用阻塞 wait 配合 thread timeout

// 方案：spawn 一个线程去 wait，主线程带 timeout join

let handle = std::thread::spawn(move || {
    child.wait()
});

match std::thread::scope(|s| {
    s.spawn(|| handle.join()).join_timeout(Duration::from_millis(5000))
}) {
    Ok(Ok(Ok(status))) => handle_exit_status(status),
    Err(_) => {
        // 超时 = sudo 成功运行
        info!("sudo still running after 5s, assuming success");
        std::process::exit(0);
    }
    _ => ...
}
```

实际上更简单的方式是认识到：**当前的 200ms 轮询只在启动时执行最多 25 次（5秒）**，这对性能影响很小。真正的问题是代码模式不好。

**我的建议**：保持逻辑不变，但改用 `wait-timeout` crate 让代码更清晰。

---

#### 2.3.3 Tray Helper 的 accept 轮询

**文件**: `crates/app/src/tray_helper.rs` (第446-481行)

**当前实现**：
```rust
fn accept_with_timeout(listener: &UnixListener, timeout: Duration) -> io::Result<UnixStream> {
    // 设置非阻塞
    unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    
    loop {
        match listener.accept() {
            Ok((stream, _)) => return Ok(stream),
            Err(e) if e.kind() == WouldBlock => {
                if timeout_expired {
                    return Err(TimedOut);
                }
                std::thread::sleep(Duration::from_millis(200));  // 轮询！
            }
            Err(e) => return Err(e),
        }
    }
}
```

**问题分析**：
- 手动设置非阻塞 + 轮询 accept
- 每 200ms 唤醒一次检查

**优化方案**：

**方案 1**: 使用 mio 或 tokio 的异步 UnixListener（推荐）

```rust
// 使用 tokio::net::UnixListener
let listener = tokio::net::UnixListener::bind(path)?;

tokio::time::timeout(Duration::from_secs(30), listener.accept()).await
```

但这需要把整个 `tray_helper::run()` 改为 async，改动较大。

**方案 2**: 使用 `poll` 或 `select` 系统调用

```rust
fn accept_with_timeout(listener: &UnixListener, timeout: Duration) -> io::Result<UnixStream> {
    use std::os::unix::io::AsRawFd;
    
    let fd = listener.as_raw_fd();
    let mut pollfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    
    let timeout_ms = timeout.as_millis() as libc::c_int;
    
    loop {
        // poll() 会阻塞直到有事件或超时，不是轮询！
        let ret = unsafe { libc::poll(&mut pollfd, 1, timeout_ms) };
        
        match ret {
            0 => return Err(io::Error::new(io::ErrorKind::TimedOut, "accept timeout")),
            n if n > 0 => {
                if (pollfd.revents & libc::POLLIN) != 0 {
                    // 有连接了，accept 一定会立即成功
                    return listener.accept().map(|(s, _)| s);
                }
            }
            _ => return Err(io::Error::last_os_error()),
        }
    }
}
```

**方案 2 的优势**：
- 使用 `poll()` 系统调用，操作系统直接挂起线程直到有事件或超时
- **没有轮询，没有忙等**
- 改动极小，不改变现有代码结构
- 完全兼容当前的同步代码

**收益**：
- 消除了 200ms 间隔的轮询
- 有连接时立即响应，不需要等到下一次 sleep 醒来
- 更高效，更省电

---

## 三、优化实施计划

### 阶段一：高优先级（真正的性能问题）

#### 1. 窗口保存线程 → FLTK timeout

**文件**: `crates/app/src/app.rs`

**改动点**：
1. 删除 `loop { sleep(100ms); check_flag; }` 的线程
2. 使用 `fltk::app::add_timeout3` 或 `repeat_timeout3`
3. 在 resize callback 中设置/重置超时

**预期代码**：
```rust
// 不再使用线程 + 轮询，改用事件驱动的 timeout

static SAVE_TIMEOUT_TOKEN: Mutex<Option<fltk::app::TimeoutHandle>> = Mutex::new(None);

main_win.resize_callback(|w, x, y, nw, nh| {
    tabs.resize(...);
    
    // 取消之前的 timeout
    if let Some(handle) = SAVE_TIMEOUT_TOKEN.lock().take() {
        fltk::app::remove_timeout3(handle);
    }
    
    // 设置新的 500ms timeout
    let handle = fltk::app::add_timeout3(
        Duration::from_millis(500),
        |_handle| {
            save_window_position();
        }
    );
    *SAVE_TIMEOUT_TOKEN.lock() = Some(handle);
});
```

#### 2. tray_helper accept_with_timeout → poll()

**文件**: `crates/app/src/tray_helper.rs`

**改动点**：
- 替换 `loop { try_accept; sleep(200ms); }`
- 使用 `libc::poll()` 阻塞等待

#### 3. elevation try_wait 轮询优化

**文件**: `crates/adapter/src/elevation.rs`

**方案选择**：

由于这是一次性的启动代码（最多轮询 25 次，5 秒），性能影响很小。

**建议**：
1. **短期**：保持现有逻辑，但添加注释说明这是启动时代码，影响很小
2. **长期**：考虑添加 `wait-timeout` 依赖或使用 `poll` 等待子进程

或者使用轻量改进：
```rust
// 缩短轮询间隔但保持总超时时间不变
// 实际上只在启动时执行，不是问题

// 更好的改进：认识到我们只关心"sudo 是否立即失败"
// 所以可以这样：

// 只等 1 秒看是否立即失败
// 如果 1 秒内没失败，就是成功了

// 但这改变了超时语义，需要确认原始意图：
// 原始代码等 5 秒是因为用户输入密码可能需要时间
```

**实际上**，xelevate 的密码对话框是在 `try_wait` **之前**就完成的：
```rust
// 第72-80行：先请求密码
let password = match xelevate::request_password() { ... }

// 第135行：spawn sudo，已经输入完密码了
let mut child = cmd.spawn()...;

// 第149行：try_wait 循环
```

所以 `try_wait` 的 5 秒超时不是给用户输密码的，而是给 sudo 启动的。

实际上 sudo 应该很快：
- 成功 → fork + exec，立即返回（子进程继续运行）
- 失败 → 密码错误等，立即返回非零

**真正的问题**：
`sudo -S -k VAR=value ./rabbit` 执行后：
1. sudo 验证密码（已经通过管道输入了）
2. 如果验证成功，sudo fork/exec 目标程序
3. sudo 本身会**一直等待**目标程序退出！

哦，原来如此！所以如果 sudo 成功了，`child.wait()` 会一直等到 rabbit 退出。这就是为什么要用 `try_wait` + 超时 + sleep 的轮询。

**最佳解决方案**：使用 `nix` crate 的 `waitpid` 配合 `WNOHANG` + `pselect` 或使用 `signal-hook` 捕获 SIGCHLD。

或者更简单：使用 `libc::ppoll` 等待 SIGCHLD。

但这需要处理信号，比较复杂。

**实际建议**：
考虑到这是启动时执行一次的代码，且 5 秒内最多 25 次轮询（每次 200ms），对性能的影响可以忽略。真正的问题是代码模式不够好。

**我们可以做一个最小改动优化**：
```rust
// 使用更短的 sleep 时间，但总超时不变
// 比如 sleep 50ms 而不是 200ms
// 这样响应更快，但轮询次数增加到 100 次（5秒内）

// 或者：既然用户已经输入完密码，
// sudo 应该很快就知道成功还是失败

// 实际逻辑应该是：
// 1. sudo 启动子进程后，sudo 本身会变成等待状态
// 2. 但 wait 无法区分"sudo 正在等子进程"和"sudo 还在启动"

// 所以最简单的优化是：保持代码不变，但添加清晰的注释
// 说明这是一次性启动代码，不影响运行时性能
```

**但是**，让我重新考虑一下。原始代码：

```rust
// 第152-157行
Ok(None) => {
    if std::time::Instant::now() > deadline {
        info!("elevate_with_sudo: sudo still running after 5s, assuming success");
        std::process::exit(0);  // 超时就认为成功
    }
    std::thread::sleep(Duration::from_millis(200));
}
```

**注释说得很清楚**：5秒后还在运行就认为成功。

但这里有一个问题：**为什么需要 5 秒？**

如果 sudo 密码正确，sudo 应该**立即** exec 子进程，然后进入 wait 状态。`try_wait()` 应该立即返回 `Ok(None)`。

实际上代码的逻辑是：
```
if try_wait returns Ok(None) = 子进程还在运行
    - 检查是否超过 5 秒
    - 超过 5 秒 → 认为成功，退出
    - 没超过 → sleep 200ms，再检查
```

这其实是**等待确认 sudo 不会立即失败**。

**问题**：为什么不直接用更短的超时？比如 500ms？

或者更根本的问题：能不能不用轮询？

**答案**：在当前的同步代码架构下，不使用信号的话，无法以事件驱动的方式等待子进程退出。

**我的建议**：
1. 对于 `elevation.rs`，**保持当前实现**，因为：
   - 这是一次性启动代码
   - 不影响运行时性能
   - 改动风险 > 收益
2. 添加清晰的注释说明为什么这样实现

---

### 阶段二：验证

1. `cargo check` 通过
2. `cargo clippy` 无新增警告
3. `cargo build` 成功

---

## 四、优化实施记录

### 4.1 已完成的优化 (2026-05-09)

| 文件 | 问题 | 优化方案 | 状态 |
|------|------|----------|------|
| `app.rs` | 永久轮询线程 + 100ms 循环检查 | `thread_local!` + `add_timeout3` 事件驱动 | ✅ 完成 |
| `tray_helper.rs` | `O_NONBLOCK` + 200ms sleep 轮询 | `libc::poll()` 阻塞等待 | ✅ 完成 |
| `elevation.rs` | `try_wait` + 200ms sleep | 一次性启动代码，保持 | 保持 |

### 4.2 优化详情

#### 4.2.1 app.rs 窗口大小保存防抖

**优化前**：
```rust
// 永久运行的轮询线程
std::thread::spawn(move || {
    loop {
        std::thread::sleep(Duration::from_millis(100));  // 每秒醒10次
        // 检查标志位...
    }
});
```

**问题**：
- 永久占用系统线程
- 即使没有 resize 发生也持续轮询
- 每次 resize 可能添加多个定时器（没有取消机制）

**优化后**：
```rust
// 线程本地存储定时器 handle
thread_local! {
    static RESIZE_TIMEOUT: Cell<Option<TimeoutHandle>> = const { Cell::new(None) };
}

// resize callback 中使用真正的防抖
RESIZE_TIMEOUT.with(|cell| {
    // 取消之前的定时器
    if let Some(handle) = cell.take() {
        app::remove_timeout3(handle);
    }
    // 调度新的保存
    let new_handle = app::add_timeout3(0.5, |_| save_window_position());
    cell.set(Some(new_handle));
});
```

**优势**：
- 无轮询线程
- 真正的防抖：连续 resize 只保存一次
- 符合 FLTK 事件驱动模型

#### 4.2.2 tray_helper.rs accept 轮询

**优化前**：
```rust
// 设置非阻塞
unsafe { libc::fcntl(fd, F_SETFL, flags | O_NONBLOCK) };

loop {
    match listener.accept() {
        Ok(stream) => return Ok(stream),
        Err(e) if e.kind() == WouldBlock => {
            if timeout_expired { return Err(...) }
            std::thread::sleep(Duration::from_millis(200));  // 轮询！
        }
        Err(e) => return Err(e),
    }
}
// 恢复阻塞标志
unsafe { libc::fcntl(fd, F_SETFL, flags) };
```

**优化后**：
```rust
// 使用 poll() 阻塞等待，无需设置/恢复非阻塞
let mut pollfd = libc::pollfd {
    fd,
    events: libc::POLLIN,
    revents: 0,
};

let ret = unsafe { libc::poll(&mut pollfd, 1, timeout_ms) };

match ret {
    0 => Err(TimedOut),
    n if n > 0 => {
        if (pollfd.revents & POLLIN) != 0 {
            listener.accept()  // 一定会立即成功
        }
    }
    _ => Err(last_os_error()),
}
```

**优势**：
- 代码更简洁（无需 fcntl 操作）
- 操作系统直接挂起线程直到有事件
- 有连接时立即响应（不需要等到 sleep 醒来）
- 更省电、更高效

### 4.3 Helper 进程架构决策

**决策**：保持 Helper 进程方案

**原因**：
1. **替代方案复杂度高**：`seteuid` 方案需要修改 `ksni` 库的 D-Bus 回调机制
2. **职责分离清晰**：
   - Helper 进程：用户身份，负责 D-Bus/系统托盘
   - 主进程：root 身份，负责需要权限的操作
3. **已验证稳定**：当前实现已经可以工作
4. **IPC 开销可接受**：Unix socket 通信开销很低

**替代方案留待未来评估**：
- 可考虑 `seteuid` + 异步运行时的用户身份切换
- 或 fork 一个以用户身份运行的线程来处理 D-Bus

### 4.4 项目事件驱动成熟度评估

**✅ 优秀的事件驱动实现**：
| 模块 | 实现方式 |
|------|----------|
| Plan 服务 | `sleep_until` + `Notify` 完全事件驱动 |
| HTTP/TFTP 服务 | `tokio::select!` 事件驱动 |
| Chat 服务 | UDP socket + `mpsc` 异步 |
| UI 更新 | `awake_callback` 事件回调 |
| Ping 服务 | `interval` 周期性任务（业务需求） |

**⚠️ 已优化的部分**：
- 窗口大小保存：轮询线程 → FLTK timeout 事件
- Tray accept：手动轮询 → `poll()` 系统调用

**整体评价**：
项目架构已经高度事件驱动。优化前发现的 3 处低效轮询中，2 处已经完成优化，1 处（elevation.rs）是一次性启动代码不影响运行时性能。

---

## 五、验证确认 (2026-05-09)

### 5.1 窗口大小保存防抖验证

**问题 1：低效轮询线程**

| 维度 | 优化前 | 优化后 |
|------|--------|--------|
| 实现方式 | `std::thread::spawn` + `loop { sleep(100ms) }` | `thread_local!` + `add_timeout3` 事件驱动 |
| 线程占用 | 永久占用系统线程 | 无额外线程 |
| CPU 唤醒 | 每秒唤醒 10 次检查标志位 | 仅在 resize 时调度，500ms 后执行 |
| 状态 | ❌ 低效轮询 | ✅ 已解决 |

**问题 2：多定时器重复保存**

| 场景 | 优化前行为 | 优化后行为 |
|------|-------------|-------------|
| t=0ms, resize #1 | 添加定时器 A (500ms) | 添加定时器 A (500ms) |
| t=50ms, resize #2 | 添加定时器 B (550ms) | **取消 A**，添加定时器 B (550ms) |
| t=100ms, resize #3 | 添加定时器 C (600ms) | **取消 B**，添加定时器 C (600ms) |
| t=500ms 后 | A、B、C 都触发，保存 3 次 | 只有最后一个定时器触发，保存 1 次 |

**关键代码**：
```rust
RESIZE_TIMEOUT.with(|cell| {
    if let Some(handle) = cell.take() {
        app::remove_timeout3(handle);  // ← 关键：取消前一个定时器
    }
    let new_handle = app::add_timeout3(0.5, |_handle| save_window_position());
    cell.set(Some(new_handle));
});
```

**验证结论**：
- ✅ 无低效轮询（已删除轮询线程）
- ✅ 无多定时器问题（每次 resize 都会取消前一个定时器）
- ✅ 真正的防抖实现（连续 resize 只保存一次）

### 5.2 Tray Helper accept 验证

| 维度 | 优化前 | 优化后 |
|------|--------|--------|
| 实现方式 | `O_NONBLOCK` + `loop { accept(); sleep(200ms) }` | `libc::poll()` 阻塞等待 |
| 响应延迟 | 最多 200ms 延迟（需要等到 sleep 醒来） | 有连接立即响应 |
| 系统调用 | 循环 accept + fcntl 设置/恢复 | 单次 poll + accept |
| 状态 | ❌ 低效轮询 | ✅ 已解决 |

### 5.3 当前状态总结

| 问题类型 | 位置 | 状态 |
|----------|------|------|
| 低效轮询线程 | `app.rs` | ✅ 已优化删除 |
| 多定时器重复保存 | `app.rs` | ✅ 已修复（取消机制） |
| accept 轮询 | `tray_helper.rs` | ✅ 已优化（poll 阻塞） |
| try_wait 轮询 | `elevation.rs` | ⚠️ 一次性启动代码，保持不变 |

**所有运行时性能问题已解决。**
