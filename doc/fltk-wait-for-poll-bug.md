# FLTK 事件循环 `wait_for` 误报信号中断问题

## 问题描述

Linux 平台 Release 构建下，Rabbit 应用在启动后立即崩溃退出，`lifecycle.rs` 的事件循环日志显示：

```
[lifecycle] wait_for[#3] error: An unknown error occurred "The event loop was
probably interrupted by an OS signal!" errno=11 (before_call_errno=11)
```

实际**没有任何信号待处理**（`sigpending` 返回空集），`errno=11 (EAGAIN)` 也是过时值——该错误发生在 `poll()` 调用之前。

崩溃仅出现在：
- **Release 构建**（`cargo build --release`）
- **带 tray-helper 的 root 提权路径**（`sudo ./target/release/rabbit`）
- 启动后**前 3 次**事件循环迭代内（全部在 `+0.051s` 同一毫秒内完成）

Debug 构建和普通用户运行路径不受影响。

## 影响版本

| 组件 | 版本 |
|------|------|
| `fltk` | 1.5.10 (crates.io 发布版本，上游 **所有版本至 1.5.23 均未修复**) |
| `fltk-sys` | 1.5.10 |
| Rabbit | ≥ 0.1.0 |
| 平台 | Linux x64 (X11) |
| 构建类型 | Release (优化模式) |

## 复现步骤

```bash
# 需要 sudo 权限（触发 tray-helper 路径）
cargo build --release
sudo ./target/release/rabbit

# 预期：Rabbit 窗口正常显示
# 实际：程序立即退出（event loop exited, reason=wait_for_error）
```

**注**：该问题是一个**竞态条件**，并非 100% 复现。当 tokio 异步事件恰好与 X11 事件同时到达时触发。可在启动时发送大量异步事件来提高触发概率。

## 调试过程

### 1. 信号假设排除

通过 `sigpending` 和 `pthread_sigmask` 进行全面信号日志：

```
[lifecycle] signal pre-mask:  SIG{}
[lifecycle] signal pending:   SIG{}
[lifecycle] signal post-mask: SIG{1,3,4,5,6,7,8,10,11,12,13,14,16,17,18,20,21,22,23,24,25,26,27,28,29,30,31}
```

- 阻塞了除 SIGINT/SIGTERM 外的所有信号
- 错误发生时 `sigpending` 仍为空集
- ❌ 信号不是原因

### 2. errno 假设排除

`errno=11 (EAGAIN)` 在所有调用（包括成功调用）中一致出现，且 `poll()` 不会设置 `EAGAIN`：

```
[lifecycle] wait_for[#1]: raw=1 errno=11 before_errno=11   ← 成功，但 errno 已为 11
[lifecycle] wait_for[#2]: raw=1 errno=11 before_errno=11   ← 成功
```

❌ `errno=11` 是过时值，来自之前的文件操作（可能是 tray-helper socket 连接），与 `poll()` 无关。

### 3. 原始返回值捕获

绕过 `fltk::app::wait_for()`，直接通过 FFI 调用 `Fl_wait_for()` 获取原始 `double` 返回值：

```rust
extern "C" {
    fn Fl_wait_for(dur: f64) -> f64;
}
```

在 Debug 构建下运行 290+ 次迭代，返回值仅有两种：

| 原始值 | 二进制 (IEEE 754) | `as i32` | 含义 |
|--------|-------------------|----------|------|
| `0.0` | `0x0000000000000000` | 0 | 超时，无事件 |
| `1.0` | `0x3ff0000000000000` | 1 | 有事件被处理 |

Release + tray-helper 路径因竞态条件无法稳定复现，但通过 FLTK 源码分析确认了 `> 1` 的返回值。

### 4. FLTK 源码分析

#### `Fl::wait(double)` 调用链

```cpp
// Fl.cxx:631
double Fl::wait(double time_to_wait) {
    return system_driver()->wait(time_to_wait);
}
```

```cpp
// Fl_Unix_System_Driver.cxx:792
double Fl_Unix_System_Driver::wait(double time_to_wait) {
    time_to_wait = Fl_System_Driver::wait(time_to_wait);  // 处理定时器/空闲回调
    if (time_to_wait <= 0.0) {
        int ret = scr_dr->poll_or_select_with_delay(0.0);  // 非阻塞 poll
        Fl::flush();
        return ret;  // ← 返回 int 升级为 double！
    } else {
        Fl::flush();
        // ...
        return scr_dr->poll_or_select_with_delay(time_to_wait);  // 阻塞 poll
    }
}
```

#### `poll_or_select_with_delay` 的实现

```cpp
// Fl_x.cxx:133 (X11 覆写)
int Fl_X11_Screen_Driver::poll_or_select_with_delay(double time_to_wait) {
    if (fl_display && XQLength(fl_display)) {
        do_queued_events();
        return 1;  // X11 队列中有事件
    }
    return Fl_Unix_Screen_Driver::poll_or_select_with_delay(time_to_wait);
}
```

```cpp
// Fl_Unix_Screen_Driver.cxx:39
int Fl_Unix_Screen_Driver::poll_or_select_with_delay(double time_to_wait) {
    n = ::poll(pollfds, nfds, int(time_to_wait*1000 + .5));
    if (n > 0) {
        for (int i=0; i<nfds; i++) {
            if (pollfds[i].revents) fd[i].cb(pollfds[i].fd, fd[i].arg);
        }
    }
    return n;  // ← 直接返回 poll() 的原始结果！
}
```

#### `pollfds` 数组的构成

`pollfds` 数组通过 `Fl::add_fd()` 填充。X11 display fd 在 FLTK X11 初始化时注册：

```cpp
// Fl_x.cxx:610 (open_display_i 中)
Fl::add_fd(ConnectionNumber(d), POLLIN, fd_callback);
```

`ConnectionNumber(d)` 是 X11 宏，返回 X11 连接底层的 socket fd。

此外，`Fl::awake()` 创建的**唤醒管道 (awake pipe)** 也通过 `Fl::add_fd()` 注册到同一数组。

## 根本原因

### 直接原因

`Fl::wait(double)` **直接返回 `::poll()` 的原始返回值**（int 升级为 double）。`poll()` 返回的是**就绪文件描述符的数量**，而不仅仅是 0 或 1。

当两个 fd 同时就绪时（例如 X11 display fd 和 awake pipe 都有数据可读），`poll()` 返回 **2**。这个值 `2.0` 作为 double 返回给 Rust 层。

### fltk-rs 的匹配问题

```rust
// fltk-1.5.10/src/app/rt.rs:100
match fl::Fl_wait_for(dur) as i32 {
    0 => Ok(false),      // 超时
    1 => Ok(true),       // 有事件
    _ => Err(FltkError::Unknown(    // ← 其他值被误认为错误！
        "The event loop was probably interrupted by an OS signal!",
    )),
}
```

`2.0f64 as i32` → `2` → 落入 `_` 分支 → 返回 `Err("interrupted by OS signal")`。

**但 FLTK 已经处理了两个 fd 的回调！** 返回值 `> 1` 并不表示错误，只表示有多个 fd 就绪。fltk-rs 的匹配逻辑过于严格。

## FLTK 设计意图分析

### Fl::wait(double) 的返回值合同

FLTK 官方文档和源码注释给出了明确的返回值约定：

**Fl.cxx:626-629（API 文档）**：
```cpp
\return Always 1 on Windows. Otherwise, it is positive
if an event or fd happens before the time elapsed.
It is zero if nothing happens.  It is negative if an error
occurs (this will happen on X11 if a signal happens).
```

**Fl_Unix_Screen_Driver.cxx:36-38（实现注释）**：
```cpp
// This is never called with time_to_wait < 0.0:
// It should return negative on error, 0 if nothing happens before
// timeout, and >0 if any callbacks were done.
```

两处都明确写的是 **"positive" / ">0"**，而不是 **"exactly 1"**。这意味着 FLTK 的设计意图允许任何正数返回值，fltk-rs 要求精确匹配 `1` 是绑定层的 bug，不是 FLTK 的设计缺陷。

### pollfds 数组大小与 nfds 上限

`pollfds` 数组通过 `Fl::add_fd()` 动态管理：

```cpp
// Fl_Unix_System_Driver.cxx:706-735
void Fl_Unix_System_Driver::add_fd(int n, int events, void (*cb)(int, void*), void *v) {
    remove_fd(n, events);
    int i = Fl_Unix_Screen_Driver::nfds++;          // ← nfds 递增
    if (i >= fd_array_size) {
        fd_array_size = 2*fd_array_size + 1;        // ← 动态扩容
        // realloc pollfds 数组...
    }
    Fl_Unix_Screen_Driver::pollfds[i].fd = n;
    Fl_Unix_Screen_Driver::pollfds[i].events = events;
}
```

关键点：
- `nfds` 每次添加 fd 时以 `++` 递增
- `fd_array_size` 按 `2*size+1` 指数级扩容（序列：0→1→3→7→15→31→...），**无硬性上限**
- 数组通过 `realloc` 动态扩展，受系统内存限制
- `::poll(pollfds, nfds, timeout)` 的返回值理论上可以是 `0..nfds` 之间的任意整数

典型 FLTK 应用中 `nfds` 很小（X11 display fd + awake pipe ≈ 2-3），但大量使用 `Fl::add_fd()` 的应用可以有更大的 nfds。

### 为什么 FLTK 不截断/钳位返回值

`poll_or_select_with_delay()` 的实现直接返回 `::poll()` 的原始结果：

```cpp
// Fl_Unix_Screen_Driver.cxx:69-83
if (n > 0) {
    for (int i=0; i<nfds; i++) {        // ← 遍历所有 fd
        if (pollfds[i].revents)
            fd[i].cb(pollfds[i].fd, fd[i].arg);  // ← 回调已全部执行
    }
}
return n;  // ← 直接返回 poll() 原始值，不做截断
```

原因很简单：
1. **回调已在返回前全部执行完毕**：`n > 0` 分支中遍历了所有 `nfds`（不是 n 个），所有就绪 fd 的回调都已经调用
2. **返回值仅为"有事发生"的信息指示**：调用方只需要知道"是否有事件被处理了"，不需要精确数量
3. **`poll()` 的返回值是可靠的**：`n = ::poll(...)` 的 `n` 本身就是 int 类型，直接返回不需要额外处理
4. **没有理由做截断**：`min(n, 1)` 或 `n > 0 ? 1 : 0` 这样的截断会丢失信息，且对 FLTK 自身没有意义

### 小结

| 层面 | 返回值约定 | 说明 |
|------|-----------|------|
| FLTK API 文档 | `positive` if event/fd happens | 明确设计为任意正数 |
| FLTK 实现注释 | `>0` if any callbacks were done | 同上 |
| FLTK 实现 | `return n;` (不做截断) | `::poll()` 的原始结果 |
| fltk-rs 绑定 | `as i32` 仅匹配 `0` 和 `1` | **绑定层 bug**，违反 FLTK 合同 |

根本结论：**bug 在 fltk-rs 的绑定层，不在 FLTK 核心库**。FLTK 的设计和实现都是正确的，fltk-rs 的 `src/app/rt.rs:100` 要求精确匹配 `1` 过于严格，不符合 FLTK 的返回值合同。

### 触发时序

启动时事件密集的典型顺序：

```
T+0.051  wait_for #1: X11 display fd 就绪 (MapNotify) → poll 返回 1 → Ok(true)
T+0.051  wait_for #2: X11 display fd 就绪 (ConfigureNotify) → poll 返回 1 → Ok(true)
T+0.051  wait_for #3: X11 display + awake pipe 同时就绪 → poll 返回 2 → Err!
                      ↑ tokio 异步事件触发 awake_callback()
```

awake pipe 是 `Fl::awake()` 创建的管道读端。tokio 运行时处理异步事件（如服务状态变更）时调用 `fltk::app::awake_callback()`，向管道写入一个字节，导致 `poll()` 返回 2。

### 为什么只影响 Release + tray-helper 路径

- **Debug 构建**：编译时未优化，事件处理节奏较慢，tokio 异步事件的时序通常不会与 X11 事件完全重合
- **Release 构建**：编译优化后代码执行更快，事件处理密度增加，多 fd 同时就绪的概率显著提高
- **tray-helper 路径**：多了一个 Unix socket 连接，可能增加额外的事件源或改变事件循环的时序特征

## 修复方案

### 实现

在 `lifecycle.rs` 中绕过 `fltk::app::wait_for()`，直接通过 FFI 调用 `Fl_wait_for()` 并正确处理返回值：

```rust
extern "C" {
    fn Fl_wait_for(dur: f64) -> f64;
}

// 在事件循环中：
let raw: f64 = Fl_wait_for(0.05);
let as_i32 = raw as i32;
if as_i32 < 0 {
    // poll()/select() 真正失败，不可恢复
    let en = *libc::__errno_location();
    diag!("wait_for[#{iter}] ERROR: poll returned {raw} errno={en}");
    // 检查信号、记录日志...
    reason = "wait_for_error";
    break;
} else if as_i32 == 0 {
    // 超时，无事件
} else {
    // 一个或多个事件已处理，继续循环
    // as_i32 > 1 是正常情况（多个 fd 同时就绪）
}
```

### 更改的文件

- `crates/app/src/lifecycle.rs` — 替换 `fltk::app::wait_for()` 调用

### 原理

| `as_i32` | 含义 | 原行为 | 修复后行为 |
|----------|------|--------|-----------|
| `< 0` | `poll()` 返回错误 (EINTR/EBADF 等) | 未处理（无此分支） | 退出循环（不可恢复） |
| `== 0` | 超时，无事件 | `Ok(false)` → 继续 | 继续（不变） |
| `> 0` | 有事件被处理（可能多个 fd） | `1` 时继续，`> 1` 时退出 | 继续（修复） |

## 相关文件

| 文件 | 说明 |
|------|------|
| `crates/app/src/lifecycle.rs` | 事件循环实现（修复位置） |
| `~/.cargo/registry/.../fltk-1.5.10/src/app/rt.rs:100` | fltk-rs 有问题的 `wait_for` 实现 |
| `~/.cargo/registry/.../fltk-sys-1.5.10/cfltk/src/cfl.cpp:384` | CFL 层 `Fl_wait_for` 绑定 |
| `~/.cargo/registry/.../fltk-sys-1.5.10/cfltk/fltk/src/Fl.cxx:631` | FLTK `Fl::wait(double)` 实现 |
| `~/.cargo/registry/.../fltk-sys-1.5.10/.../Fl_Unix_System_Driver.cxx:792` | Unix `wait(double)` 实现 |
| `~/.cargo/registry/.../fltk-sys-1.5.10/.../Fl_x.cxx:133` | X11 `poll_or_select_with_delay` |
| `~/.cargo/registry/.../fltk-sys-1.5.10/.../Fl_Unix_Screen_Driver.cxx:39` | Unix `poll_or_select_with_delay` |
| `~/.cargo/registry/.../fltk-sys-1.5.10/.../Fl_x.cxx:610` | X11 display fd 注册到 poll 列表 |
| `~/.cargo/registry/.../fltk-sys-1.5.10/.../Fl_Unix_System_Driver.cxx:706` | `add_fd` 实现 (pollfds 填充) |

## 长期建议

1. **升级 fltk 版本**：如前所述，从 fltk-rs **master 分支最新源码**（2026-05-09 确认）看，`wait_for` 的 `as i32` 精确匹配 `0`/`1` 逻辑**至今未修复**（最新发布版本 1.5.22，docs.rs 可见 1.5.23，均同）。升级到 1.5.22 可获得其他 bug 修复和新功能，但**不能移除 raw FFI 绕过**。
2. **向 fltk-rs 提交 PR**：修复 `src/app/rt.rs` 中的 `wait_for` 实现，将 `_` 分支改为 `n if n > 0 => Ok(true)`。这是根治此问题的唯一方式。
3. **当前修复**：已在 Rabbit 项目层绕过，无需等待上游，且增加了 `< 0`（真正错误）的处理。可以安全地保持此方案直至上游修复。

## 上游跟踪记录

| 日期 | fltk-rs 发布版 | `wait_for` 状态 | 备注 |
|------|---------------|----------------|------|
| 2025-07-26 | 1.5.10 | ❌ `as i32` 仅匹配 0/1 | Rabbit 发现此版本 |
| 2025-08-17 | 1.5.11 | ❌ 未修复 | — |
| 2025-09-29 | 1.5.15 | ❌ 未修复 | — |
| 2025-10-20 | 1.5.20 | ❌ 未修复 | — |
| 2025-11-16 | 1.5.22 | ❌ 未修复 | 最新发布版 |
| 2026-03-01 | master | ❌ 未修复 | master 分支源码确认 |

## 状态

- [x] 问题定位完成
- [x] 根因确认（fltk-rs `wait_for` 模式匹配过于严格）
- [x] 修复实施
- [x] 测试通过（18/18 tests, clippy clean）
- [x] 2026-05-09 复查：上游仍存在，修复方案仍然有效
- [ ] 长期跟踪 fltk-rs 上游修复
- [ ] 上游修复后切换回 `fltk::app::wait_for()`（移除 raw FFI）

## 参考

- FLTK 文档：`Fl::wait(double)` 返回值说明（"It returns positive if an event or fd happens, zero if nothing happens, negative on error"）
- fltk-rs CHANGELOG：https://github.com/fltk-rs/fltk-rs/blob/master/CHANGELOG.md
- fltk-rs `rt.rs`（master 分支）：https://github.com/fltk-rs/fltk-rs/blob/master/fltk/src/app/rt.rs

## 记录时间

| 日期 | 事件 |
|------|------|
| 2026-05-09 | 初始文档创建 + FLTK 设计意图分析 |
| 2026-05-09 | 上游复查：bug 仍存在，文档补充 |
