//! Rabbit Diagnostic Logging
//!
//! Provides a simple timestamped diagnostic log to `/tmp/rabbit-startup-{pid}.log`.
//! Extracted from `adapter::diag` as part of platform layer refactoring.
//! Uses `std::process::id()` instead of `libc::getpid()` to eliminate platform dependency.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

fn start() -> &'static Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now)
}

fn log_file() -> &'static Mutex<std::fs::File> {
    static LOG: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
    LOG.get_or_init(|| {
        let path = format!("/tmp/rabbit-startup-{}.log", std::process::id());
        let file = OpenOptions::new().create(true).append(true).open(&path).expect("cannot open diagnostic log");
        Mutex::new(file)
    })
}

/// Write a timestamped diagnostic line to `/tmp/rabbit-startup-{pid}.log`.
///
/// Example output: `[+0.003] pid=12345 entering main()`
pub fn log(msg: &str) {
    let elapsed = start().elapsed();
    let secs = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
    let pid = std::process::id();
    let line = format!("[{:+.3}] pid={} {msg}\n", secs, pid);

    if let Ok(f) = log_file().lock() {
        let _ = (&*f).write_all(line.as_bytes());
    }
}
