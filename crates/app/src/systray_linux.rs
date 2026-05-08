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

fn load_icon_from(path: &std::path::Path) -> Option<ksni::Icon> {
    let img = image::open(path).ok()?;
    let img = img.into_rgba8();
    let (width, height) = img.dimensions();
    let mut data = img.into_raw();
    // ksni::Icon uses ARGB32 (network byte order), image crate gives RGBA
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1);
    }
    Some(ksni::Icon {
        width: width as i32,
        height: height as i32,
        data,
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

fn load_icon() -> ksni::Icon {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        let ico_path = exe_path.join("resources").join("icon.ico");
        if let Some(icon) = load_icon_from(&ico_path) {
            return icon;
        }
    }
    for path_str in &["crates/app/resources/icon.ico", "resources/icon.ico"] {
        let p = std::path::Path::new(path_str);
        if let Some(icon) = load_icon_from(p) {
            return icon;
        }
    }
    warn!("Icon file not found, using embedded fallback icon");
    make_fallback_icon()
}

pub fn set_main_window(win: fltk::window::Window) {
    if let Ok(mut store) = MAIN_WIN.lock() {
        *store = Some(win);
    }
}

pub fn is_active() -> bool {
    SYSTRAY_ENABLED.load(Ordering::SeqCst)
}

pub fn init_systray(lifecycle: Lifecycle) -> Result<(), String> {
    remove_systray();

    // Store lifecycle for later init_systray calls (e.g. from update_systray)
    if let Ok(mut guard) = LIFECYCLE.lock() {
        *guard = Some(lifecycle.clone());
    }

    let icon = load_icon();
    let tray = RabbitTray { icon, lifecycle };

    // block_in_place temporarily exits the runtime so Handle::block_on can
    // re-enter it safely — required because we're called from sync callbacks
    // inside Runtime::block_on where direct Handle::block_on would panic.
    let handle = tokio::task::block_in_place(move || {
        tokio::runtime::Handle::current().block_on(async {
            tray.assume_sni_available(true)
                .spawn()
                .await
                .map_err(|e| format!("ksni spawn failed: {e}"))
        })
    })?;

    if let Ok(mut guard) = TRAY_HANDLE.lock() {
        *guard = Some(handle);
    }
    SYSTRAY_ENABLED.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn remove_systray() {
    if let Ok(mut guard) = TRAY_HANDLE.lock() {
        if let Some(handle) = guard.take() {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(handle.shutdown());
            });
        }
    }
    SYSTRAY_ENABLED.store(false, Ordering::SeqCst);
}

pub fn update_systray(enabled: bool) {
    let currently_active = is_active();
    if enabled && !currently_active {
        let lifecycle = LIFECYCLE.lock().ok().and_then(|g| g.clone());
        match lifecycle {
            Some(lc) => {
                if let Err(e) = init_systray(lc) {
                    warn!("Failed to init system tray: {e}");
                }
            }
            None => warn!("Cannot init systray: no Lifecycle set"),
        }
    } else if !enabled && currently_active {
        remove_systray();
    }
}
