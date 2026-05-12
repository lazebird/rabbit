//! Windows Taskbar progress bar support
//!
//! 提供统一的任务栏进度管理接口，内部处理平台差异。
//! 调用方无需关心 Windows API 细节和条件编译。

/// 任务栏进度管理器
pub struct TaskbarProgress;

impl TaskbarProgress {
    /// 清除任务栏进度显示
    pub fn clear() {
        if let Some(hwnd) = get_main_window_hwnd() {
            platform::update_taskbar(hwnd, 0, 0, "");
        }
    }

    /// 更新任务栏进度
    /// - progress: 当前进度值
    /// - total: 总值
    /// - color: "green" | "red"
    pub fn update(progress: u32, total: u32, color: &str) {
        if let Some(hwnd) = get_main_window_hwnd() {
            platform::update_taskbar(hwnd, progress, total, color);
        }
    }
}

/// 获取主窗口句柄
fn get_main_window_hwnd() -> Option<usize> {
    get_stored_hwnd()
}

/// 获取已存储的主窗口句柄（通过 window.rs 的统一存储）
fn get_stored_hwnd() -> Option<usize> {
    #[cfg(target_os = "windows")]
    {
        crate::window::get_hwnd()
    }
    #[cfg(not(target_os = "windows"))]
    None
}

/// 平台相关的任务栏操作
mod platform {
    /// 更新任务栏进度（平台特定实现）
    pub fn update_taskbar(hwnd: usize, progress: u32, total: u32, color: &str) {
        #[cfg(target_os = "windows")]
        {
            super::windows::update_taskbar_from_state(hwnd, progress, total, color);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (hwnd, progress, total, color);
        }
    }
}

#[cfg(target_os = "windows")]
pub mod windows {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
    use windows_sys::Win32::UI::Shell::{TBPF_ERROR, TBPF_INDETERMINATE, TBPF_NOPROGRESS, TBPF_NORMAL, TBPF_PAUSED};

    const CLSID_TASKBARLIST: windows_sys::core::GUID = windows_sys::core::GUID {
        data1: 0x56FDF344,
        data2: 0xFD6D,
        data3: 0x11D0,
        data4: [0x95, 0x8A, 0x00, 0x60, 0x97, 0xC9, 0xA0, 0x90],
    };

    const IID_ITASKBARLIST3: windows_sys::core::GUID = windows_sys::core::GUID {
        data1: 0xEA1AFB91,
        data2: 0x9E28,
        data3: 0x4B86,
        data4: [0x90, 0xE9, 0x9E, 0x9F, 0x8A, 0x5E, 0xEF, 0xAF],
    };

    #[repr(C)]
    struct ITaskbarList3 {
        lp_vtbl: *const ITaskbarList3Vtbl,
    }

    #[repr(C)]
    struct ITaskbarList3Vtbl {
        query_interface: Option<unsafe extern "system" fn(*mut ITaskbarList3, *const windows_sys::core::GUID, *mut *mut std::ffi::c_void) -> i32>,
        add_ref: Option<unsafe extern "system" fn(*mut ITaskbarList3) -> u32>,
        release: Option<unsafe extern "system" fn(*mut ITaskbarList3) -> u32>,
        hr_init: Option<unsafe extern "system" fn(*mut ITaskbarList3) -> i32>,
        add_tab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        delete_tab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        activate_tab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        set_active_tab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        mark_fullscreen_window: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, i32) -> i32>,
        set_progress_value: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, u64, u64) -> i32>,
        set_progress_state: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, u32) -> i32>,
        register_tab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, HWND) -> i32>,
        unregister_tab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        set_tab_order: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, u32) -> i32>,
        set_tab_active: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, HWND, u32) -> i32>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TaskbarState {
        None,
        Normal,
        Error,
        Paused,
        Indeterminate,
    }

    fn set_taskbar_state(hwnd: usize, state: TaskbarState, progress: u32) -> Result<(), String> {
        unsafe {
            let _ = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);

            let mut taskbar: *mut ITaskbarList3 = std::ptr::null_mut();
            let result = CoCreateInstance(
                &CLSID_TASKBARLIST,
                std::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &IID_ITASKBARLIST3,
                &mut taskbar as *mut *mut ITaskbarList3 as *mut *mut std::ffi::c_void,
            );

            if result != 0 || taskbar.is_null() {
                return Err(format!("Failed to create TaskbarList: HRESULT={}", result));
            }

            let taskbar = &mut *taskbar;

            if let Some(init) = taskbar.lp_vtbl.as_ref().and_then(|v| v.hr_init) {
                let _ = init(taskbar);
            }

            let tbpflag: u32 = match state {
                TaskbarState::None => TBPF_NOPROGRESS as u32,
                TaskbarState::Normal => TBPF_NORMAL as u32,
                TaskbarState::Error => TBPF_ERROR as u32,
                TaskbarState::Paused => TBPF_PAUSED as u32,
                TaskbarState::Indeterminate => TBPF_INDETERMINATE as u32,
            };

            if let Some(set_state) = taskbar.lp_vtbl.as_ref().and_then(|v| v.set_progress_state) {
                let hr = set_state(taskbar, hwnd as isize as HWND, tbpflag);
                if hr != 0 {
                    if let Some(release) = taskbar.lp_vtbl.as_ref().and_then(|v| v.release) {
                        let _ = release(taskbar);
                    }
                    return Err(format!("SetProgressState failed: HRESULT={}", hr));
                }
            }

            if state != TaskbarState::None {
                if let Some(set_value) = taskbar.lp_vtbl.as_ref().and_then(|v| v.set_progress_value) {
                    let hr = set_value(taskbar, hwnd as isize as HWND, progress as u64, 100);
                    if hr != 0 {
                        if let Some(release) = taskbar.lp_vtbl.as_ref().and_then(|v| v.release) {
                            let _ = release(taskbar);
                        }
                        return Err(format!("SetProgressValue failed: HRESULT={}", hr));
                    }
                }
            }

            if let Some(release) = taskbar.lp_vtbl.as_ref().and_then(|v| v.release) {
                let _ = release(taskbar);
            }

            Ok(())
        }
    }

    /// 直接使用 PingState 预计算的数据更新任务栏
    /// progress: 当前进度值 (相对于 total)
    /// total: 总值
    /// color: "green" | "red"
    pub fn update_taskbar_from_state(hwnd: usize, progress: u32, total: u32, color: &str) {
        if total == 0 {
            let _ = set_taskbar_state(hwnd, TaskbarState::None, 0);
            return;
        }

        let state = match color {
            "green" => TaskbarState::Normal,
            "red" => TaskbarState::Error,
            _ => TaskbarState::Normal,
        };

        // 计算百分比 (0-100)
        let percentage = (progress as f64 / total as f64 * 100.0) as u32;
        let _ = set_taskbar_state(hwnd, state, percentage);
    }
}

#[cfg(not(target_os = "windows"))]
pub mod non_windows {
    pub fn update_taskbar_from_state(_hwnd: usize, _progress: u32, _total: u32, _color: &str) {}
}
