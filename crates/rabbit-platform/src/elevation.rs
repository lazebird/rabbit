use elevated_command::Command as ElevatedCommand;
use std::path::absolute;
use std::process::exit;
use std::process::Command as StdCommand;

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

#[cfg(all(target_os = "windows"))]
fn show_error_dialog(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{MessageBoxW, MB_ICONERROR, MB_OK};

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
    let exe_path = get_exe_path();
    let mut cmd = StdCommand::new(&exe_path);
    cmd.env("RABBIT_ELEVATED", "1");
    let output = ElevatedCommand::new(cmd).output().map_err(|e| format!("ElevatedCommand error: {}", e))?;
    if output.status.success() {
        return Ok(());
    }
    return Err(format!("ElevatedCommand output error: {}, stderr: {}", output.status, String::from_utf8_lossy(&output.stderr)));
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
