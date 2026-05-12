//! PlatformService — 统一的平台无关 API 入口
//!
//! 将所有平台相关功能集中 re-export，对外提供一致的接口。
//! 调用方可写 `adapter::platform::set_window_on_top(true)`
//! 或 `adapter::set_window_on_top(true)`（通过 lib.rs 的 `pub use platform::*`）。

pub use crate::window::{set_window_on_top, set_main_window, hide_window, show_window};
pub use crate::dialog::*;
pub use crate::autostart::set_autostart;
pub use crate::notification::*;
pub use crate::ping::*;
pub use crate::shell::set_shell_integration;
pub use crate::taskbar::TaskbarProgress;

/// Return the numeric user ID on Unix platforms, `None` on other platforms.
pub fn getuid() -> Option<u32> {
    #[cfg(target_os = "linux")]
    {
        Some(unsafe { libc::getuid() })
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Return current platform identifier string, e.g. "linux-x64", "windows-arm64"
pub fn current_platform() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))] { "windows-x64" }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))] { "windows-arm64" }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))] { "linux-x64" }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))] { "linux-arm64" }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))] { "macos-x64" }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))] { "macos-arm64" }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))] { "unsupported" }
}
