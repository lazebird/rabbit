//! Linux system tray using ksni (pure Rust StatusNotifierItem via D-Bus).
//!
//! ksni speaks the KDE/freedesktop StatusNotifierItem protocol directly through
//! zbus — no C library dependency (no libappindicator), works on Wayland and X11.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tracing::warn;

use crate::lifecycle::Lifecycle;
use fltk::prelude::WidgetExt;
use ksni::{Handle, TrayMethods};

// ---------------------------------------------------------------------------
// Diag logging helper (writes to /tmp/rabbit-startup-*.log even before tracing)
// ---------------------------------------------------------------------------
macro_rules! diag {
    ($($arg:tt)*) => {
        rabbit_diag::log(&format!("[systray] {}", format_args!($($arg)*)))
    };
}

static SYSTRAY_ENABLED: AtomicBool = AtomicBool::new(false);
static TRAY_HANDLE: Mutex<Option<Handle<RabbitTray>>> = Mutex::new(None);
static MAIN_WIN: Mutex<Option<fltk::window::Window>> = Mutex::new(None);
static LIFECYCLE: Mutex<Option<Lifecycle>> = Mutex::new(None);

struct RabbitTray {
    icon: ksni::Icon,
    lifecycle: Lifecycle,
}

impl ksni::Tray for RabbitTray {
    fn id(&self) -> String {
        "rabbit".into()
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
                activate: Box::new(|_: &mut Self| {
                    fltk::app::awake_callback(show_main_window);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Hide".into(),
                activate: Box::new(|_: &mut Self| {
                    fltk::app::awake_callback(hide_main_window);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|tray: &mut Self| {
                    tray.lifecycle.request_shutdown();
                    fltk::app::awake_callback(|| {
                        fltk::app::quit();
                    });
                }),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        fltk::app::awake_callback(show_main_window);
    }
}

fn show_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            let mut w = win.clone();
            w.show();
        }
    }
}

fn hide_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            let mut w = win.clone();
            w.hide();
        }
    }
}

fn load_icon() -> ksni::Icon {
    let data = crate::icon::load_app_icon();
    let mut rgba = data.rgba;
    // ksni::Icon uses ARGB32 (network byte order), image crate gives RGBA
    for pixel in rgba.chunks_exact_mut(4) {
        pixel.rotate_right(1);
    }
    ksni::Icon {
        width: data.width as i32,
        height: data.height as i32,
        data: rgba,
    }
}

/// Ensure `LIFECYCLE` is populated even when the helper handles the tray at
/// startup (in which case `init_systray` is never called and `LIFECYCLE` stays
/// `None`).  Without this, `update_systray(true)` on re-enable silently does
/// nothing and the tray never reappears.
pub fn set_lifecycle(lifecycle: &Lifecycle) {
    if let Ok(mut guard) = LIFECYCLE.lock() {
        if guard.is_none() {
            *guard = Some(lifecycle.clone());
            diag!("set_lifecycle: stored");
        }
    }
}

pub fn set_main_window(win: fltk::window::Window) {
    if let Ok(mut store) = MAIN_WIN.lock() {
        *store = Some(win);
    }
}

pub fn is_active() -> bool {
    if crate::tray_helper::is_connected() {
        // Helper manages the tray — report actual visibility, not just
        // connection state.  When the tray is hidden via hide_tray() the
        // helper stays connected but is_tray_visible() returns false.
        crate::tray_helper::is_tray_visible()
    } else {
        SYSTRAY_ENABLED.load(Ordering::SeqCst)
    }
}

pub async fn init_systray(lifecycle: Lifecycle) -> Result<(), String> {
    diag!("init_systray: start");
    remove_systray();

    // Store lifecycle for later init_systray calls (e.g. from update_systray)
    if let Ok(mut guard) = LIFECYCLE.lock() {
        *guard = Some(lifecycle.clone());
    }

    let icon = load_icon();
    let tray = RabbitTray { icon, lifecycle };

    diag!("init_systray: spawning ksni");
    let handle = tray
        .assume_sni_available(true)
        .spawn()
        .await
        .map_err(|e| {
            diag!("init_systray: spawn error: {e}");
            format!("ksni spawn failed: {e}")
        })?;

    if let Ok(mut guard) = TRAY_HANDLE.lock() {
        *guard = Some(handle);
    }
    SYSTRAY_ENABLED.store(true, Ordering::SeqCst);
    diag!("init_systray: done, SYSTRAY_ENABLED=true");
    Ok(())
}

pub fn remove_systray() {
    if let Ok(mut guard) = TRAY_HANDLE.lock() {
        if let Some(handle) = guard.take() {
            // Send the shutdown message through the unbounded channel (immediate,
            // non-blocking) and let the service task process it when it gets
            // polled by the runtime.  Do NOT await the oneshot — the event-loop
            // worker thread is about to block on recv() and any task we spawn
            // here would land on the same thread's local queue and never run.
            //
            // Clippy's let_underscore_future is suppressed because the
            // ShutdownAwaiter must NOT be awaited (see comment above).
            #[allow(clippy::let_underscore_future)]
            let _ = handle.shutdown();
        }
    }
    SYSTRAY_ENABLED.store(false, Ordering::SeqCst);
}

pub async fn update_systray(enabled: bool) {
    diag!("update_systray(enabled={enabled})");
    // When tray helper is active, the helper controls the tray independently.
    if crate::tray_helper::is_connected() {
        diag!("update_systray: helper connected, skipping");
        return;
    }
    let currently_active = is_active();
    diag!("update_systray: currently_active={currently_active}, SYSTRAY_ENABLED={}", SYSTRAY_ENABLED.load(Ordering::SeqCst));
    if enabled && !currently_active {
        let lifecycle = LIFECYCLE.lock().ok().and_then(|g| g.clone());
        match lifecycle {
            Some(lc) => {
                diag!("update_systray: calling init_systray");
                if let Err(e) = init_systray(lc).await {
                    warn!("Failed to init system tray: {e}");
                    diag!("update_systray: init_systray error: {e}");
                }
            }
            None => {
                warn!("Cannot init systray: no Lifecycle set");
                diag!("update_systray: LIFECYCLE is None!");
            }
        }
    } else if !enabled && currently_active {
        diag!("update_systray: calling remove_systray");
        remove_systray();
    }
}
