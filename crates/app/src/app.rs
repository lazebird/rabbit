//! Application Controller - FLTK UI Implementation


use crate::ui::{ChatTab, HttpTab, PingTab, PlanTab, ScanTab, SettingsTab, TabComponent, TftpcTab, TftpdTab};
use crate::ui_events::{init_event_system, send_event, EventHandler, UiEvent};
use crate::ui_state::UiState;
use crate::upgrade;
use crate::view_model::AppViewModel;
use adapter::tray;
use fltk::{
    app,
    group::Tabs,
    image::IcoImage,
    prelude::*,
    window::{Window, WindowType},
};
use service::{
    ui_channel::{Module, UiData},
    ChatService, HttpService, PingService, PlanService, ScanService, ServiceUpdateResult, TftpcService, TftpdService,
};
use schema::{config::{self, ConfigValue}, AppConfig};

use rabbit_config::{load_config, save_config};
use adapter::Lifecycle;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Window position save with debounce (event-driven, no polling)
// ---------------------------------------------------------------------------
thread_local! {
    static RESIZE_TIMEOUT: std::cell::Cell<Option<app::TimeoutHandle>> = const { std::cell::Cell::new(None) };
}

fn save_window_position() {
    RESIZE_TIMEOUT.with(|cell| {
        cell.set(None);
    });

    info!("Attempting to save window position...");
    if let Some(win) = crate::ui_state::UiState::get_main_window() {
        let win_x = win.x();
        let win_y = win.y();
        let win_w = win.w();
        let win_h = win.h();
        info!("Window position: x={}, y={}, w={}, h={}", win_x, win_y, win_w, win_h);

        if let Ok(mut config) = rabbit_config::load_config() {
            let window_config = schema::config::WindowConfig {
                x: win_x,
                y: win_y,
                width: win_w,
                height: win_h,
            };
            let json = serde_json::to_string(&window_config).unwrap_or_default();
            config.modules.insert("global", config::keys::global::WINDOW, ConfigValue::String(json));
            let _ = rabbit_config::save_config(&config);
            info!("Window position saved to config");
        }
    }
}


/// Main application struct
pub struct App {
    view_model: Arc<RwLock<AppViewModel>>,
    ping_service: Arc<RwLock<PingService>>,
    http_service: Arc<RwLock<HttpService>>,
    tftp_server_service: Arc<RwLock<TftpdService>>,
    tftp_client_service: Arc<RwLock<TftpcService>>,
    plan_service: Arc<RwLock<PlanService>>,
    chat_service: Arc<RwLock<ChatService>>,
    scan_service: Arc<RwLock<ScanService>>,
    lifecycle: Lifecycle,
    http_rx: Option<mpsc::Receiver<UiData>>,
    ping_rx: Option<mpsc::Receiver<UiData>>,
    scan_rx: Option<mpsc::Receiver<UiData>>,
    tftpd_rx: Option<mpsc::Receiver<UiData>>,
    tftpc_rx: Option<mpsc::Receiver<UiData>>,
    chat_rx: Option<mpsc::Receiver<UiData>>,
    plan_rx: Option<mpsc::Receiver<UiData>>,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> anyhow::Result<Self> {
        rabbit_diag::log("App::new(): entered");
        info!("Initializing Rabbit application");

        // Load configuration - use default if load fails (e.g., first run)
        rabbit_diag::log("App::new(): loading config");
        let config = match load_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                info!("Failed to load config ({}), using defaults", e);
                AppConfig::default()
            }
        };

        // Create UI channels
        rabbit_diag::log("App::new(): creating channels");
        let (http_tx, http_rx) = mpsc::channel(100);
        let (ping_tx, ping_rx) = mpsc::channel(100);
        let (scan_tx, scan_rx) = mpsc::channel(100);
        let (tftpd_tx, tftpd_rx) = mpsc::channel(100);
        let (tftpc_tx, tftpc_rx) = mpsc::channel(100);
        let (chat_tx, chat_rx) = mpsc::channel(100);
        let (plan_tx, plan_rx) = mpsc::channel(100);

        // Create services with channels
        rabbit_diag::log("App::new(): creating services");
        let ping_service = PingService::with_channel(ping_tx);
        let http_service = HttpService::with_channel(http_tx);
        let tftp_server_service = TftpdService::with_channel(tftpd_tx);
        let tftp_client_service = TftpcService::with_channel(tftpc_tx);
        let plan_service = PlanService::with_channel(plan_tx);
        let chat_service = ChatService::with_channel(chat_tx);
        let scan_service = ScanService::with_channel(scan_tx)
            .with_network(std::sync::Arc::new(adapter::DefaultNetworkProvider));

        // Create view model
        rabbit_diag::log("App::new(): creating view model");
        let view_model = AppViewModel::new(config);

        rabbit_diag::log("App::new(): returning");
        Ok(Self {
            view_model: Arc::new(RwLock::new(view_model)),
            ping_service: Arc::new(RwLock::new(ping_service)),
            http_service: Arc::new(RwLock::new(http_service)),
            tftp_server_service: Arc::new(RwLock::new(tftp_server_service)),
            tftp_client_service: Arc::new(RwLock::new(tftp_client_service)),
            plan_service: Arc::new(RwLock::new(plan_service)),
            chat_service: Arc::new(RwLock::new(chat_service)),
            scan_service: Arc::new(RwLock::new(scan_service)),
            lifecycle: Lifecycle::new(),
            http_rx: Some(http_rx),
            ping_rx: Some(ping_rx),
            scan_rx: Some(scan_rx),
            tftpd_rx: Some(tftpd_rx),
            tftpc_rx: Some(tftpc_rx),
            chat_rx: Some(chat_rx),
            plan_rx: Some(plan_rx),
        })
    }

    /// Run the application
    pub async fn run(&mut self) -> anyhow::Result<()> {
        rabbit_diag::log("App::run(): entered");
        info!("Running Rabbit application with FLTK UI");

        // Initialize UI state and event system
        let _ui_state = UiState::init();

        // Restore HTTP and TFTP directories from config
        {
            let config = self.view_model.read().await.get_config();
            // Restore HTTP items
            if let Some(dirs) = config.modules.get_array("http", config::keys::http::DIRS) {
                for dir in dirs {
                    crate::ui_state::add_http_item(&dir);
                }
            }
            // Restore TFTP directories
            if let Some(dirs) = config.modules.get_array("tftpd", config::keys::tftpd::WORK_DIRS) {
                for dir in dirs {
                    crate::ui_state::add_tftpd_dir(&dir);
                }
            }
            // Restore TFTP working directory selection
            if let Some(idx) = config.modules.get_integer("tftpd", config::keys::tftpd::WORKING_DIR_INDEX) {
                if let Some(dirs) = config.modules.get_array("tftpd", config::keys::tftpd::WORK_DIRS) {
                    if idx < dirs.len() as i64 {
                        crate::ui_state::set_tftpd_selected((idx + 1) as i32);
                    }
                }
            }
        }

        let event_receiver = init_event_system();

        // Spawn UI channel receivers
        if let Some(mut rx) = self.ping_rx.take() {
            let view_model = Arc::clone(&self.view_model);
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    handle_ui_data(data, &view_model).await;
                }
            });
        }

        if let Some(mut rx) = self.scan_rx.take() {
            let view_model = Arc::clone(&self.view_model);
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    handle_ui_data(data, &view_model).await;
                }
            });
        }

        if let Some(mut rx) = self.chat_rx.take() {
            let view_model = Arc::clone(&self.view_model);
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    handle_ui_data(data, &view_model).await;
                }
            });
        }

        if let Some(mut rx) = self.plan_rx.take() {
            let view_model = Arc::clone(&self.view_model);
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    handle_ui_data(data, &view_model).await;
                }
            });
        }

        // Load saved tasks into PlanService and start service
        {
            let saved_tasks = crate::ui_state::load_plan_tasks();
            let mut plan_service = self.plan_service.write().await;
            for task in &saved_tasks {
                let _ = plan_service.add_task(
                    &task.date, &task.time, task.cycle, &task.unit, &task.msg, true
                ).await;
            }
            // Start the plan service (starts timer for all loaded tasks)
            let _ = plan_service.start().await;
        }

        if let Some(mut rx) = self.http_rx.take() {
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    handle_http_data(data).await;
                }
            });
        }

        if let Some(mut rx) = self.tftpd_rx.take() {
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    handle_tftp_data(data).await;
                }
            });
        }

        if let Some(mut rx) = self.tftpc_rx.take() {
            tokio::spawn(async move {
                while let Some(data) = rx.recv().await {
                    handle_tftp_data(data).await;
                }
            });
        }

        // Create FLTK application
        let _fltk_app = app::App::default();

        // Install platform-specific error handlers (X11 diag on Linux).
        // Must be done AFTER app::App::default() (initialises X11 display),
        // but BEFORE any window operations.
        adapter::install_platform_handlers();

        // Set application-wide colors (lighter theme - similar to old version)
        app::background(0xF0, 0xF0, 0xF0); // Light gray background
        app::background2(0xFF, 0xFF, 0xFF); // White for inputs
        app::foreground(0x00, 0x00, 0x00); // Black text
        app::set_visible_focus(true);

        // Create main window - load config directly from disk to get window position
        let disk_config = load_config().unwrap_or_else(|_| AppConfig::default());
        let modules = &disk_config.modules;
        let (win_x, win_y, win_w, win_h) = modules
            .get_string("global", config::keys::global::WINDOW)
            .and_then(|s| serde_json::from_str::<schema::config::WindowConfig>(&s).ok())
            .map(|w| (w.x, w.y, w.width, w.height))
            .unwrap_or((100, 100, 748, 518));
        info!("Loaded config: window pos=({}, {}), size=({}x{})", win_x, win_y, win_w, win_h);

        let mut main_win = Window::new(win_x, win_y, win_w, win_h, "Rabbit");
        main_win.set_type(WindowType::Double);
        main_win.make_resizable(true);

        info!("Creating window at ({}, {}) size {}x{}", win_x, win_y, win_w, win_h);

        // Set window icon from embedded bytes (single source: adapter)
        if let Ok(icon) = IcoImage::from_data(adapter::icon::ico_bytes()) {
            main_win.set_icon(Some(icon));
            info!("Loaded window icon from embedded bytes");
        } else {
            warn!("Failed to load window icon from embedded bytes");
        }

        // Create Tabs widget - positioned to leave room for tab labels
        let mut tabs = Tabs::new(5, 5, win_w - 10, win_h - 10, "");

        // Build each tab - y=25 leaves room for tab labels at top
        let ping_tab = PingTab::build(5, 30, win_w - 18, win_h - 43, &disk_config);
        let scan_tab = ScanTab::build(5, 30, win_w - 18, win_h - 43, &disk_config);
        let http_tab = HttpTab::build(5, 30, win_w - 18, win_h - 43, &disk_config);
        let tftpd_tab = TftpdTab::build(5, 30, win_w - 18, win_h - 43, &disk_config);
        let tftpc_tab = TftpcTab::build(5, 30, win_w - 18, win_h - 43, &disk_config);
        let plan_tab = PlanTab::build(5, 30, win_w - 18, win_h - 43, &disk_config);
        let chat_tab = ChatTab::build(5, 30, win_w - 18, win_h - 43, &disk_config);
        let settings_tab = SettingsTab::build(5, 30, win_w - 18, win_h - 43, &disk_config);

        tabs.end();

        // Restore last active tab from config
        let config = self.view_model.read().await.get_config();
        let modules = &config.modules;
        let last_tab = modules.get_integer("global", config::keys::global::LAST_ACTIVE_TAB).unwrap_or(0) as usize;
        let autoupdate = modules.get_bool("global", config::keys::global::AUTOUPDATE).unwrap_or(true);
        let top_requested = modules.get_bool("global", config::keys::global::TOP).unwrap_or(false);
        let systray_requested = modules.get_bool("global", config::keys::global::SYSTRAY).unwrap_or(true);
        let autostart_requested = modules.get_bool("global", config::keys::global::AUTOSTART).unwrap_or(false);
        let http_shell_requested = modules.get_bool("http", config::keys::http::SHELL).unwrap_or(false);
        drop(config);
        let tab_ptrs: Vec<usize> = vec![
            ping_tab.as_widget_ptr() as usize,
            scan_tab.as_widget_ptr() as usize,
            http_tab.as_widget_ptr() as usize,
            tftpd_tab.as_widget_ptr() as usize,
            tftpc_tab.as_widget_ptr() as usize,
            plan_tab.as_widget_ptr() as usize,
            chat_tab.as_widget_ptr() as usize,
            settings_tab.as_widget_ptr() as usize,
        ];
        let tab_groups: [&fltk::group::Flex; 8] = [&ping_tab, &scan_tab, &http_tab, &tftpd_tab, &tftpc_tab, &plan_tab, &chat_tab, &settings_tab];
        if last_tab < tab_groups.len() {
            tabs.set_value(tab_groups[last_tab]).ok();
        }

        // Track tab changes and save to config
        let view_model_for_tab = self.view_model.clone();
        let mut tabs_for_set_cb = tabs.clone();
        let tabs_for_closure = tabs.clone();
        let tab_ptrs_clone = tab_ptrs.clone();
        tabs_for_set_cb.set_callback(move |_| {
            if let Some(current) = tabs_for_closure.value() {
                let ptr = current.as_widget_ptr() as usize;
                let idx = tab_ptrs_clone.iter().position(|&p| p == ptr).unwrap_or(0);
                if view_model_for_tab.try_write().is_ok() {
                    rabbit_config::update_config(|cfg| {
                        cfg.modules.insert("global", config::keys::global::LAST_ACTIVE_TAB, schema::config::ConfigValue::Integer(idx as i64));
                    })
                    .ok();
                }
            }
        });

        // UI refresh now uses event-driven callbacks (no polling loop needed)

        // Add global keyboard event handling (after tabs are created)
        let mut main_win_for_keys = main_win.clone();
        let tabs_for_keys = tabs.clone();
        let tab_ptrs_for_keys: Vec<usize> = vec![
            ping_tab.as_widget_ptr() as usize,
            scan_tab.as_widget_ptr() as usize,
            http_tab.as_widget_ptr() as usize,
            tftpd_tab.as_widget_ptr() as usize,
            tftpc_tab.as_widget_ptr() as usize,
            plan_tab.as_widget_ptr() as usize,
            chat_tab.as_widget_ptr() as usize,
            settings_tab.as_widget_ptr() as usize,
        ];
        let esc_lc = self.lifecycle.clone();
        main_win_for_keys.handle(move |win, ev| {
            use fltk::enums::Event;
            use fltk::enums::Key;
            if ev == Event::KeyDown {
                let key = app::event_key();
                match key {
                    // Esc: Exit the program
                    Key::Escape => {
                        info!("Esc key pressed - exiting program");
                        esc_lc.request_shutdown();
                        win.hide();
                        app::quit();
                        true
                    }
                    // Enter: Execute current tab's main button
                    Key::Enter => {
                        info!("Enter key pressed - triggering current tab action");
                        // Get current tab index
                        if let Some(current) = tabs_for_keys.value() {
                            let ptr = current.as_widget_ptr() as usize;
                            let idx = tab_ptrs_for_keys.iter().position(|&p| p == ptr).unwrap_or(0);
                            // Trigger the button for this tab
                            crate::ui::ui_refresh::trigger_tab_button(idx);
                        }
                        true
                    }
                    // F1: Open help documentation
                    Key::F1 => {
                        info!("F1 key pressed - opening help");
                        let _ = adapter::dialog::open_url("https://github.com/lazebird/rabbit/blob/rewrite/doc/manual.md");
                        true
                    }
                    // F2: Open project homepage
                    Key::F2 => {
                        info!("F2 key pressed - opening project homepage");
                        let _ = adapter::dialog::open_url("https://github.com/lazebird/rabbit");
                        true
                    }
                    // F3: Open config file directory
                    Key::F3 => {
                        info!("F3 key pressed - opening config directory");
                        if let Some(config_path) = dirs::config_local_dir() {
                            let rabbit_config = config_path.join("Rabbit");
                            let path_str = rabbit_config.to_string_lossy().to_string();
                            let _ = adapter::dialog::open_file_manager(&path_str);
                        }
                        true
                    }
                    _ => false,
                }
            } else {
                false
            }
        });

        // Let tabs fill the window on resize and save window position with debouncing
        let mut tabs_clone = tabs.clone();

        main_win.resize_callback(move |w, x, y, nw, nh| {
            tabs_clone.resize(5, 5, nw - 10, nh - 10);
            w.redraw();

            info!("resize callback: pos=({},{}) size={}x{}", x, y, nw, nh);

            RESIZE_TIMEOUT.with(|cell| {
                if let Some(handle) = cell.take() {
                    app::remove_timeout3(handle);
                }
                let new_handle = app::add_timeout3(0.5, |_handle| save_window_position());
                cell.set(Some(new_handle));
            });
        });

        // Apply autostart setting
        if let Err(e) = adapter::autostart::set_autostart(autostart_requested) {
            warn!("Failed to set autostart: {}", e);
        }

        // Apply HTTP shell integration at startup
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_path_str = exe_path.to_string_lossy().to_string();
            if let Err(e) = adapter::shell::set_shell_integration(http_shell_requested, &exe_path_str) {
                warn!("Failed to set HTTP shell integration at startup: {}", e);
            } else {
                info!("HTTP shell integration applied at startup: {}", http_shell_requested);
            }
        }

        // Restore business running states from config
        rabbit_diag::log("app.run(): checking restore states");
        let config_for_restore = self.view_model.read().await.get_config();
        let ping_restore = config_for_restore.modules.get_bool("ping", config::keys::ping::RUNNING).unwrap_or(false);
        let http_restore = config_for_restore.modules.get_bool("http", config::keys::http::RUNNING).unwrap_or(false);
        let tftp_restore = config_for_restore.modules.get_bool("tftpd", config::keys::tftpd::RUNNING).unwrap_or(false);
        let chat_restore = config_for_restore.modules.get_bool("chat", config::keys::chat::RUNNING).unwrap_or(false);
        drop(config_for_restore);
        rabbit_diag::log(&format!("app.run(): restore states ping={ping_restore} http={http_restore} tftp={tftp_restore} chat={chat_restore}"));

        // Handle Ctrl+C
        let ctrlc_lc = self.lifecycle.clone();
        ctrlc::set_handler(move || {
            info!("Ctrl+C received, initiating shutdown...");
            ctrlc_lc.request_shutdown();
            fltk::app::awake_callback(|| {
                info!("Executing quit() on UI thread...");
                fltk::app::quit();
            });
        })
        .ok();

        // Spawn event handler task
        rabbit_diag::log("app.run(): spawning event handler task");
        let app_clone = Arc::new(RwLock::new(AppHandle {
            view_model: self.view_model.clone(),
            ping_service: self.ping_service.clone(),
            http_service: self.http_service.clone(),
            tftp_server_service: self.tftp_server_service.clone(),
            tftp_client_service: self.tftp_client_service.clone(),
            plan_service: self.plan_service.clone(),
            chat_service: self.chat_service.clone(),
            scan_service: self.scan_service.clone(),
        }));

        let event_handle = tokio::spawn(async move {
            Self::event_loop(app_clone, event_receiver).await;
        });
        rabbit_diag::log("app.run(): event handler spawned");

        // Restore business states after event loop is ready
        rabbit_diag::log("app.run(): sending restore events");
        if ping_restore { send_event(UiEvent::ModuleToggle { module: "ping".into() }); }
        if http_restore { send_event(UiEvent::ModuleToggle { module: "http".into() }); }
        if tftp_restore { send_event(UiEvent::ModuleToggle { module: "tftpd".into() }); }
        if chat_restore { send_event(UiEvent::ModuleToggle { module: "chat".into() }); }
        rabbit_diag::log("app.run(): restore events sent");

        // FINAL WINDOW PREPARATION AND SHOW
        main_win.end();
        
        // Store main window for title updates
        crate::ui_state::UiState::set_main_window(main_win.clone());

        // Show window first (needed before getting raw_handle on some platforms)
        rabbit_diag::log(&format!("app.run(): before main_win.show() pos=({},{}) size=({}x{})",
            win_x, win_y, win_w, win_h));
        main_win.show();
        rabbit_diag::log(&format!("app.run(): after main_win.show() pos=({},{})",
            main_win.x(), main_win.y()));

        // After show(), we can get the valid OS handle (HWND on Windows)
        let hwnd = main_win.raw_handle() as usize;

        // Store main window reference for systray show/hide operations (cross-platform)
        tray::set_main_window(main_win.clone());

        // Store hwnd for taskbar progress (Windows only)
        adapter::set_main_window(hwnd);

        // Apply window topmost setting AFTER show (safer on Windows)
        if top_requested {
            adapter::set_window_on_top(true);
        }

        // Handle window close button - use set_callback which fires when the X button is clicked
        // Always quit when X is clicked (user requirement: only tray Hide should hide)
        let cb_lc = self.lifecycle.clone();
        main_win.set_callback(move |_| {
            info!("Close button clicked - exiting program");
            cb_lc.request_shutdown();
            fltk::app::quit();
        });

        // ── systray initialization ──
        //
        // Populate LIFECYCLE (needed by both helper IPC and local ksni).
        // Then initialise the tray via the unified adapter::systray::init()
        // which handles Linux tray-helper IPC and local ksni fallback.
        adapter::tray::set_lifecycle(&self.lifecycle);
        adapter::systray::init(&self.lifecycle, systray_requested, main_win.clone()).await;

        // Startup version check if autoupdate is enabled
        if autoupdate {
            info!("Auto-update enabled, checking for updates...");
            let lc_for_check = self.lifecycle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                if !lc_for_check.is_shutdown_requested() {
                    upgrade::handle_version_check_result(Some(&lc_for_check));
                }
            });
        }

        // Custom event loop — stays alive even when all windows are hidden,
        // which is required for systray "Hide" (app runs in background).
        // Uses Lifecycle::run_event_loop() which wraps fltk::app::wait_for(0.05)
        // and checks the unified shutdown flag.
        // Exit triggers: Ctrl+C / ESC / close button / tray Quit — all set the
        // same Lifecycle::shutdown_requested flag.
        rabbit_diag::log("app.run(): entering FLTK event loop");
        self.lifecycle.run_event_loop();
        rabbit_diag::log("app.run(): FLTK event loop exited");

        // Abort the event loop task (it's blocked on receiver.recv() which will never return)
        rabbit_diag::log("app.run(): aborting event handler");
        event_handle.abort();
        rabbit_diag::log("app.run(): event handler aborted");

        // Cleanup with timeout to prevent hanging on exit
        rabbit_diag::log("app.run(): starting cleanup");
        adapter::systray::cleanup();
        rabbit_diag::log("app.run(): calling service cleanup");
        let cleanup_future = self.cleanup();
        match tokio::time::timeout(std::time::Duration::from_secs(3), cleanup_future).await {
            Ok(result) => {
                if let Err(e) = result {
                    rabbit_diag::log(&format!("app.run(): cleanup FAILED: {e}"));
                    error!("Cleanup failed: {}", e);
                } else {
                    rabbit_diag::log("app.run(): cleanup OK");
                    info!("Cleanup completed successfully");
                }
            }
            Err(_) => {
                rabbit_diag::log("app.run(): cleanup timed out, forcing exit");
                warn!("Cleanup timed out after 3 seconds, forcing exit");
            }
        }

        // Force exit - FLTK may leave internal threads running
        rabbit_diag::log("app.run(): calling std::process::exit(0)");
        info!("Final exit via std::process::exit(0)");
        std::process::exit(0);
    }

    /// Event loop for processing UI events.
    ///
    /// Uses `.recv().await` (a proper async yield point) so the tokio worker
    /// thread is never blocked — co-located tasks (e.g. the ksni system-tray
    /// service) can always be polled between events.
    async fn event_loop(app: Arc<RwLock<AppHandle>>, mut receiver: tokio::sync::mpsc::UnboundedReceiver<UiEvent>) {
        while let Some(event) = receiver.recv().await {
            info!("Processing UI event: {:?}", event);
            let mut handle = app.write().await;
            if let Err(e) = handle.handle_event(event).await {
                error!("Failed to handle event: {}", e);
            }
        }
    }

    /// Cleanup resources — destroys services without sending status updates.
    async fn cleanup(&self) -> anyhow::Result<()> {
        rabbit_diag::log("app.cleanup(): started");
        macro_rules! destroy_svc {
            ($field:expr, $label:expr) => {{
                $field.write().await.destroy().await.ok();
                rabbit_diag::log(concat!("app.cleanup(): ", $label, " destroyed"));
            }};
        }
        destroy_svc!(self.ping_service, "ping");
        destroy_svc!(self.http_service, "http");
        destroy_svc!(self.tftp_server_service, "tftpd");
        destroy_svc!(self.plan_service, "plan");
        destroy_svc!(self.chat_service, "chat");
        info!("All services destroyed");
        Ok(())
    }
}



/// AppHandle for event handling (separate from App to avoid borrow issues)
struct AppHandle {
    view_model: Arc<RwLock<AppViewModel>>,
    ping_service: Arc<RwLock<PingService>>,
    http_service: Arc<RwLock<HttpService>>,
    tftp_server_service: Arc<RwLock<TftpdService>>,
    tftp_client_service: Arc<RwLock<TftpcService>>,
    plan_service: Arc<RwLock<PlanService>>,
    chat_service: Arc<RwLock<ChatService>>,
    scan_service: Arc<RwLock<ScanService>>,
}

#[async_trait::async_trait]
impl EventHandler for AppHandle {
    async fn handle_event(&mut self, event: UiEvent) -> anyhow::Result<()> {
        match event {
            // Module Toggle - 根据配置决定启动/停止
            UiEvent::ModuleToggle { module } => {
                match module.as_str() {
                    "ping" => {
                        let result = self.ping_service.write().await.update().await;
                        match result {
                            ServiceUpdateResult::Started(_) => {
                                crate::ui_state::update_module_running("ping", true);
                            }
                            ServiceUpdateResult::Stopped(_) => {
                                crate::ui_state::update_module_running("ping", false);
                            }
                            _ => {}
                        }
                    }
                    "scan" => {
                        let result = self.scan_service.write().await.update().await;
                        match result {
                            ServiceUpdateResult::Started(_) => {
                                crate::ui_state::update_module_running("scan", true);
                            }
                            ServiceUpdateResult::Stopped(_) => {
                                crate::ui_state::update_module_running("scan", false);
                            }
                            _ => {}
                        }
                    }
                    "http" => {
                        let result = self.http_service.write().await.update().await;
                        match result {
                            ServiceUpdateResult::Started(_) => {
                                crate::ui_state::update_module_running("http", true);
                                fltk::app::awake_callback(|| {
                                    crate::ui::ui_refresh::update_button_state(Module::Http, true);
                                });
                            }
                            ServiceUpdateResult::Stopped(_) => {
                                crate::ui_state::update_module_running("http", false);
                                fltk::app::awake_callback(|| {
                                    crate::ui::ui_refresh::update_button_state(Module::Http, false);
                                });
                            }
                            _ => {}
                        }
                    }
                    "tftpd" => {
                        let result = self.tftp_server_service.write().await.update().await;
                        match result {
                            ServiceUpdateResult::Started(_) => {
                                crate::ui_state::update_module_running("tftpd", true);
                            }
                            ServiceUpdateResult::Stopped(_) => {
                                crate::ui_state::update_module_running("tftpd", false);
                                crate::ui_state::write_to("tftpd_log", &crate::ui_state::fmt_log("TFTP server stopped"));
                            }
                            _ => {}
                        }
                    }
                    "chat" => {
                        let result = self.chat_service.write().await.update().await;
                        match result {
                            ServiceUpdateResult::Started(_) => {
                                // chat running state tracked by service
                            }
                            ServiceUpdateResult::Stopped(_) => {
                                crate::ui_state::set_chat_users(&[]);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            UiEvent::TftpClientPut { server, local, remote, options } => {
                info!("TFTP put {} -> {}@{} with options: {}", local, remote, server, options);

                let _opts = parse_tftp_options(&options);

                let tftp_client_service = self.tftp_client_service.write().await;
                match tftp_client_service.put(&local, &remote).await {
                    Ok(transfer_id) => {
                        info!("TFTP upload started with transfer ID: {}", transfer_id);
                    }
                    Err(e) => {
                        error!("TFTP upload failed: {}", e);
                    }
                }
            }
            UiEvent::TftpClientGet { server, local, remote, options } => {
                info!("TFTP get {}@{} -> {} with options: {}", remote, server, local, options);

                let _opts = parse_tftp_options(&options);

                let tftp_client_service = self.tftp_client_service.write().await;
                match tftp_client_service.get(&remote, &local).await {
                    Ok(transfer_id) => {
                        info!("TFTP download started with transfer ID: {}", transfer_id);
                    }
                    Err(e) => {
                        error!("TFTP download failed: {}", e);
                    }
                }
            }

            // Plan
            UiEvent::PlanAdd { date, time, cycle, unit, msg } => {
                info!("Adding plan for {} {}: {} (cycle={}, unit={})", date, time, msg, cycle, unit);
                let override_conflict = rabbit_config::get_bool("plan", config::keys::plan::OVERRIDE).unwrap_or(false);
                match self.plan_service.write().await.add_task(&date, &time, cycle, &unit, &msg, override_conflict).await {
                    Ok(true) => info!("Plan '{}' overridden", msg),
                    Ok(false) => info!("Plan '{}' added", msg),
                    Err(e) => {
                        let err_msg = format!("{}", e);
                        warn!("Plan add error: {}", err_msg);
                        fltk::app::awake_callback(move || {
                            fltk::dialog::alert_default(&err_msg);
                        });
                    }
                }
            }

            UiEvent::PlanRemove { msg } => {
                info!("Removing plan '{}'", msg);
                self.plan_service.write().await.remove_task(&msg).await?;
            }

            UiEvent::ChatSend { message } => {
                info!("Sending chat message: {}", message);
                self.chat_service.write().await.send_text(&message).await?;
            }
            UiEvent::ChatRefresh => {
                info!("Refreshing chat users");
                let chat_service = self.chat_service.write().await;
                match chat_service.refresh_users().await {
                    Ok(_) => {
                        info!("User list refreshed successfully");
                    }
                    Err(e) => {
                        error!("Failed to refresh users: {}", e);
                    }
                }
            }
            UiEvent::ChatNotify => {
                info!("Sending chat notification");
                let chat_service = self.chat_service.write().await;
                match chat_service.send_text("[NOTIFICATION]").await {
                    Ok(_) => {
                        info!("Notification sent");
                    }
                    Err(e) => {
                        error!("Failed to send notification: {}", e);
                    }
                }
            }

            // Settings
            UiEvent::SettingsSave => {
                info!("Saving settings");
                // Reload config from disk first to get any changes from sync_* functions
                let disk_config = match rabbit_config::load_config() {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        warn!("Failed to load config from disk: {}, using view_model config", e);
                        self.view_model.read().await.get_config()
                    }
                };
                // Update view_model with disk config
                self.view_model.write().await.update_config(disk_config.clone());

                let config = disk_config;
                let modules = &config.modules;
                let autostart = modules.get_bool("global", config::keys::global::AUTOSTART).unwrap_or(false);
                let systray = modules.get_bool("global", config::keys::global::SYSTRAY).unwrap_or(true);
                let top = modules.get_bool("global", config::keys::global::TOP).unwrap_or(false);
                let http_shell = modules.get_bool("http", config::keys::http::SHELL).unwrap_or(false);

                // Apply autostart setting
                if let Err(e) = adapter::autostart::set_autostart(autostart) {
                    warn!("Failed to set autostart: {}", e);
                }

                let old_systray = adapter::tray::is_active();
                if old_systray != systray {
                    adapter::systray::update(systray).await;
                }

                // Apply window topmost setting
                if crate::ui_state::UiState::get_main_window().is_some() {
                    let top_value = top;
                    fltk::app::awake_callback(move || {
                        adapter::set_window_on_top(top_value);
                    });
                }

                // Apply HTTP shell integration
                if let Ok(exe_path) = std::env::current_exe() {
                    let exe_path_str = exe_path.to_string_lossy().to_string();
                    match adapter::shell::set_shell_integration(http_shell, &exe_path_str) {
                        Ok(_) => {
                            info!("HTTP shell integration updated: {}", http_shell);
                        }
                        Err(e) => {
                            warn!("Failed to set HTTP shell integration: {}", e);
                        }
                    }
                }

                save_config(&config)?;
                // 配置已保存，新配置在下次启动时生效
                // 不触碰当前运行状态（配置保存与业务运行完全分离）
            }

            // Version Check
            UiEvent::VersionCheck => {
                info!("Checking for version updates");
            }
        }
        Ok(())
    }
}

async fn handle_ui_data(data: UiData, _view_model: &Arc<RwLock<AppViewModel>>) {
    match data {
        UiData::ServiceStatus(module, running, reason) => {
            info!("Received ServiceStatus: {:?} running={} reason={:?}", module, running, reason);
            let module_name = match module {
                Module::Ping => "ping",
                Module::Scan => "scan",
                Module::Http => "http",
                _ => "",
            };
            if !module_name.is_empty() {
                crate::ui_state::update_module_running(module_name, running);
            }
            if let Module::Ping = module {
                if let Some(mut win) = crate::ui_state::UiState::get_main_window() {
                        if running {
                            let target = rabbit_config::load_config()
                                .ok()
                                .and_then(|cfg| cfg.modules.get_string("ping", config::keys::ping::TARGET))
                                .unwrap_or_else(|| "Ping".to_string());
                            fltk::app::awake_callback(move || {
                                win.set_label(&format!("Ping {}", target));
                            });
                        } else {
                            adapter::taskbar::TaskbarProgress::clear();
                            fltk::app::awake_callback(move || {
                                win.set_label("Rabbit");
                            });
                        }
                    }
                }
            // 记录停止原因
            if !running {
                if let Some(ref reason_str) = reason {
                    info!("{:?} stopped: {}", module, reason_str);
                }
            }
            // 使用 awake_callback 直接更新按钮状态
            fltk::app::awake_callback(move || {
                crate::ui::ui_refresh::update_button_state(module, running);
            });
            fltk::app::awake();
        }
        UiData::PingStats(stats) => {
            info!("app.rs: Received PingStats={}", stats);
            crate::ui_state::set_ping_stats(&stats);
            // 事件驱动：直接刷新 UI，无轮询
            fltk::app::awake_callback(|| {
                crate::ui::ui_refresh::refresh_displays();
            });
        }
        UiData::Log(module, msg) => {
            match module {
                Module::Ping => {
                    info!("app.rs: Received PingLog={}", msg);
                    crate::ui_state::write_to("ping_output", &msg);
                }
                Module::Http => crate::ui_state::write_to("http_log", &crate::ui_state::fmt_log(&msg)),
                Module::Scan => crate::ui_state::write_to("scan_output", &msg),
                Module::Tftpd => crate::ui_state::write_to("tftpd_log", &crate::ui_state::fmt_log(&msg)),
                Module::Tftpc => crate::ui_state::write_to("tftpc_log", &crate::ui_state::fmt_log(&msg)),
                Module::Chat => crate::ui_state::write_to("chat_messages", &format!("[{}]", msg)),
                Module::Plan => crate::ui_state::write_to("plan_output", &crate::ui_state::fmt_log(&msg)),
            }
            // 事件驱动：直接刷新 UI，无轮询
            fltk::app::awake_callback(|| {
                crate::ui::ui_refresh::refresh_displays();
            });
        }

        UiData::PingState { progress, total, color, .. } => {
            // 任务栏由 PingState 数据驱动：直接使用预计算的数据更新任务栏
            adapter::taskbar::TaskbarProgress::update(progress, total, &color);
        }
        UiData::ScanProgress(msg) => {
            crate::ui_state::write_to("scan_output", &msg);
            fltk::app::awake_callback(|| {
                crate::ui::ui_refresh::refresh_displays();
            });
        }
        UiData::PlanReminder(msg) => {
            info!("Plan reminder received: {}", msg);
            fltk::app::awake_callback(move || {
                // 显示全屏黑屏提醒
                crate::ui::reminder_window::show_reminder(&msg);
            });
        }
        UiData::ChatMessage(username, msg) => {
            crate::ui_state::write_to("chat_messages", &format!("[{}] {}", username, msg));
            fltk::app::awake_callback(|| {
                crate::ui::ui_refresh::refresh_displays();
            });
        }
        UiData::ChatUserList(users) => {
            // users 已经是 Vec<String>，无需再 split
            crate::ui_state::set_chat_users(&users);
            fltk::app::awake_callback(|| {
                crate::ui::ui_refresh::refresh_displays();
            });
        }
    }
}

async fn handle_http_data(data: UiData) {
    if let UiData::Log(_, msg) = data {
        crate::ui_state::write_to("http_log", &crate::ui_state::fmt_log(&msg));
    }
}

async fn handle_tftp_data(data: UiData) {
    if let UiData::Log(module, msg) = data {
        match module {
            Module::Tftpd => crate::ui_state::write_to("tftpd_log", &crate::ui_state::fmt_log(&msg)),
            Module::Tftpc => crate::ui_state::write_to("tftpc_log", &crate::ui_state::fmt_log(&msg)),
            _ => {}
        }
    }
}

/// TFTP client options parsed from semicolon-separated key=value string.
#[allow(dead_code)]
struct TftpOptions {
    timeout: i32,
    max_retry: i32,
    blk_size: i32,
}

/// Parse TFTP option string into structured options.
///
/// Format: `"timeout=200;retry=10;blksize=1024"`
/// Used by both TftpClientPut and TftpClientGet event handlers.
fn parse_tftp_options(options: &str) -> TftpOptions {
    let mut timeout = 200;
    let mut max_retry = 10;
    let mut blk_size = 1024;

    for opt in options.split(';') {
        let parts: Vec<&str> = opt.splitn(2, '=').collect();
        if parts.len() == 2 {
            match parts[0].trim() {
                "timeout" => {
                    if let Ok(val) = parts[1].parse::<i32>() {
                        timeout = val;
                    }
                }
                "retry" => {
                    if let Ok(val) = parts[1].parse::<i32>() {
                        max_retry = val;
                    }
                }
                "blksize" => {
                    if let Ok(val) = parts[1].parse::<i32>() {
                        blk_size = val;
                    }
                }
                _ => {}
            }
        }
    }

    info!("TFTP client options parsed: timeout={}ms, retry={}, blksize={}", timeout, max_retry, blk_size);
    TftpOptions { timeout, max_retry, blk_size }
}
