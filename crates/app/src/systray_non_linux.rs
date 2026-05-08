use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tracing::warn;

use fltk::prelude::WidgetExt;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent,
};

static SYSTRAY_ENABLED: AtomicBool = AtomicBool::new(false);

static mut TRAY_PTR: *mut TrayIcon = std::ptr::null_mut();

static MAIN_WIN: Mutex<Option<fltk::window::Window>> = Mutex::new(None);

pub fn set_main_window(win: fltk::window::Window) {
    if let Ok(mut store) = MAIN_WIN.lock() {
        *store = Some(win);
    }
}

pub fn is_active() -> bool {
    SYSTRAY_ENABLED.load(Ordering::SeqCst)
}

pub fn init_systray() -> Result<(), String> {
    remove_systray();

    let tray_menu = Menu::new();

    let show_item = MenuItem::new("Show", true, None);
    let hide_item = MenuItem::new("Hide", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    tray_menu
        .append_items(&[&show_item, &hide_item, &quit_item])
        .map_err(|e| format!("Failed to add menu items: {}", e))?;

    let icon = load_icon()?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Rabbit - Network Tools")
        .with_icon(icon)
        .build()
        .map_err(|e| format!("Failed to create tray icon: {}", e))?;

    unsafe {
        TRAY_PTR = Box::into_raw(Box::new(tray));
        SYSTRAY_ENABLED.store(true, Ordering::SeqCst);
    }

    TrayIconEvent::set_event_handler(Some(move |event| {
        if let TrayIconEvent::DoubleClick { .. } = event {
            show_main_window();
        }
    }));

    let show_id = show_item.id().to_owned();
    let hide_id = hide_item.id().to_owned();
    let quit_id = quit_item.id().to_owned();

    tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
        if event.id == quit_id {
            fltk::app::quit();
        } else if event.id == show_id {
            show_main_window();
        } else if event.id == hide_id {
            hide_main_window();
        }
    }));

    Ok(())
}

pub fn remove_systray() {
    unsafe {
        if !TRAY_PTR.is_null() {
            let _ = Box::from_raw(TRAY_PTR);
            TRAY_PTR = std::ptr::null_mut();
            SYSTRAY_ENABLED.store(false, Ordering::SeqCst);
        }
    }
}

pub fn update_systray(enabled: bool) {
    let currently_active = is_active();

    if enabled && !currently_active {
        if let Err(e) = init_systray() {
            warn!("Failed to init system tray: {}", e);
        }
    } else if !enabled && currently_active {
        remove_systray();
    }
}

fn load_icon_from(path: &std::path::Path) -> Result<tray_icon::Icon, String> {
    let img = image::open(path).map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
    let img = img.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    tray_icon::Icon::from_rgba(rgba, width, height)
        .map_err(|e| format!("Failed to create icon from {}: {}", path.display(), e))
}

fn make_fallback_icon() -> tray_icon::Icon {
    let w: u32 = 32;
    let h: u32 = 32;
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
    tray_icon::Icon::from_rgba(rgba, w, h).unwrap_or_else(|_| {
        tray_icon::Icon::from_rgba(vec![0, 0, 0, 0], 1, 1).unwrap()
    })
}

fn load_icon() -> Result<tray_icon::Icon, String> {
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        let ico_path = exe_path.join("resources").join("icon.ico");
        if ico_path.exists() {
            return load_icon_from(&ico_path);
        }
    }
    for path_str in &["crates/app/resources/icon.ico", "resources/icon.ico"] {
        let p = std::path::Path::new(path_str);
        if p.exists() {
            return load_icon_from(p);
        }
    }
    warn!("Icon file not found, using embedded fallback icon");
    Ok(make_fallback_icon())
}

fn show_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            let mut win = win.clone();
            fltk::app::awake_callback(move || {
                #[cfg(target_os = "windows")]
                {
                    let hwnd = win.raw_handle() as usize;
                    adapter::window::show_window(hwnd);
                }
                #[cfg(not(target_os = "windows"))]
                {
                    win.show();
                }
            });
        }
    }
}

fn hide_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            let mut win = win.clone();
            fltk::app::awake_callback(move || {
                #[cfg(target_os = "windows")]
                {
                    let hwnd = win.raw_handle() as usize;
                    adapter::window::hide_window(hwnd);
                }
                #[cfg(not(target_os = "windows"))]
                {
                    win.hide();
                }
            });
        }
    }
}
