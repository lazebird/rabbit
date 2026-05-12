//! Rabbit Diagnostic Logging
//!
//! Provides a simple timestamped diagnostic log to the system temp directory
//! (`rabbit-startup-{pid}.log` under [`std::env::temp_dir`]).
//! Extracted from `adapter::diag` as part of platform layer refactoring.
//! Uses `std::process::id()` instead of `libc::getpid()` to eliminate platform dependency.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

fn start() -> &'static Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now)
}

/// Compute the diagnostic log path (cross-platform).
///
/// Uses [`std::env::temp_dir`] so the path is valid on both Unix (`/tmp/`)
/// and Windows (`%TEMP%`).
fn log_path() -> PathBuf {
    std::env::temp_dir().join(format!("rabbit-startup-{}.log", std::process::id()))
}

fn log_file() -> &'static Mutex<std::fs::File> {
    static LOG: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
    LOG.get_or_init(|| {
        let path = log_path();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|e| panic!("cannot open diagnostic log at {path:?}: {e}"));
        Mutex::new(file)
    })
}

/// Write a timestamped diagnostic line to the log file in the system temp
/// directory.
///
/// Example output: `[+0.003] pid=12345 entering main()`
///
/// The path is `<temp_dir>/rabbit-startup-<pid>.log`.
pub fn log(msg: &str) {
    let elapsed = start().elapsed();
    let secs = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
    let pid = std::process::id();
    let line = format!("[{:+.3}] pid={} {msg}\n", secs, pid);

    if let Ok(f) = log_file().lock() {
        let _ = (&*f).write_all(line.as_bytes());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_path_uses_temp_dir() {
        let path = log_path();
        let expected_parent = std::env::temp_dir();
        assert!(
            path.starts_with(&expected_parent),
            "log path {path:?} should be under temp dir {expected_parent:?}",
        );
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        assert!(
            filename.starts_with("rabbit-startup-"),
            "log filename {filename:?} should start with 'rabbit-startup-'",
        );
        assert!(
            filename.ends_with(".log"),
            "log filename {filename:?} should end with '.log'",
        );
    }

    #[test]
    fn test_log_does_not_panic_on_current_platform() {
        // This is the exact scenario that crashes on Windows release builds:
        // the log file must be creatable in the system temp directory.
        // If the path is wrong (e.g. hardcoded /tmp/ on Windows), this panics.
        log("rabbit-diag test: diagnostic logging works");
        // If we reach here, the log call succeeded — the file was created.
    }

    #[test]
    fn test_log_content_format() {
        // Write a known message and verify the file contains it.
        log("rabbit-diag test: content-format-check");

        let path = log_path();
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read log file {path:?}: {e}"));

        let pid = std::process::id();
        let expected_pat = format!("pid={pid}");
        assert!(
            content.contains(&expected_pat),
            "log content should contain 'pid={pid}', got:\n{content}"
        );
        assert!(
            content.contains("content-format-check"),
            "log content should contain the test message, got:\n{content}"
        );
        // Each call produces exactly one line.
        let line_count = content.lines().count();
        assert!(
            line_count >= 1,
            "expected at least 1 log line, got {line_count}",
        );
    }
}
