# xelevate 密码对话框未自动关闭问题

## 问题描述

在 Linux 平台使用 `xelevate` 0.1.0 进行提权时，双击 release 版本后密码输入框弹出，输入密码点击确认后，对话框窗口不会自动关闭，导致提权流程卡住。

## 影响版本

- `xelevate` 0.1.0
- Linux 平台（使用 sudo 提权路径）
- Rabbit release 0.1.0

## 复现步骤

1. 构建 release 版本：`cargo build --release`
2. 在 Linux 上以普通用户双击运行 `target/release/rabbit`
3. 弹出密码输入框
4. 输入 sudo 密码并点击"确认"
5. **预期**：对话框关闭，程序以提权状态重启
6. **实际**：对话框不关闭，程序无响应

## 根本原因分析

查看 `xelevate` 源码 `~/.cargo/registry/src/.../xelevate-0.1.0/src/auth.rs`：

```rust
let confirm_fn = {
    let p_clone = password.clone();
    let input = input.clone();
    move || {
        *p_clone.borrow_mut() = Some(input.value());
        app::quit();  // 问题所在
    }
};
```

**问题点**：
1. `app::quit()` 仅设置 FLTK 内部退出标志，**不会立即销毁窗口**
2. 密码对话框窗口 `wind` 没有显式调用 `hide()` 或 `destroy()`
3. `app.run()` 事件循环虽然收到退出信号，但窗口对象在函数返回前不会被释放
4. 窗口仍然显示在屏幕上，用户感知为"卡住"

## 临时解决方案

### 方案 1：本地 patch xelevate（推荐）

修改 `~/.cargo/registry/src/.../xelevate-0.1.0/src/auth.rs`：

```rust
let confirm_fn = {
    let p_clone = password.clone();
    let input = input.clone();
    let mut wind = wind.clone();  // 增加 wind 的引用
    move || {
        *p_clone.borrow_mut() = Some(input.value());
        wind.hide();  // 先隐藏窗口
        app::quit();
    }
};
```

### 方案 2：在 Rabbit 中自行实现密码输入

参考 `rabbit-platform/src/elevation.rs` 的 `show_elevation_error`，自行实现带 `hide()` 的密码输入对话框。

### 方案 3：使用 pkexec 替代

修改 `rabbit-platform/src/elevation.rs`，Linux 平台直接使用 `pkexec`：

```rust
#[cfg(unix)]
pub fn ensure_elevated() {
    // ... 检查是否已提权 ...

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let exe_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| std::env::args().next().unwrap_or_default());

        Command::new("pkexec")
            .arg(&exe_path)
            .spawn()
            .expect("Failed to spawn pkexec");
        std::process::exit(0);
    }
}
```

## 长期解决方案

1. **向 xelevate 提交 PR**：修复 `auth.rs` 中的窗口生命周期问题
2. **等待 xelevate 新版本**：作者修复后升级依赖
3. **考虑替换方案**：评估 `pkexec`、`bees` 等其他提权库

## 相关文件

- `crates/rabbit-platform/src/elevation.rs` - Rabbit 提权逻辑
- `~/.cargo/registry/src/.../xelevate-0.1.0/src/auth.rs` - 问题源码
- `~/.cargo/registry/src/.../xelevate-0.1.0/src/linux.rs` - Linux 提权实现
- `doc/known-issues.md` - 其他已知问题汇总（待创建）

## 状态

- [ ] 向 xelevate 提交 issue（等待上游修复）
- [ ] 向 xelevate 提交 PR（附带修复）
- [x] 决定临时解决方案并实施方案 → **已选择方案3：保持现状等待上游修复**
- [ ] 定期跟踪 xelevate 新版本发布

## 记录时间

2026-05-01
