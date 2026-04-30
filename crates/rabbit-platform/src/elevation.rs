pub fn ensure_elevated() {
    #[cfg(any(debug_assertions, target_os = "android", target_os = "ios"))]
    {}

    #[cfg(not(any(debug_assertions, target_os = "android", target_os = "ios")))]
    {
        #[cfg(target_os = "linux")]
        {
            use elevated_command::Command as ElevatedCommand;
            use std::process::Command as StdCommand;
            use std::thread;
            use std::time::Duration;

            if std::env::var("RABBIT_ELEVATED").is_ok() || ElevatedCommand::is_elevated() {
                return;
            }

            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(3) {
                let mut found = false;
                for agent in ["polkit-gnome-authentication-agent", "polkit-kde-authentication-agent", "polkitd"] {
                    if let Ok(o) = StdCommand::new("pgrep").arg("-x").arg(agent).output() {
                        if o.status.success() {
                            found = true;
                            break;
                        }
                    }
                }
                if found {
                    break;
                }
                thread::sleep(Duration::from_millis(500));
            }
        }

        #[cfg(target_os = "windows")]
        {
            use elevated_command::Command as ElevatedCommand;
            use std::process::Command as StdCommand;
            use std::thread;
            use std::time::Duration;

            if std::env::var("RABBIT_ELEVATED").is_ok() || ElevatedCommand::is_elevated() {
                return;
            }
        }

        let max_retries = 3;
        for attempt in 1..=max_retries {
            #[cfg(target_os = "linux")]
            {
                use elevated_command::Command as ElevatedCommand;
                use std::process::Command as StdCommand;
                use std::thread;
                use std::time::Duration;

                let path = {
                    let r = std::env::var("APPIMAGE").unwrap_or_else(|_| std::env::args().next().unwrap_or_else(|| "unknown".to_string()));
                    std::path::Path::new(&r).canonicalize().unwrap_or_else(|_| std::path::PathBuf::from(&r)).to_string_lossy().into_owned()
                };

                let mut cmd = StdCommand::new(&path);
                cmd.env("RABBIT_ELEVATED", "1");

                if let Ok(d) = std::env::var("DISPLAY") {
                    cmd.env("DISPLAY", d);
                }

                if let Ok(x) = std::env::var("XAUTHORITY") {
                    cmd.env("XAUTHORITY", x);
                }

                match ElevatedCommand::new(cmd).output() {
                    Ok(o) => {
                        if o.status.success() {
                            std::process::exit(0);
                        }
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        let code = o.status.code().unwrap_or(-1);

                        if code == 126 && stderr.contains("Request dismissed") {
                            let _ = StdCommand::new("zenity").args(&["--error", "--title", "elevation failed", "--text", "用户取消授权"]).spawn();
                            std::process::exit(1);
                        }
                        if attempt < max_retries && (code == 127 || stderr.contains("Not authorized")) {
                            thread::sleep(Duration::from_millis(1500));
                            continue;
                        }

                        let _ = StdCommand::new("zenity")
                            .args(&["--error", "--title", "elevation failed", "--text", &format!("error: {}", o.status)])
                            .spawn();
                        std::process::exit(1);
                    }
                    Err(e) => {
                        if attempt < max_retries {
                            thread::sleep(Duration::from_millis(1500));
                            continue;
                        }
                        let _ = StdCommand::new("zenity").args(&["--error", "--title", "elevation failed", "--text", &format!("error: {}", e)]).spawn();
                        std::process::exit(1);
                    }
                }
            }

            #[cfg(target_os = "windows")]
            {
                use elevated_command::Command as ElevatedCommand;
                use std::ffi::OsStr;
                use std::os::windows::ffi::OsStrExt;
                use std::process::Command as StdCommand;
                use std::thread;
                use std::time::Duration;
                use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

                let path = {
                    let r = std::env::var("APPIMAGE").unwrap_or_else(|_| std::env::args().next().unwrap_or_else(|| "unknown".to_string()));
                    std::path::Path::new(&r).canonicalize().unwrap_or_else(|_| std::path::PathBuf::from(&r)).to_string_lossy().into_owned()
                };

                let mut cmd = StdCommand::new(&path);
                cmd.env("RABBIT_ELEVATED", "1");

                match ElevatedCommand::new(cmd).output() {
                    Ok(o) => {
                        if o.status.success() {
                            std::process::exit(0);
                        }
                        let code = o.status.code().unwrap_or(-1);

                        if attempt < max_retries && code == 5 {
                            thread::sleep(Duration::from_millis(1500));
                            continue;
                        }

                        let tw: Vec<u16> = OsStr::new("elevation failed").encode_wide().chain(std::iter::once(0)).collect();
                        let mw: Vec<u16> = OsStr::new(&format!("error: {}", o.status)).encode_wide().chain(std::iter::once(0)).collect();
                        unsafe {
                            MessageBoxW(std::ptr::null_mut(), mw.as_ptr(), tw.as_ptr(), MB_OK | MB_ICONERROR);
                        }
                        std::process::exit(1);
                    }
                    Err(e) => {
                        if attempt < max_retries {
                            thread::sleep(Duration::from_millis(1500));
                            continue;
                        }
                        let tw: Vec<u16> = OsStr::new("elevation failed").encode_wide().chain(std::iter::once(0)).collect();
                        let mw: Vec<u16> = OsStr::new(&format!("error: {}", e)).encode_wide().chain(std::iter::once(0)).collect();
                        unsafe {
                            MessageBoxW(std::ptr::null_mut(), mw.as_ptr(), tw.as_ptr(), MB_OK | MB_ICONERROR);
                        }
                        std::process::exit(1);
                    }
                }
            }

            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                std::process::exit(1);
            }
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command as StdCommand;
            let _ = StdCommand::new("zenity").args(&["--error", "--title", "elevation failed", "--text", "max retries exceeded"]).spawn();
        }
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
            let tw: Vec<u16> = OsStr::new("elevation failed").encode_wide().chain(std::iter::once(0)).collect();
            let mw: Vec<u16> = "max retries exceeded".encode_utf16().chain(std::iter::once(0)).collect();
            unsafe {
                MessageBoxW(std::ptr::null_mut(), mw.as_ptr(), tw.as_ptr(), MB_OK | MB_ICONERROR);
            }
        }
        std::process::exit(1);
    }
}
