use elevated_command::Command as ElevatedCommand;
use std::path::absolute;
use std::process::exit;
use std::process::Command as StdCommand;
use std::thread;
use std::time::Duration;

#[cfg(all(target_os = "linux"))]
fn show_error_dialog(title: &str, message: &str) {
    let mut cmd = StdCommand::new("zenity");
    cmd.args(&["--error", "--title", title, "--text", message]);
    if let Ok(d) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", d);
    }
    if let Ok(x) = std::env::var("XAUTHORITY") {
        cmd.env("XAUTHORITY", x);
    }
    let _ = cmd.spawn();
}

/// 检查 polkit 认证代理是否正在运行（Linux）
#[cfg(all(target_os = "linux"))]
fn is_polkit_agent_running() -> bool {
    let agents = ["polkit-gnome-authentication-agent", "polkit-kde-authentication-agent", "polkitd"];

    for agent in agents {
        let output = StdCommand::new("pgrep").arg("-x").arg(agent).output();

        if let Ok(output) = output {
            if output.status.success() {
                return true;
            }
        }
    }

    let output = StdCommand::new("dbus-send")
        .arg("--print-reply")
        .arg("--dest=org.freedesktop.PolicyKit1")
        .arg("/org/freedesktop/PolicyKit1/AuthenticationAgent")
        .arg("org.freedesktop.PolicyKit1.AuthenticationAgent.GetDefaultAgent")
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

/// 检查 UAC 提示是否可用（Windows）
#[cfg(all(target_os = "windows"))]
fn is_auth_agent_running() -> bool {
    // Windows UAC 总是可用，只要用户已登录
    true
}

/// 通用接口：检查认证代理是否可用
#[cfg(target_os = "linux")]
fn check_auth_agent() -> bool {
    is_polkit_agent_running()
}

#[cfg(target_os = "windows")]
fn check_auth_agent() -> bool {
    is_auth_agent_running()
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn check_auth_agent() -> bool {
    false
}

/// 等待 polkit 认证代理启动（Linux）
#[cfg(all(target_os = "linux"))]
fn wait_for_polkit_agent(max_wait_secs: u64) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(max_wait_secs) {
        if is_polkit_agent_running() {
            return true;
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}

#[cfg(all(target_os = "windows"))]
fn show_error_dialog(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
    let msg_wide: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(std::ptr::null_mut(), msg_wide.as_ptr(), title_wide.as_ptr(), MB_OK | MB_ICONERROR);
    }
}

fn is_elevated() -> bool {
    std::env::var("RABBIT_ELEVATED").is_ok() || ElevatedCommand::is_elevated()
}

fn get_exe_path() -> String {
    let rpath = std::env::var("APPIMAGE").ok().unwrap_or_else(|| std::env::args().next().unwrap_or_else(|| "unknown".to_string()));
    absolute(std::path::Path::new(&rpath))
        .unwrap_or_else(|_| std::path::PathBuf::from(&rpath))
        .to_string_lossy()
        .into_owned()
}

fn restart_with_elevation() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let _ = wait_for_polkit_agent(3);
    }

    let max_retries = 3;
    for attempt in 1..=max_retries {
        let exe_path = get_exe_path();
        let mut cmd = StdCommand::new(&exe_path);
        cmd.env("RABBIT_ELEVATED", "1");
        if let Ok(d) = std::env::var("DISPLAY") {
            cmd.env("DISPLAY", d);
        }
        if let Ok(x) = std::env::var("XAUTHORITY") {
            cmd.env("XAUTHORITY", x);
        }

        let agent_available = check_auth_agent();

        match ElevatedCommand::new(cmd).output() {
            Ok(output) => {
                if output.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                let code = output.status.code().unwrap_or(-1);

                // Linux: 仅在认证代理已运行时才认为 126 是用户取消
                #[cfg(target_os = "linux")]
                {
                    if code == 126 && agent_available && stderr.contains("Request dismissed") {
                        return Err("用户取消授权".to_string());
                    }
                }

                // Windows: 拒绝/错误可重试
                #[cfg(target_os = "windows")]
                {
                    if attempt < max_retries && code == 5 {
                        thread::sleep(Duration::from_millis(1500));
                        continue;
                    }
                }

                // Linux: 127 或未授权可重试
                #[cfg(target_os = "linux")]
                {
                    if attempt < max_retries && (code == 127 || stderr.contains("Not authorized")) {
                        thread::sleep(Duration::from_millis(1500));
                        continue;
                    }
                }

                #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                {
                    let _ = code;
                }

                return Err(format!("ElevatedCommand error: {}, stderr: {}", output.status, stderr));
            }
            Err(e) => {
                if attempt < max_retries {
                    thread::sleep(Duration::from_millis(1500));
                    continue;
                }
                return Err(format!("ElevatedCommand error: {}", e));
            }
        }
    }
    Err("Max retries exceeded".to_string())
}

pub fn ensure_elevated() {
    #[cfg(debug_assertions)]
    {
        return;
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        return;
    }

    if is_elevated() {
        return;
    }

    match restart_with_elevation() {
        Ok(()) => exit(0),
        Err(e) => {
            show_error_dialog("elevation failed", &e);
            exit(1);
        }
    }
}
