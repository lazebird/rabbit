//! Windows Taskbar progress bar support
//!
//! 任务栏状态由 PingState 数据驱动：
//! - ping 模块计算好进度和颜色，通过 UiData::PingState 发送
//! - 本模块只负责使用预计算的数据呈现任务栏状态

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
        lpVtbl: *const ITaskbarList3Vtbl,
    }

    #[repr(C)]
    struct ITaskbarList3Vtbl {
        QueryInterface: Option<unsafe extern "system" fn(*mut ITaskbarList3, *const windows_sys::core::GUID, *mut *mut std::ffi::c_void) -> i32>,
        AddRef: Option<unsafe extern "system" fn(*mut ITaskbarList3) -> u32>,
        Release: Option<unsafe extern "system" fn(*mut ITaskbarList3) -> u32>,
        HrInit: Option<unsafe extern "system" fn(*mut ITaskbarList3) -> i32>,
        AddTab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        DeleteTab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        ActivateTab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        SetActiveTab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        MarkFullscreenWindow: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, i32) -> i32>,
        SetProgressValue: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, u64, u64) -> i32>,
        SetProgressState: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, u32) -> i32>,
        RegisterTab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, HWND) -> i32>,
        UnregisterTab: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND) -> i32>,
        SetTabOrder: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, u32) -> i32>,
        SetTabActive: Option<unsafe extern "system" fn(*mut ITaskbarList3, HWND, HWND, u32) -> i32>,
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

            if let Some(init) = (*taskbar).lpVtbl.as_ref().and_then(|v| v.HrInit) {
                let _ = init(taskbar);
            }

            let tbpflag: u32 = match state {
                TaskbarState::None => TBPF_NOPROGRESS as u32,
                TaskbarState::Normal => TBPF_NORMAL as u32,
                TaskbarState::Error => TBPF_ERROR as u32,
                TaskbarState::Paused => TBPF_PAUSED as u32,
                TaskbarState::Indeterminate => TBPF_INDETERMINATE as u32,
            };

            if let Some(set_state) = (*taskbar).lpVtbl.as_ref().and_then(|v| v.SetProgressState) {
                let hr = set_state(taskbar, hwnd as isize as HWND, tbpflag);
                if hr != 0 {
                    if let Some(release) = (*taskbar).lpVtbl.as_ref().and_then(|v| v.Release) {
                        let _ = release(taskbar);
                    }
                    return Err(format!("SetProgressState failed: HRESULT={}", hr));
                }
            }

            if state != TaskbarState::None {
                if let Some(set_value) = (*taskbar).lpVtbl.as_ref().and_then(|v| v.SetProgressValue) {
                    let hr = set_value(taskbar, hwnd as isize as HWND, progress as u64, 100);
                    if hr != 0 {
                        if let Some(release) = (*taskbar).lpVtbl.as_ref().and_then(|v| v.Release) {
                            let _ = release(taskbar);
                        }
                        return Err(format!("SetProgressValue failed: HRESULT={}", hr));
                    }
                }
            }

            if let Some(release) = (*taskbar).lpVtbl.as_ref().and_then(|v| v.Release) {
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
