//! Privilege elevation.
//!
//! Uses `xelevate` for privilege checking and password dialog, but replaces
//! the internal `sudo -S` invocation with `sudo -S` + explicit `VAR=value`
//! forwarding for critical GUI environment variables (DISPLAY, XAUTHORITY,
//! D-Bus session bus address) so they survive into the elevated (root)
//! process — otherwise FLTK cannot create windows and the GUI stays invisible.
//!
//! **Why `VAR=value` instead of `-E` / `--preserve-env`:**
//! `sudo -E` requires the `SETENV` tag in sudoers, which is not enabled by
//! default on many systems.  `sudo VAR=value command` is a different syntax
//! that the sudoers policy treats as part of the command invocation and is
//! NOT subject to `env_keep` / `env_reset` — it always works.

/// Errors that can occur during privilege elevation.
#[derive(Debug)]
pub enum ElevationError {
    /// User cancelled the password dialog.
    Cancelled,
    /// sudo process failed with an error.
    SudoFailed(String),
    /// Failed to spawn the elevation helper.
    SpawnFailed(String),
}

impl std::fmt::Display for ElevationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElevationError::Cancelled => write!(f, "Elevation cancelled by user"),
            ElevationError::SudoFailed(msg) => write!(f, "sudo failed: {msg}"),
            ElevationError::SpawnFailed(msg) => write!(f, "Elevation spawn failed: {msg}"),
        }
    }
}

/// Returns `true` when the current process already has elevated privileges.
pub fn is_elevated() -> bool {
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let _ = true;
        false
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        xelevate::is_elevated()
    }
}

/// Ensure the process has elevated privileges.
///
/// - 当已提权时返回 `Ok(())`。
/// - 需要提权时内部调用 sudo/UAC，成功则 `process::exit(0)` 以 root 身份重启进程，
///   **不会返回**到调用方。
/// - 提权失败时返回 `Err(ElevationError)`。
pub fn ensure_elevated() -> Result<(), ElevationError> {
    // Debug / Android / iOS: skip elevation
    #[cfg(any(debug_assertions, target_os = "android", target_os = "ios"))]
    {
        return Ok(());
    }

    #[cfg(not(any(debug_assertions, target_os = "android", target_os = "ios")))]
    {
        if is_elevated() {
            return Ok(());
        }

        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from(std::env::args().next().unwrap_or_default()))
            .to_string_lossy()
            .into_owned();

        // Linux: sudo with VAR=value forwarding (preserves DISPLAY etc.)
        // Other:  xelevate native (UAC on Windows)
        #[cfg(target_os = "linux")]
        return elevate_with_sudo_inner(&exe_path);

        #[cfg(not(target_os = "linux"))]
        return elevate_with_xelevate_inner(&exe_path);
    }
}

// ---------------------------------------------------------------------------
// Linux: sudo -S + VAR=value forwarding
// ---------------------------------------------------------------------------

/// Elevate via `sudo -S` using xelevate's password dialog, with critical GUI
/// environment variables forwarded via `VAR=value` syntax (works even when
/// `env_reset` is enabled and SETENV is not granted).
///
/// 成功时内部 `process::exit(0)`（子进程已启动），失败时返回 `Err`。
#[cfg(all(
    target_os = "linux",
    not(any(debug_assertions, target_os = "android", target_os = "ios"))
))]
fn elevate_with_sudo_inner(exe_path: &str) -> Result<(), ElevationError> {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};

    rabbit_diag::log("elevate_with_sudo_inner: requesting password");
    let password = match xelevate::request_password() {
        Some(p) => p,
        None => {
            rabbit_diag::log("elevate_with_sudo_inner: password cancelled");
            return Err(ElevationError::Cancelled);
        }
    };
    rabbit_diag::log("elevate_with_sudo_inner: password obtained");

    let gui_vars = ["DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY",
                    "DBUS_SESSION_BUS_ADDRESS", "XDG_RUNTIME_DIR"];

    let mut cmd = Command::new("sudo");
    cmd.arg("-S");
    cmd.arg("-k");

    for var in &gui_vars {
        if let Ok(val) = std::env::var(var) {
            if !val.is_empty() {
                let pair = format!("{var}={val}");
                cmd.arg(&pair);
                rabbit_diag::log(&format!("  sudo env: {pair}"));
            }
        }
    }

    if std::env::var("XAUTHORITY").ok().map_or(true, |v| v.is_empty()) {
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                let default_xauth = format!("{home}/.Xauthority");
                cmd.arg(format!("XAUTHORITY={default_xauth}"));
                rabbit_diag::log(&format!(
                    "  sudo env: XAUTHORITY={default_xauth} (derived from HOME)"
                ));
            }
        }
    }

    cmd.env("XELEVATE_ELEVATED", "1");
    cmd.arg(exe_path);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    rabbit_diag::log("elevate_with_sudo_inner: spawning sudo");
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            rabbit_diag::log(&format!("elevate_with_sudo_inner: spawn failed: {e}"));
            return Err(ElevationError::SpawnFailed(format!("Failed to spawn sudo: {e}")));
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(format!("{password}\n").as_bytes());
    }

    let timeout_ms = 5000u64;
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

    let exit_status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if std::time::Instant::now() > deadline {
                    rabbit_diag::log("elevate_with_sudo_inner: sudo still running after 5s, assuming success");
                    std::process::exit(0);
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(e) => {
                rabbit_diag::log(&format!("elevate_with_sudo_inner: sudo try_wait error: {e}"));
                break None;
            }
        }
    };

    match exit_status {
        Some(status) if status.success() => {
            rabbit_diag::log("elevate_with_sudo_inner: sudo success, exiting");
            std::process::exit(0);
        }
        Some(status) => {
            let mut stderr_buf = String::new();
            let _ = child.stderr.take().map(|mut s| s.read_to_string(&mut stderr_buf));
            let code = status.code();
            rabbit_diag::log(&format!("elevate_with_sudo_inner: sudo failed code={code:?} stderr={stderr_buf:?}"));
            Err(ElevationError::SudoFailed(format!(
                "sudo exited with code {code:?}.\nstderr: {stderr_buf}",
            )))
        }
        None => {
            rabbit_diag::log("elevate_with_sudo_inner: sudo wait error");
            Err(ElevationError::SudoFailed(
                "Failed to wait for sudo. Please try running from a terminal.".into(),
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Windows / other: xelevate (UAC)
// ---------------------------------------------------------------------------

#[cfg(not(any(
    debug_assertions,
    target_os = "android",
    target_os = "ios",
    target_os = "linux"
)))]
fn elevate_with_xelevate_inner(exe_path: &str) -> Result<(), ElevationError> {
    match xelevate::elevate(exe_path) {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            rabbit_diag::log(&format!("elevate_with_xelevate_inner: failed: {e}"));
            Err(ElevationError::SpawnFailed(e.to_string()))
        }
    }
}
