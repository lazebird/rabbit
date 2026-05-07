//! Window management utilities
//!
//! Provides platform-specific window operations:
//! - Setting window on top (topmost) or normal state
//! - Hiding/showing window for tray (Windows only)

/// Set or unset the window on-top state
pub fn set_window_on_top(hwnd: usize, on_top: bool) {
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

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (hwnd, on_top);
    }
}

/// Hide window (for tray) - Windows only
/// On Linux, FLTK's hide()/show() are used directly in systray.rs
#[cfg(target_os = "windows")]
pub fn hide_window(hwnd: usize) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
    unsafe {
        ShowWindow(hwnd, SW_HIDE);
    }
}

/// Show window (restore from tray) - Windows only
#[cfg(target_os = "windows")]
pub fn show_window(hwnd: usize) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNORMAL};
    let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
    unsafe {
        ShowWindow(hwnd, SW_SHOWNORMAL);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn hide_window(_hwnd: usize) {}

#[cfg(not(target_os = "windows"))]
pub fn show_window(_hwnd: usize) {}
