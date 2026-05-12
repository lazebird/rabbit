//! Non-Linux system tray using `tray-icon` crate (native APIs on Windows/macOS).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tracing::warn;

use crate::Lifecycle;
#[cfg(target_os = "windows")]
use fltk::prelude::WindowExt;
#[cfg(not(target_os = "windows"))]
use fltk::prelude::WidgetExt;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent,
};

// Fixed menu item IDs so the OnceCell-based event handler matches
// across tray re-initialization (see register_event_handlers_once).
const MENU_ID_SHOW: &str = "show";
const MENU_ID_HIDE: &str = "hide";
const MENU_ID_QUIT: &str = "quit";

static SYSTRAY_ENABLED: AtomicBool = AtomicBool::new(false);

static TRAY_PTR: Mutex<Option<Box<TrayIcon>>> = Mutex::new(None);

static MAIN_WIN: Mutex<Option<fltk::window::Window>> = Mutex::new(None);
static LIFECYCLE: Mutex<Option<Lifecycle>> = Mutex::new(None);

pub fn set_main_window(win: fltk::window::Window) {
    if let Ok(mut store) = MAIN_WIN.lock() {
        *store = Some(win);
    }
}

/// Store the Lifecycle handle for later use by systray init/update.
pub fn set_lifecycle(lifecycle: &Lifecycle) {
    if let Ok(mut guard) = LIFECYCLE.lock() {
        if guard.is_none() {
            *guard = Some(lifecycle.clone());
        }
    }
}

pub fn is_active() -> bool {
    SYSTRAY_ENABLED.load(Ordering::SeqCst)
}

pub fn init_systray(lifecycle: Lifecycle) -> Result<(), String> {
    remove_systray();

    if let Ok(mut guard) = LIFECYCLE.lock() {
        *guard = Some(lifecycle);
    }

    let tray_menu = Menu::new();

    let show_item = MenuItem::with_id(MENU_ID_SHOW, "Show", true, None);
    let hide_item = MenuItem::with_id(MENU_ID_HIDE, "Hide", true, None);
    let quit_item = MenuItem::with_id(MENU_ID_QUIT, "Quit", true, None);

    tray_menu
        .append_items(&[&show_item, &hide_item, &quit_item])
        .map_err(|e| format!("Failed to add menu items: {}", e))?;

    let data = crate::icon::load_app_icon();
    let icon = tray_icon::Icon::from_rgba(data.rgba, data.width, data.height)
        .expect("embedded icon RGBA is valid");

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Rabbit - Network Tools")
        .with_icon(icon)
        .build()
        .map_err(|e| format!("Failed to create tray icon: {}", e))?;

    if let Ok(mut guard) = TRAY_PTR.lock() {
        *guard = Some(Box::new(tray));
    }
    SYSTRAY_ENABLED.store(true, Ordering::SeqCst);

    register_event_handlers_once();

    Ok(())
}

/// Register global tray and menu event handlers.
///
/// SAFETY: Must be safe to call multiple times — both `TrayIconEvent::set_event_handler`
/// and `MenuEvent::set_event_handler` use `OnceCell` internally and silently discard
/// all calls after the first. The handlers use fixed string IDs (see `MENU_ID_*` constants)
/// and read `Lifecycle` from the `LIFECYCLE` static on each invocation, so they remain
/// correct across tray re-initializations.
fn register_event_handlers_once() {
    use std::sync::Once;
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        TrayIconEvent::set_event_handler(Some(move |event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                show_main_window();
            }
        }));

        tray_icon::menu::MenuEvent::set_event_handler(Some(
            move |event: tray_icon::menu::MenuEvent| {
                let lc = LIFECYCLE.lock().ok().and_then(|g| g.clone());
                if event.id == MENU_ID_QUIT {
                    if let Some(ref lifecycle) = lc {
                        lifecycle.request_shutdown();
                    }
                    fltk::app::awake_callback(|| {
                        fltk::app::quit();
                    });
                } else if event.id == MENU_ID_SHOW {
                    show_main_window();
                } else if event.id == MENU_ID_HIDE {
                    hide_main_window();
                }
            },
        ));
    });
}

pub fn remove_systray() {
    if let Ok(mut guard) = TRAY_PTR.lock() {
        if guard.take().is_some() {
            SYSTRAY_ENABLED.store(false, Ordering::SeqCst);
        }
    }
}

pub fn update_systray(enabled: bool) {
    let currently_active = is_active();

    if enabled && !currently_active {
        let lifecycle = LIFECYCLE.lock().ok().and_then(|g| g.clone());
        match lifecycle {
            Some(lc) => {
                if let Err(e) = init_systray(lc) {
                    warn!("Failed to init system tray: {}", e);
                }
            }
            None => warn!("Cannot init systray: no Lifecycle set"),
        }
    } else if !enabled && currently_active {
        remove_systray();
    }
}

fn show_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            let win = win.clone();
            fltk::app::awake_callback(move || {
                #[cfg(target_os = "windows")]
                {
                    crate::window::show_window();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let mut win = win;
                    win.show();
                }
            });
        }
    }
}

fn hide_main_window() {
    if let Ok(store) = MAIN_WIN.lock() {
        if let Some(win) = &*store {
            let win = win.clone();
            fltk::app::awake_callback(move || {
                #[cfg(target_os = "windows")]
                {
                    crate::window::hide_window();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let mut win = win;
                    win.hide();
                }
            });
        }
    }
}
