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
