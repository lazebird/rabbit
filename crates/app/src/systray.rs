//! System tray integration using tray-icon crate

use fltk::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent,
};

// Track enabled state
static SYSTRAY_ENABLED: AtomicBool = AtomicBool::new(false);

// Store the TrayIcon as raw pointer (TrayIcon is not Send+Sync)
// Only accessed from main thread via awake_callback.
static mut TRAY_PTR: *mut TrayIcon = std::ptr::null_mut();

// Store main window reference for show/hide (cross-platform)
// Mutex used for static initialization (FLTK window is !Send, but we only access from main thread)
static MAIN_WIN: Mutex<Option<fltk::window::Window>> = Mutex::new(None);

/// Set the main window reference for systray operations
pub fn set_main_window(win: fltk::window::Window) {
    if let Ok(mut store) = MAIN_WIN.lock() {
        *store = Some(win);
    }
}

/// Check if systray is currently active
pub fn is_active() -> bool {
    SYSTRAY_ENABLED.load(Ordering::SeqCst)
}

/// Initialize the system tray.
/// Removes any existing icon first to avoid duplicates.
pub fn init_systray() -> Result<(), String> {
    // Remove old icon first (if any) to prevent duplicates
    remove_systray();

    let tray_menu = Menu::new();

    // Show, Hide, and Quit
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

    // Store as raw pointer
    unsafe {
        TRAY_PTR = Box::into_raw(Box::new(tray));
        SYSTRAY_ENABLED.store(true, Ordering::SeqCst);
    }

    // Double-click to show window
    TrayIconEvent::set_event_handler(Some(move |event| {
        if let TrayIconEvent::DoubleClick { .. } = event {
            show_main_window();
        }
    }));

    // Handle menu events - capture owned IDs
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

/// Remove the system tray icon immediately.
/// Must be called from the main thread (where TrayIcon was created).
pub fn remove_systray() {
    unsafe {
        if !TRAY_PTR.is_null() {
            // Reconstruct Box and drop it - this calls TrayIcon::drop()
            let _ = Box::from_raw(TRAY_PTR);
            TRAY_PTR = std::ptr::null_mut();
            SYSTRAY_ENABLED.store(false, Ordering::SeqCst);
        }
    }
}

/// Update systray visibility based on config.
/// Only acts if the enabled state actually changes.
pub fn update_systray(enabled: bool) {
    let currently_active = is_active();

    if enabled && !currently_active {
        let _ = init_systray();
    } else if !enabled && currently_active {
        remove_systray();
    }
    // If state unchanged, do nothing - prevents duplicate icons
}

/// Load icon from executable directory (same as main window)
fn load_icon() -> Result<tray_icon::Icon, String> {
    // Try next to executable first (most reliable for installed apps)
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop(); // Remove exe name
        exe_path.push("resources");
        exe_path.push("icon.ico");
        if let Ok(img) = image::open(&exe_path) {
            let img = img.into_rgba8();
            let (width, height) = img.dimensions();
            let rgba = img.into_raw();
            return tray_icon::Icon::from_rgba(rgba, width, height)
                .map_err(|e| format!("Failed to create icon: {}", e));
        }
    }

    // Fallback to relative paths (development)
    for path in &["crates/app/resources/icon.ico", "resources/icon.ico"] {
        if let Ok(img) = image::open(path) {
            let img = img.into_rgba8();
            let (width, height) = img.dimensions();
            let rgba = img.into_raw();
            return tray_icon::Icon::from_rgba(rgba, width, height)
                .map_err(|e| format!("Failed to create icon: {}", e));
        }
    }

    Err("Could not load icon from any location".to_string())
}

/// Show main window (called from tray event)
fn show_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            let win = win.clone();
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

/// Hide main window (called from tray event)
fn hide_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            let win = win.clone();
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
