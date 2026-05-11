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

pub fn ensure_elevated() {
    // Debug / Android / iOS: skip elevation
    #[cfg(any(debug_assertions, target_os = "android", target_os = "ios"))]
    {
        // skip
    }

    #[cfg(not(any(debug_assertions, target_os = "android", target_os = "ios")))]
    {
        if is_elevated() {
            return;
        }

        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from(std::env::args().next().unwrap_or_default()))
            .to_string_lossy()
            .into_owned();

        // Linux: sudo with VAR=value forwarding (preserves DISPLAY etc.)
        // Other:  xelevate native (UAC on Windows)
        #[cfg(target_os = "linux")]
        elevate_with_sudo(&exe_path);

        #[cfg(not(target_os = "linux"))]
        elevate_with_xelevate(&exe_path);
    }
}

// ---------------------------------------------------------------------------
// Linux: sudo -S + VAR=value forwarding
// ---------------------------------------------------------------------------

/// Elevate via `sudo -S` using xelevate's password dialog, with critical GUI
/// environment variables forwarded via `VAR=value` syntax (works even when
/// `env_reset` is enabled and SETENV is not granted).
#[cfg(all(
    target_os = "linux",
    not(any(debug_assertions, target_os = "android", target_os = "ios"))
))]
fn elevate_with_sudo(exe_path: &str) -> ! {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};

    rabbit_diag::log("elevate_with_sudo: requesting password");
    let password = match xelevate::request_password() {
        Some(p) => p,
        None => {
            show_elevation_error("Elevation cancelled by user.");
            std::process::exit(1);
        }
    };
    rabbit_diag::log("elevate_with_sudo: password obtained");

    // ── Build sudo command ──────────────────────────────────────────────
    // sudo -S -k VAR=value1 VAR=value2 ... ./rabbit
    //
    // VAR=value placed BEFORE the command path bypasses env_reset even
    // without the SETENV sudoers tag.  This is the only portable way to
    // inject environment into the root process.
    // Forward critical GUI environment variables via sudo's VAR=value
    // syntax (bypasses env_reset even without SETENV sudoers tag).
    //
    // NOTE on XAUTHORITY: if the original environment doesn't explicitly
    // set XAUTHORITY, the X11 library falls back to $HOME/.Xauthority.
    // But sudo changes HOME to /root, so the fallback looks for
    // /root/.Xauthority — which doesn't exist.  We handle this below by
    // deriving XAUTHORITY from the original HOME when it's missing.
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

    // If XAUTHORITY wasn't in the original environment (or was empty),
    // derive it from $HOME/.Xauthority.  This prevents the X library's
    // fallback from looking at /root/.Xauthority after sudo changes HOME.
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
    cmd.stderr(Stdio::piped());      // Capture errors for diagnostics

    rabbit_diag::log("elevate_with_sudo: spawning sudo");
    let mut child = cmd.spawn().unwrap_or_else(|e| {
        show_elevation_error(&format!("Failed to spawn sudo: {e}"));
        std::process::exit(1);
    });

    // Pipe the password to sudo -S
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(format!("{password}\n").as_bytes());
    }

    // Wait for sudo to finish (with a longer timeout)
    let timeout_ms = 5000u64;
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

    let exit_status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if std::time::Instant::now() > deadline {
                    rabbit_diag::log("elevate_with_sudo: sudo still running after 5s, assuming success");
                    std::process::exit(0);
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(e) => {
                rabbit_diag::log(&format!("elevate_with_sudo: sudo try_wait error: {e}"));
                break None;
            }
        }
    };

    match exit_status {
        Some(status) if status.success() => {
            rabbit_diag::log("elevate_with_sudo: sudo success, exiting");
            std::process::exit(0);
        }
        Some(status) => {
            // Capture stderr for diagnostics
            let mut stderr_buf = String::new();
            let _ = child.stderr.take().map(|mut s| s.read_to_string(&mut stderr_buf));
            let code = status.code();
            rabbit_diag::log(&format!("elevate_with_sudo: sudo failed code={code:?} stderr={stderr_buf:?}"));

            show_elevation_error(&format!(
                "sudo exited with code {code:?}.\n\n\
                 stderr: {stderr_buf}\n\n\
                 Make sure you have sudo access and the password is correct.\n\
                 Try running from a terminal to see the exact error:\n  {exe_path}",
            ));
            std::process::exit(1);
        }
        None => {
            rabbit_diag::log("elevate_with_sudo: sudo wait error");
            show_elevation_error("Failed to wait for sudo. Please try running from a terminal.");
            std::process::exit(1);
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
fn elevate_with_xelevate(exe_path: &str) -> ! {
    match xelevate::elevate(exe_path) {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            show_elevation_error(&format!("{e}"));
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Error dialog (FLTK)
// ---------------------------------------------------------------------------

/// Show a modal FLTK error dialog and wait for the user to click OK.
#[cfg(not(any(debug_assertions, target_os = "android", target_os = "ios")))]
fn show_elevation_error(msg: &str) {
    rabbit_diag::log("show_elevation_error: creating FLTK dialog");
    use fltk::prelude::*;

    let app = fltk::app::App::default();
    let mut wind =
        fltk::window::Window::default().with_size(400, 180).with_label("Elevation Failed");
    wind.make_modal(true);

    let mut msg_box =
        fltk::text::TextDisplay::default().with_pos(20, 20).with_size(360, 100);
    msg_box.set_buffer(Some(fltk::text::TextBuffer::default()));
    msg_box.buffer().unwrap().set_text(msg);

    let mut btn =
        fltk::button::Button::default().with_pos(160, 140).with_size(80, 30).with_label("OK");
    btn.set_callback({
        let mut w = wind.clone();
        move |_| w.hide()
    });

    wind.end();
    rabbit_diag::log("show_elevation_error: showing dialog");
    wind.show();
    rabbit_diag::log("show_elevation_error: entering FLTK event loop");
    app.run().unwrap();
    rabbit_diag::log("show_elevation_error: dialog closed");
}

#[cfg(test)]
mod tests {
    // Keep empty test module to avoid "unused module" warning.
}
