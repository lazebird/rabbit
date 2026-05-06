# Windows 平台启动与退出优化技术文档

本文详细记录了 Rabbit 在 Windows 平台上遇到的启动窗口闪现及程序退出失效问题的技术分析与最终解决方案。

## 1. 窗口闪现问题 (Window Flashing)

闪现问题分为三个层次：控制台闪现、子进程控制台闪现、以及 GUI 状态跳变。

### 1.1 控制台窗口闪现
- **现象**：双击运行时，会出现一个黑色的控制台窗口瞬间消失。
- **原因**：Rust 程序默认编译为控制台子系统（Console Subsystem）。即使在代码中调用 `FreeConsole()`，OS 仍会先分配控制台。
- **解决方案**：在 `main.rs` 顶部添加属性：
  ```rust
  #![windows_subsystem = "windows"]
  ```

### 1.2 后台子进程闪现
- **现象**：即便主程序无控制台，启动时仍会出现“宽度大、高度小”的黑色矩形闪现。
- **原因**：程序调用 `reg.exe`（自启动/右键菜单）或 `powershell.exe`（通知）等子进程时，Windows 默认会为这些后台命令行工具分配短暂的控制台窗口。
- **解决方案**：使用 `CREATE_NO_WINDOW` 标志启动子进程。
  ```rust
  use std::os::windows::process::CommandExt;
  const CREATE_NO_WINDOW: u32 = 0x08000000;
  
  Command::new("reg")
      .creation_flags(CREATE_NO_WINDOW)
      .args(...)
      .output();
  ```

### 1.3 GUI 状态初始化跳变
- **现象**：窗口出现时，按钮颜色或文字发生瞬间切换（如从 Start 变 Stop）。
- **原因**：UI 构建时使用了默认值，窗口显示后（`show()`）才通过异步事件同步真实配置状态。
- **解决方案**：
  - **原子化初始化**：重构 `TabComponent` 接口，在 `build` 阶段传入 `AppConfig`。
  - **构建即最终态**：组件在创建时就根据配置直接设置为运行态的颜色和文字。
  - **延迟显示**：将 `main_win.show()` 移至所有初始化（包括系统集成和状态恢复）的最末尾。

---

## 2. 关闭按钮失效/程序无法退出 (Exit Failure)

### 2.1 异步清理挂起
- **现象**：点击关闭按钮后窗口消失，但进程仍留在任务管理器中。
- **原因**：退出时需调用各服务的异步 `cleanup()`（销毁资源、关闭监听）。如果某个服务（如 Ping 扫描、HTTP 阻塞 IO）因持有锁或等待网络响应而挂起，`cleanup().await` 将无限期等待。
- **解决方案**：引入强制清理超时机制。
  ```rust
  let cleanup_future = self.cleanup();
  match tokio::time::timeout(std::time::Duration::from_secs(3), cleanup_future).await {
      Ok(_) => info!("Cleanup done"),
      Err(_) => warn!("Cleanup timed out, forcing exit"),
  }
  std::process::exit(0);
  ```

### 2.2 事件处理冲突
- **现象**：关闭按钮点击无响应。
- **原因**：同时存在全局事件处理器和窗口回调，导致 `Event::Close` 被拦截或重复处理。
- **解决方案**：简化事件流，以主窗口 `set_callback` 作为唯一的退出触发源，并确保调用 `app::quit()` 停止 FLTK 事件循环。

---

## 3. 其它相关优化

### 3.1 管理员权限 (UAC)
- **优化**：通过 `rabbit.manifest` 声明 `requireAdministrator`。
- **效果**：Windows 在程序运行前直接处理提权，避免了程序内部重启进程带来的视觉断层。

### 3.2 高 DPI 支持
- **优化**：在 Manifest 中声明 `PerMonitorV2`。
- **效果**：确保窗口在显示瞬间即拥有正确的缩放比例，防止 OS 二次拉伸导致的模糊或闪烁。
