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
        let path = format!("/tmp/rabbit-startup-{}.log", unsafe { libc::getpid() });
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .expect("cannot open diagnostic log");
        Mutex::new(file)
    })
}

/// Write a timestamped diagnostic line to `/tmp/rabbit-startup-{pid}.log`.
///
/// Example output: `[+0.003] entering main()`
pub fn log(msg: &str) {
    let elapsed = start().elapsed();
    let secs = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
    let line = format!("[{:+.3}] pid={} {msg}\n", secs, unsafe { libc::getpid() });

    if let Ok(f) = log_file().lock() {
        let _ = (&*f).write_all(line.as_bytes());
    }
}
