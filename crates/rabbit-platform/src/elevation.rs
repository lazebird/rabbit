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
    let display = std::env::var("DISPLAY").unwrap_or(":1".into());
    let xauth = std::env::var("XAUTHORITY").unwrap_or("/run/user/1000/gdm/Xauthority".into());

    // 关键：允许 root 访问 X11
    let _ = std::process::Command::new("xhost").arg("+SI:localuser:root").status();

    std::process::Command::new("pkexec")
        .arg("env")
        .arg(format!("DISPLAY={display}"))
        .arg(format!("XAUTHORITY={xauth}"))
        .arg(&exe_path)
        .status()
        .map_err(|e| e.to_string())?;

    Ok(())
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
            show_error_dialog("提权失败", &e);
            exit(1);
        }
    }
}
