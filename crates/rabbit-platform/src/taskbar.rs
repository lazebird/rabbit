//! Windows Taskbar progress bar support
//!
//! Implements the taskbar integration for Ping status:
//! - Green: Last 5 pings all successful
//! - Red: Any failures in last 5 pings

#[cfg(target_os = "windows")]
pub mod windows {
    use windows_sys::Win32::UI::Shell::{
        ITaskbarList3, ITaskbarList, TaskbarList,
        TBPFLAG, TBPF_NOPROGRESS, TBPF_INDETERMINATE, TBPF_NORMAL, TBPF_ERROR, TBPF_PAUSED
    };
    use windows_sys::Win32::Foundation::{HWND, S_OK};
    use windows_sys::core::Interface;

    /// Taskbar progress state
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TaskbarState {
        /// No progress indicator
        None,
        /// Green - success/normal
        Normal,
        /// Red - error
        Error,
        /// Yellow - paused
        Paused,
        /// Marquee - indeterminate
        Indeterminate,
    }

    /// Set taskbar progress state for a window
    pub fn set_taskbar_state(hwnd: usize, state: TaskbarState, progress: u32) -> Result<(), String> {
        unsafe {
            // Create taskbar list COM object
            let taskbar: ITaskbarList3 = match TaskbarList.activate() {
                Ok(tb) => tb.cast().map_err(|e| format!("Failed to cast to ITaskbarList3: {:?}", e))?,
                Err(e) => return Err(format!("Failed to create TaskbarList: {:?}", e)),
            };

            let tbpflag = match state {
                TaskbarState::None => TBPF_NOPROGRESS,
                TaskbarState::Normal => TBPF_NORMAL,
                TaskbarState::Error => TBPF_ERROR,
                TaskbarState::Paused => TBPF_PAUSED,
                TaskbarState::Indeterminate => TBPF_INDETERMINATE,
            };

            // Set progress state
            let result = taskbar.SetProgressState(hwnd as HWND, tbpflag);
            if result != S_OK {
                return Err(format!("SetProgressState failed: {}", result));
            }

            // Set progress value (0-100)
            if state != TaskbarState::None {
                let result = taskbar.SetProgressValue(hwnd as HWND, hwnd as HWND, progress as u64);
                if result != S_OK {
                    return Err(format!("SetProgressValue failed: {}", result));
                }
            }

            Ok(())
        }
    }

    /// Update taskbar based on ping results (last 5 pings)
    pub fn update_taskbar_for_ping(hwnd: usize, success_count: u32, total_count: u32) {
        if total_count == 0 {
            // No pings yet
            let _ = set_taskbar_state(hwnd, TaskbarState::None, 0);
            return;
        }

        let progress = (success_count * 100) / total_count;
        
        if success_count == total_count {
            // All successful - green
            let _ = set_taskbar_state(hwnd, TaskbarState::Normal, progress);
        } else {
            // Some failures - red
            let _ = set_taskbar_state(hwnd, TaskbarState::Error, progress);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod non_windows {
    /// Placeholder for non-Windows platforms
    pub fn update_taskbar_for_ping(_hwnd: usize, _success_count: u32, _total_count: u32) {
        // No-op on non-Windows platforms
    }
}
