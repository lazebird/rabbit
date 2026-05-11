//! Window management utilities
//!
//! Provides platform-specific window operations:
//! - Setting window on top (topmost) or normal state
//! - Hiding/showing window for tray (Windows only)
//!
//! hwnd 统一通过 `set_main_window()` 注册，所有操作从内部静态读取，
//! 不再暴露 hwnd 到公共 API。

use std::sync::OnceLock;

/// 主窗口平台句柄（hwnd / XID），一次注册全局共享
static MAIN_HWND: OnceLock<usize> = OnceLock::new();

/// 注册主窗口句柄（窗口创建后调用一次，跨平台）
pub fn set_main_window(hwnd: usize) {
    let _ = MAIN_HWND.set(hwnd);
}

/// 获取已注册的窗口句柄（内部使用）
fn get_hwnd() -> Option<usize> {
    MAIN_HWND.get().copied()
}

/// Set or unset the window on-top state
///
/// hwnd 通过 `set_main_window()` 预先注册。
pub fn set_window_on_top(on_top: bool) {
    let Some(hwnd) = get_hwnd() else { return };

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, HWND_TOPMOST, HWND_NOTOPMOST, SWP_NOMOVE, SWP_NOSIZE,
        };

        let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
        let pos = if on_top {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        unsafe {
            SetWindowPos(hwnd, pos, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
    }

    #[cfg(target_os = "linux")]
    {
        set_window_on_top_x11(hwnd, on_top);
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = on_top;
    }
}

// ─── Linux: _NET_WM_STATE protocol via FLTK display ─────────────────
//
// Uses FLTK's own X11 display connection (fl_display) obtained via
// fltk::app::display(), so we never open a duplicate connection.
// X11 FFI declarations mirror the approach used in x11_diag.rs.
#[cfg(target_os = "linux")]
mod x11_ffi {
    #![allow(non_camel_case_types, dead_code)]

    use std::ffi::c_void;

    pub type Display = *mut c_void;
    pub type Window = u64;
    pub type Atom = u64;
    pub type Bool = i32;

    extern "C" {
        pub fn XInternAtom(display: Display, name: *const i8, only_if_exists: Bool) -> Atom;
        pub fn XDefaultRootWindow(display: Display) -> Window;
        pub fn XSendEvent(
            display: Display,
            w: Window,
            propagate: Bool,
            event_mask: i64,
            event_send: *mut u8,
        ) -> i32;
        pub fn XFlush(display: Display) -> i32;
    }

    // XClientMessageEvent layout on 64-bit Linux (LP64 data model):
    //
    //   ┌──────────────────────┬────────┬──────┐
    //   │ field                │ offset │ size │
    //   ├──────────────────────┼────────┼──────┤
    //   │ type (i32)           │   0    │   4  │
    //   │ padding              │   4    │   4  │
    //   │ serial (u64)         │   8    │   8  │
    //   │ send_event (i32)     │  16    │   4  │
    //   │ padding              │  20    │   4  │
    //   │ display (*mut u8)    │  24    │   8  │
    //   │ window (u64)         │  32    │   8  │
    //   │ message_type (u64)   │  40    │   8  │
    //   │ format (i32)         │  48    │   4  │
    //   │ padding              │  52    │   4  │
    //   │ data (5 × i64)       │  56    │  40  │
    //   └──────────────────────┴────────┴──────┘
    //   Total: 96 bytes
    #[repr(C)]
    pub struct XClientMessageEvent {
        pub type_: i32,
        _pad0: [u8; 4],
        serial: u64,
        send_event: i32,
        _pad1: [u8; 4],
        display: *mut u8,
        pub window: u64,
        pub message_type: u64,
        pub format: i32,
        _pad2: [u8; 4],
        pub data: [i64; 5],
    }

    pub const CLIENT_MESSAGE: i32 = 33;
    pub const SUBSTRUCTURE_REDIRECT_MASK: i64 = 1 << 22;
    pub const SUBSTRUCTURE_NOTIFY_MASK: i64 = 1 << 19;

    pub const _NET_WM_STATE_REMOVE: i64 = 0;
    pub const _NET_WM_STATE_ADD: i64 = 1;
}

#[cfg(target_os = "linux")]
fn set_window_on_top_x11(x11_window: usize, on_top: bool) {
    use self::x11_ffi::*;

    unsafe {
        let display = fltk::app::display();
        if display.is_null() {
            return;
        }

        let wm_state = XInternAtom(display, "_NET_WM_STATE\0".as_ptr() as *const i8, 0);
        let above = XInternAtom(display, "_NET_WM_STATE_ABOVE\0".as_ptr() as *const i8, 0);

        let root = XDefaultRootWindow(display);
        let window = x11_window as Window;

        let mut event: XClientMessageEvent = std::mem::zeroed();
        event.type_ = CLIENT_MESSAGE;
        event.window = window;
        event.message_type = wm_state;
        event.format = 32;
        event.data[0] = if on_top {
            _NET_WM_STATE_ADD
        } else {
            _NET_WM_STATE_REMOVE
        };
        event.data[1] = above as i64;
        event.data[2] = 0;
        event.data[3] = 1; // source indication: application

        let event_mask = SUBSTRUCTURE_REDIRECT_MASK | SUBSTRUCTURE_NOTIFY_MASK;

        XSendEvent(
            display,
            root,
            0,
            event_mask,
            &mut event as *mut _ as *mut u8,
        );
        XFlush(display);
    }
}

/// Hide window (for tray) - Windows only
/// On Linux, FLTK's hide()/show() are used directly in systray
#[cfg(target_os = "windows")]
pub fn hide_window() {
    let Some(hwnd) = get_hwnd() else { return };
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
    unsafe {
        ShowWindow(hwnd, SW_HIDE);
    }
}

/// Show window (restore from tray) - Windows only
#[cfg(target_os = "windows")]
pub fn show_window() {
    let Some(hwnd) = get_hwnd() else { return };
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNORMAL};
    let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
    unsafe {
        ShowWindow(hwnd, SW_SHOWNORMAL);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn hide_window() {}

#[cfg(not(target_os = "windows"))]
pub fn show_window() {}
