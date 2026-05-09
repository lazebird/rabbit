//! Tray helper process for elevated (root) runs on Linux.
//!
//! When rabbit needs to elevate to root, the original (non-elevated) process
//! spawns a helper subprocess that owns the D-Bus tray icon via ksni — root
//! cannot register a StatusNotifierItem on the user's session bus because the
//! D-Bus daemon rejects connections from a different UID.
//!
//! The elevated parent communicates with the helper via a Unix domain socket.
//! The socket path is deterministic: `/tmp/rabbit-tray-{UID}.sock` where UID
//! comes from `SUDO_UID` (set by sudo) in the elevated process, or from the
//! current UID in the helper process.
//!
//! # Tray hide/show (avoiding kill)
//!
//! Previously the helper was killed on tray disable and re-spawning was
//! impossible because the root process cannot connect to D-Bus.  Now the
//! helper stays alive for the entire app session and accepts HIDE/SHOW
//! commands from the parent via the socket — the helper simply drops or
//! re-creates the ksni handle in response.
//!
//! `shutdown_helper()` is only called on full app exit.

use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Diag logging helper (writes to /tmp/rabbit-startup-*.log)
// ---------------------------------------------------------------------------
macro_rules! diag {
    ($($arg:tt)*) => {
        adapter::diag::log(&format!("[tray-helper] {}", format_args!($($arg)*)))
    };
}

// ---------------------------------------------------------------------------
// IPC protocol — fixed-size little-endian message tags
// ---------------------------------------------------------------------------

/// Helper → Parent: tray icon was clicked (show/hide toggle).
const TAG_ACTIVATE: u32 = 0x01;
/// Helper → Parent: user clicked tray "Show".
const TAG_MENU_SHOW: u32 = 0x02;
/// Helper → Parent: user clicked tray "Hide".
const TAG_MENU_HIDE: u32 = 0x03;
/// Helper → Parent: user clicked tray "Quit" (full app exit).
const TAG_MENU_QUIT: u32 = 0x04;

/// Parent → Helper: hide the tray icon (keep helper alive).
const TAG_TRAY_HIDE: u32 = 0x10;
/// Parent → Helper: show the tray icon (re-create ksni handle).
const TAG_TRAY_SHOW: u32 = 0x11;

// ---------------------------------------------------------------------------
// Init-once: embed icon bytes so the helper always finds it regardless of cwd
// ---------------------------------------------------------------------------

static EMBEDDED_ICON: &[u8] = include_bytes!("../resources/icon.ico");

// ---------------------------------------------------------------------------
// Socket path helpers
// ---------------------------------------------------------------------------

fn socket_path_for(uid: u32) -> String {
    format!("/tmp/rabbit-tray-{uid}.sock")
}

fn helper_socket_path() -> String {
    socket_path_for(unsafe { libc::getuid() })
}

fn parent_socket_path() -> Option<String> {
    let uid_str = std::env::var("SUDO_UID").ok()
        .or_else(|| std::env::var("PKEXEC_UID").ok())?;
    let uid: u32 = uid_str.parse().ok()?;
    Some(socket_path_for(uid))
}

// ---------------------------------------------------------------------------
// Globals (parent side)
// ---------------------------------------------------------------------------

/// Set to `true` once the helper accepts (or the parent connects).
static HELPER_CONNECTED: AtomicBool = AtomicBool::new(false);

/// Cloned write-end of the parent→helper socket for sending commands.
static HELPER_STREAM: Mutex<Option<UnixStream>> = Mutex::new(None);

/// Tracks whether the helper's tray is currently visible.
/// Written optimistically by hide_tray()/show_tray() on the parent side.
static HELPER_TRAY_VISIBLE: AtomicBool = AtomicBool::new(false);

/// Monotonic counter so each ksni spawn gets a unique tray ID.
static TRAY_SPAWN_COUNT: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// `true` once the helper has accepted a parent connection (helper side) or
/// the parent has successfully connected and started the listener (parent side).
pub fn is_connected() -> bool {
    HELPER_CONNECTED.load(Ordering::SeqCst)
}

/// Whether the helper is currently showing a tray icon.
pub fn is_tray_visible() -> bool {
    HELPER_TRAY_VISIBLE.load(Ordering::SeqCst)
}

/// Whether a helper socket exists on the filesystem (checked from the
/// elevated process after restart).
pub fn has_helper_socket() -> bool {
    parent_socket_path()
        .as_deref()
        .map(Path::new)
        .is_some_and(|p| p.exists())
}

/// Spawn the tray helper subprocess (called **before** elevation).
pub fn spawn() -> io::Result<()> {
    let socket_path = helper_socket_path();
    let _ = std::fs::remove_file(&socket_path);

    let exe = std::env::current_exe()?;
    info!("Spawning tray helper: {exe:?} --tray-helper");
    std::process::Command::new(&exe)
        .arg("--tray-helper")
        .spawn()?;
    Ok(())
}

/// Tray helper entry point (called from `main.rs` when `--tray-helper` is present).
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = helper_socket_path();
    let _ = std::fs::remove_file(&socket_path);

    let listener = UnixListener::bind(&socket_path)?;
    diag!("socket bound");
    std::fs::set_permissions(
        &socket_path,
        std::os::unix::fs::PermissionsExt::from_mode(0o666),
    )?;
    info!("Tray helper listening on {socket_path}");

    let (event_tx, event_rx) = mpsc::channel::<u32>();
    let tray_handle: Arc<Mutex<Option<ksni::Handle<HelperTray>>>> =
        Arc::new(Mutex::new(None));

    // Tray creation is deferred until the parent sends TAG_TRAY_SHOW.
    // This prevents the tray icon from appearing during the authorization
    // (sudo password) phase or when the user has disabled the systray.

    // ── accept connection from elevated parent ──
    diag!("accept: waiting for parent");
    let stream = accept_with_timeout(&listener, Duration::from_secs(30))?;
    diag!("accept: parent connected");
    info!("Tray helper: elevated parent connected");

    // Split stream into reader (for parent→helper commands)
    // and writer (for helper→parent events).
    let mut cmd_reader = stream.try_clone()?;
    let mut event_writer = stream;

    // ── command reader thread: handles HIDE / SHOW from parent ──
    let cmd_tray_handle = tray_handle.clone();
    let cmd_event_tx = event_tx.clone();
    std::thread::Builder::new()
        .name("tray-helper-commands".into())
        .spawn(move || {
            let mut cmd_buf = [0u8; 4];
            loop {
                cmd_buf.fill(0);
                match cmd_reader.read_exact(&mut cmd_buf) {
                    Ok(()) => {
                        let tag = u32::from_le_bytes(cmd_buf);
                        match tag {
                            TAG_TRAY_HIDE => {
                                info!("Tray helper: hiding tray");
                                if let Ok(mut guard) = cmd_tray_handle.lock() {
                                    if let Some(handle) = guard.take() {
                                        // Must not await — the ksni runtime runs on
                                        // a separate thread and we are in a command
                                        // reader thread that must not block.
                                        #[allow(clippy::let_underscore_future)]
                                        let _ = handle.shutdown();
                                    }
                                }
                            }
                            TAG_TRAY_SHOW => {
                                info!("Tray helper: showing tray");
                                spawn_tray_inner(
                                    cmd_tray_handle.clone(),
                                    cmd_event_tx.clone(),
                                );
                            }
                            _ => {
                                warn!("Tray helper: unknown command: {tag:#x}");
                            }
                        }
                    }
                    Err(e) => {
                        info!("Tray helper: command connection closed: {e}");
                        // Shut down the tray so the icon disappears immediately.
                        if let Ok(mut guard) = cmd_tray_handle.lock() {
                            if let Some(handle) = guard.take() {
                                #[allow(clippy::let_underscore_future)]
                                let _ = handle.shutdown();
                            }
                        }
                        // Signal forward_events to exit — the tray thread may
                        // still hold a channel sender (blocked on ctrl_c),
                        // preventing channel closure from unblocking recv().
                        let _ = cmd_event_tx.send(TAG_MENU_QUIT);
                        break;
                    }
                }
            }
        })?;

    // Drop our sender so the channel closes once all clones are gone
    // (reader thread, tray).  Without this, forward_events would block
    // forever on recv() because we hold a sender alive in this scope.
    drop(event_tx);

    // ── forward events to parent (blocking writes) ──
    let result = forward_events(&mut event_writer, &event_rx);

    let _ = std::fs::remove_file(&socket_path);
    info!("Tray helper exiting");
    result.map_err(|e| e.into())
}

/// Connect to the helper from the elevated parent and start the event listener.
///
/// Returns `true` on success.  The caller should skip `systray::init_systray`
/// when this returns `true`.
pub fn connect<F1, F2>(
    lifecycle: crate::lifecycle::Lifecycle,
    show_win: F1,
    hide_win: F2,
) -> bool
where
    F1: Fn() + Send + Clone + 'static,
    F2: Fn() + Send + Clone + 'static,
{
    let stream = match try_connect_inner() {
        Some(s) => s,
        None => {
            diag!("connect: returning false (try_connect_inner failed)");
            return false;
        }
    };

    // Keep a write-end for later commands and shutdown.
    if let Ok(clone) = stream.try_clone() {
        if let Ok(mut guard) = HELPER_STREAM.lock() {
            *guard = Some(clone);
        }
    }

    diag!("connect: spawning event listener thread");
    start_event_listener(stream, lifecycle, show_win, hide_win);
    diag!("connect: returning true");
    true
}

/// Tell the helper to hide the tray (keeps the helper alive).
pub fn hide_tray() {
    HELPER_TRAY_VISIBLE.store(false, Ordering::SeqCst);
    send_command(TAG_TRAY_HIDE);
}

/// Tell the helper to show the tray icon again.
pub fn show_tray() {
    HELPER_TRAY_VISIBLE.store(true, Ordering::SeqCst);
    send_command(TAG_TRAY_SHOW);
}

/// Ask the helper to shut down (tray icon disappears immediately).
///
/// Shuts down the Unix socket so the helper's read/write fails, causing
/// the helper process to exit cleanly.
///
/// Called ONLY on full app exit (event loop cleanup), NOT on tray toggle.
pub fn shutdown_helper() {
    diag!("shutdown_helper: shutting down socket");
    if let Ok(mut guard) = HELPER_STREAM.lock() {
        if let Some(stream) = guard.take() {
            let r = stream.shutdown(std::net::Shutdown::Both);
            diag!("shutdown_helper: socket shutdown result={r:?}");
        } else {
            diag!("shutdown_helper: HELPER_STREAM was None (already taken)");
        }
    } else {
        diag!("shutdown_helper: HELPER_STREAM lock failed");
    }

    HELPER_CONNECTED.store(false, Ordering::SeqCst);
    HELPER_TRAY_VISIBLE.store(false, Ordering::SeqCst);
    diag!("shutdown_helper: done, HELPER_CONNECTED=false");
}

// ---------------------------------------------------------------------------
// Internal — parent side
// ---------------------------------------------------------------------------

fn send_command(tag: u32) {
    if let Ok(mut guard) = HELPER_STREAM.lock() {
        if let Some(stream) = guard.as_mut() {
            let buf = tag.to_le_bytes();
            let _ = stream.write_all(&buf);
            let _ = stream.flush();
        }
    }
}

fn try_connect_inner() -> Option<UnixStream> {
    let path = parent_socket_path()?;
    let path = Path::new(&path);
    if !path.exists() {
        warn!("Tray helper socket not found at {path:?}");
        diag!("socket not found at {path:?}");
        return None;
    }
    diag!("connect: socket exists, calling UnixStream::connect");
    match UnixStream::connect(path) {
        Ok(stream) => {
            info!("Connected to tray helper at {path:?}");
            diag!("connect: success");
            HELPER_CONNECTED.store(true, Ordering::SeqCst);
            Some(stream)
        }
        Err(e) => {
            warn!("Failed to connect to tray helper at {path:?}: {e}");
            diag!("connect: failed ({e})");
            None
        }
    }
}

fn start_event_listener<F1, F2>(
    stream: UnixStream,
    lifecycle: crate::lifecycle::Lifecycle,
    show_win: F1,
    hide_win: F2,
) where
    F1: Fn() + Send + Clone + 'static,
    F2: Fn() + Send + Clone + 'static,
{
    std::thread::Builder::new()
        .name("tray-helper-reader".into())
        .spawn(move || {
            let mut reader = stream;
            let mut tag_buf = [0u8; 4];
            loop {
                tag_buf.fill(0);
                if let Err(e) = reader.read_exact(&mut tag_buf) {
                    info!("Tray helper connection closed: {e}");
                    break;
                }
                let tag = u32::from_le_bytes(tag_buf);
                let lc = lifecycle.clone();
                match tag {
                    TAG_ACTIVATE | TAG_MENU_SHOW => {
                        let f = show_win.clone();
                        fltk::app::awake_callback(f);
                    }
                    TAG_MENU_HIDE => {
                        let f = hide_win.clone();
                        fltk::app::awake_callback(f);
                    }
                    TAG_MENU_QUIT => {
                        info!("Tray helper requested shutdown");
                        lc.request_shutdown();
                        fltk::app::awake_callback(|| {
                            fltk::app::quit();
                        });
                        break;
                    }
                    _ => {
                        warn!("Unknown tray helper message tag: {tag:#x}");
                    }
                }
            }
            HELPER_CONNECTED.store(false, Ordering::SeqCst);
            HELPER_TRAY_VISIBLE.store(false, Ordering::SeqCst);
            let _ = HELPER_STREAM.lock().map(|mut g| *g = None);
        })
        .ok();
}

// ---------------------------------------------------------------------------
// Internal — helper side (socket + event forwarder)
// ---------------------------------------------------------------------------

fn spawn_tray_inner(
    handle: Arc<Mutex<Option<ksni::Handle<HelperTray>>>>,
    sender: mpsc::Sender<u32>,
) {
    let spawn_n = TRAY_SPAWN_COUNT.fetch_add(1, Ordering::Relaxed);
    std::thread::Builder::new()
        .name(format!("tray-helper-ksni-{spawn_n}"))
        .spawn(move || {
            let icon = load_icon();
            let tray = HelperTray {
                icon,
                sender,
                spawn_id: spawn_n,
            };

            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    error!("tray helper: failed to build tokio runtime: {e}");
                    return;
                }
            };

            rt.block_on(async {
                use ksni::TrayMethods;
                match tray.assume_sni_available(true).spawn().await {
                    Ok(h) => {
                        info!("Tray helper: ksni tray spawned (id={spawn_n})");
                        if let Ok(mut guard) = handle.lock() {
                            *guard = Some(h);
                        }
                        tokio::signal::ctrl_c().await.ok();
                    }
                    Err(e) => {
                        warn!("Tray helper: ksni spawn failed (id={spawn_n}): {e}");
                    }
                }
            });
        })
        .ok();
}

fn accept_with_timeout(listener: &UnixListener, timeout: Duration) -> io::Result<UnixStream> {
    use std::os::unix::io::AsRawFd;

    let fd = listener.as_raw_fd();
    let mut flags = unsafe { libc::fcntl(fd, libc::F_GETFL, 0) };
    if flags < 0 {
        flags = 0;
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }

    let deadline = std::time::Instant::now() + timeout;
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                unsafe { libc::fcntl(fd, libc::F_SETFL, flags) };
                HELPER_CONNECTED.store(true, Ordering::SeqCst);
                return Ok(stream);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() > deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "timed out waiting for parent",
                    ));
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => {
                unsafe { libc::fcntl(fd, libc::F_SETFL, flags) };
                return Err(e);
            }
        }
    }
}

fn forward_events(writer: &mut UnixStream, rx: &mpsc::Receiver<u32>) -> io::Result<()> {
    while let Ok(tag) = rx.recv() {
        let buf = tag.to_le_bytes();
        writer.write_all(&buf)?;
        writer.flush()?;
        if tag == TAG_MENU_QUIT {
            std::thread::sleep(Duration::from_millis(500));
            break;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// ksni tray
// ---------------------------------------------------------------------------

struct HelperTray {
    icon: ksni::Icon,
    sender: mpsc::Sender<u32>,
    /// Unique spawn ID so multiple show/hide cycles don't collide.
    spawn_id: u32,
}

impl ksni::Tray for HelperTray {
    fn id(&self) -> String {
        format!("rabbit-tray-helper-{}", self.spawn_id)
    }

    fn title(&self) -> String {
        "Rabbit - Network Tools".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![self.icon.clone()]
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        vec![
            StandardItem {
                label: "Show".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.sender.send(TAG_MENU_SHOW);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Hide".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.sender.send(TAG_MENU_HIDE);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.sender.send(TAG_MENU_QUIT);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(TAG_ACTIVATE);
    }
}

// ---------------------------------------------------------------------------
// Icon loading — embedded bytes, no reliance on cwd
// ---------------------------------------------------------------------------

fn load_icon() -> ksni::Icon {
    if let Some(icon) = load_icon_from_bytes(EMBEDDED_ICON) {
        return icon;
    }
    warn!("Embedded icon decode failed, using fallback icon");
    make_fallback_icon()
}

fn load_icon_from_bytes(data: &[u8]) -> Option<ksni::Icon> {
    let img = image::load_from_memory(data).ok()?;
    let img = img.into_rgba8();
    let (width, height) = img.dimensions();
    let mut rgba = img.into_raw();
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.rotate_right(1);
    }
    Some(ksni::Icon {
        width: width as i32,
        height: height as i32,
        data: rgba,
    })
}

fn make_fallback_icon() -> ksni::Icon {
    let w = 32u32;
    let h = 32u32;
    let cx = 16.0f64;
    let cy = 16.0;
    let r = 14.0;
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= r {
                rgba.extend_from_slice(&[0x33, 0x99, 0xFF, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    let mut data = rgba;
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1);
    }
    ksni::Icon {
        width: w as i32,
        height: h as i32,
        data,
    }
}
