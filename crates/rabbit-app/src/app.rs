//! Application Controller - FLTK UI Implementation

use crate::view_model::AppViewModel;
use crate::ui::{TabComponent, PingTab, ScanTab, HttpTab, TftpdTab, TftpcTab, PlanTab, ChatTab, SettingsTab};
use crate::ui::check_version_update;
use crate::upgrade::{self, VersionsManifest, PlatformInfo};
use crate::ui_events::{UiEvent, init_event_system, send_event, EventHandler};
use crate::ui_state::UiState;
use fltk::{
    app,
    group::Tabs,
    image::IcoImage,
    prelude::*,
    window::{Window, WindowType},
};
use rabbit_core::{ChatService, HttpService, PingService, PlanService, ScanService, TftpdService, TftpcService, ServiceUpdateResult, ui_channel::{UiData, Module}};
use rabbit_models::AppConfig;

use rabbit_platform::config::{load_config, save_config};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tokio::task::JoinHandle;
use tracing::{info, warn, error};
use ctrlc;

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
    ping_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    scan_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    shutdown_flag: Arc<AtomicBool>,
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
        info!("Initializing Rabbit application");

        // Load configuration - use default if load fails (e.g., first run)
        let config = match load_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                info!("Failed to load config ({}), using defaults", e);
                AppConfig::default()
            }
        };

        // Create UI channels
        let (http_tx, http_rx) = mpsc::channel(100);
        let (ping_tx, ping_rx) = mpsc::channel(100);
        let (scan_tx, scan_rx) = mpsc::channel(100);
        let (tftpd_tx, tftpd_rx) = mpsc::channel(100);
        let (tftpc_tx, tftpc_rx) = mpsc::channel(100);
        let (chat_tx, chat_rx) = mpsc::channel(100);
        let (plan_tx, plan_rx) = mpsc::channel(100);

        // Create services with channels
        let mut ping_service = PingService::with_channel(ping_tx);
        ping_service.init().await?;

        let mut http_service = HttpService::with_channel(http_tx);
        http_service.init().await?;

        let mut tftp_server_service = TftpdService::with_channel(tftpd_tx);
        tftp_server_service.init().await?;
        
        let mut tftp_client_service = TftpcService::with_channel(tftpc_tx);
        tftp_client_service.init().await?;

        let mut plan_service = PlanService::with_channel(plan_tx);
        plan_service.init().await?;
        let _ = plan_service.update().await;

        let mut chat_service = ChatService::with_channel(chat_tx);
        chat_service.init().await?;

        let mut scan_service = ScanService::with_channel(scan_tx);
        scan_service.init().await?;


        // Create view model
        let view_model = AppViewModel::new(config);

Ok(Self {
            view_model: Arc::new(RwLock::new(view_model)),
            ping_service: Arc::new(RwLock::new(ping_service)),
            http_service: Arc::new(RwLock::new(http_service)),
            tftp_server_service: Arc::new(RwLock::new(tftp_server_service)),
            tftp_client_service: Arc::new(RwLock::new(tftp_client_service)),
            plan_service: Arc::new(RwLock::new(plan_service)),
            chat_service: Arc::new(RwLock::new(chat_service)),
            scan_service: Arc::new(RwLock::new(scan_service)),
            ping_task: Arc::new(RwLock::new(None)),
            scan_task: Arc::new(RwLock::new(None)),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
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
        info!("Running Rabbit application with FLTK UI");

        // Initialize UI state and event system
        let _ui_state = UiState::init();
        
        // Restore HTTP and TFTP directories from config
        {
            let config = self.view_model.read().await.get_config();
            // Restore HTTP items
            if let Some(dirs) = config.modules.get_array("http", "dirs") {
                for dir in dirs {
                    crate::ui_state::add_http_item(&dir);
                }
            }
            // Restore TFTP directories
            if let Some(dirs) = config.modules.get_array("tftpd", "work_dirs") {
                for dir in dirs {
                    crate::ui_state::add_tftpd_dir(&dir);
                }
            }
            // Restore TFTP working directory selection
            if let Some(idx) = config.modules.get_integer("tftpd", "working_dir_index") {
                if let Some(dirs) = config.modules.get_array("tftpd", "work_dirs") {
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
                    handle_plan_data(data, &view_model).await;
                }
            });
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
        let fltk_app = app::App::default();

        // Set application-wide colors (lighter theme - similar to old version)
        app::background(0xF0, 0xF0, 0xF0);      // Light gray background
        app::background2(0xFF, 0xFF, 0xFF);     // White for inputs
        app::foreground(0x00, 0x00, 0x00);       // Black text
        app::set_visible_focus(true);

        // Create main window - load config directly from disk to get window position
        let disk_config = load_config().unwrap_or_else(|_| AppConfig::default());
        let modules = &disk_config.modules;
        let win_x = modules.get_integer("global", "window_x").unwrap_or(100) as i32;
        let win_y = modules.get_integer("global", "window_y").unwrap_or(100) as i32;
        let win_w = modules.get_integer("global", "window_width").unwrap_or(748) as i32;
        let win_h = modules.get_integer("global", "window_height").unwrap_or(518) as i32;
        info!("Loaded config: window pos=({}, {}), size=({}x{})", win_x, win_y, win_w, win_h);
        drop(disk_config);
        
        let last_resize_time = Arc::new(AtomicU64::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        ));
        let save_pending = Arc::new(AtomicBool::new(false));
        
        let mut main_win = Window::new(win_x, win_y, win_w, win_h, "Rabbit");
        main_win.set_type(WindowType::Double);
        main_win.make_resizable(true);
        
        info!("Creating window at ({}, {}) size {}x{}", win_x, win_y, win_w, win_h);

        if let Ok(icon) = IcoImage::load("crates/rabbit-app/resources/icon.ico") {
            main_win.set_icon(Some(icon));
        }

        // Create Tabs widget - positioned to leave room for tab labels
        let mut tabs = Tabs::new(5, 5, 738, 508, "");

        // Build each tab - y=25 leaves room for tab labels at top
        let ping_tab = PingTab::build(5, 30, 730, 475);
        let scan_tab = ScanTab::build(5, 30, 730, 475);
        let http_tab = HttpTab::build(5, 30, 730, 475);
        let tftpd_tab = TftpdTab::build(5, 30, 730, 475);
        let tftpc_tab = TftpcTab::build(5, 30, 730, 475);
        let plan_tab = PlanTab::build(5, 30, 730, 475);
        let chat_tab = ChatTab::build(5, 30, 730, 475);
        let settings_tab = SettingsTab::build(5, 30, 730, 475);

        tabs.end();

        // Restore last active tab from config
        let config = self.view_model.read().await.get_config();
        let modules = &config.modules;
        let last_tab = modules.get_integer("global", "last_active_tab").unwrap_or(0) as usize;
        let autoupdate = modules.get_bool("global", "autoupdate").unwrap_or(true);
        let top_requested = modules.get_bool("global", "top").unwrap_or(false);
        let systray_requested = modules.get_bool("global", "systray").unwrap_or(true);
        let autostart_requested = modules.get_bool("global", "autostart").unwrap_or(false);
        let http_shell_requested = modules.get_bool("http", "shell").unwrap_or(false);
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
        let tab_groups: [&fltk::group::Flex; 8] = [
            &ping_tab, &scan_tab, &http_tab, &tftpd_tab,
            &tftpc_tab, &plan_tab, &chat_tab, &settings_tab,
        ];
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
                let idx = tab_ptrs_clone
                    .iter()
                    .position(|&p| p == ptr)
                    .unwrap_or(0);
                if let Ok(_) = view_model_for_tab.try_write() {
                    rabbit_platform::config::update_config(|cfg| {
                        cfg.modules.insert("global", "last_active_tab", rabbit_models::config::ConfigValue::Integer(idx as i64));
                    }).ok();
                }
            }
        });

        // Start centralized UI refresh loop (100ms interval, replaces 7 per-frame idle callbacks)
        crate::ui::ui_refresh::start_refresh_loop();

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
        main_win_for_keys.handle(move |win, ev| {
            use fltk::enums::Event;
            use fltk::enums::Key;
            if ev == Event::KeyDown {
                let key = app::event_key();
                match key {
                    // Esc: Exit the program
                    Key::Escape => {
                        info!("Esc key pressed - exiting program");
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
                            let idx = tab_ptrs_for_keys
                                .iter()
                                .position(|&p| p == ptr)
                                .unwrap_or(0);
                            // Trigger the button for this tab
                            crate::ui::ui_refresh::trigger_tab_button(idx);
                        }
                        true
                    }
                    // F1: Open help documentation
                    Key::F1 => {
                        info!("F1 key pressed - opening help");
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://github.com/lazebird/rabbit/blob/rewrite/doc/manual.md")
                            .spawn();
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open")
                            .arg("https://github.com/lazebird/rabbit/blob/rewrite/doc/manual.md")
                            .spawn();
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd")
                            .args(&["/c", "start", "https://github.com/lazebird/rabbit/blob/rewrite/doc/manual.md"])
                            .spawn();
                        true
                    }
                    // F2: Open project homepage
                    Key::F2 => {
                        info!("F2 key pressed - opening project homepage");
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://github.com/lazebird/rabbit")
                            .spawn();
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open")
                            .arg("https://github.com/lazebird/rabbit")
                            .spawn();
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd")
                            .args(&["/c", "start", "https://github.com/lazebird/rabbit"])
                            .spawn();
                        true
                    }
                    // F3: Open config file directory
                    Key::F3 => {
                        info!("F3 key pressed - opening config directory");
                        if let Some(config_path) = dirs::config_local_dir() {
                            let rabbit_config = config_path.join("Rabbit");
                            let path_str = rabbit_config.to_string_lossy().to_string();
                            
                            #[cfg(target_os = "linux")]
                            let _ = std::process::Command::new("xdg-open").arg(&path_str).spawn();
                            
                            #[cfg(target_os = "macos")]
                            let _ = std::process::Command::new("open").arg(&path_str).spawn();
                            
                            #[cfg(target_os = "windows")]
                            let _ = std::process::Command::new("explorer").arg(&path_str).spawn();
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
        
        let last_resize_for_thread = last_resize_time.clone();
        let save_pending_for_thread = save_pending.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let last = last_resize_for_thread.load(Ordering::Relaxed);
                let pending = save_pending_for_thread.load(Ordering::Relaxed);
                
                if pending && now.saturating_sub(last) >= 500 {
                    // 500ms has passed since last resize, save now
                    save_pending_for_thread.store(false, Ordering::Relaxed);

                    info!("Attempting to save window position...");
                    if let Some(win) = crate::ui_state::UiState::get_main_window() {
                        let win_x = win.x();
                        let win_y = win.y();
                        let win_w = win.w();
                        let win_h = win.h();
                        info!("Window position: x={}, y={}, w={}, h={}", win_x, win_y, win_w, win_h);

                        if let Ok(config_dir) = rabbit_platform::config::get_config_dir() {
                            let config_path = config_dir.join("rabbit.toml");
                            info!("Config path: {:?}", config_path);
                            if let Ok(content) = std::fs::read_to_string(&config_path) {
                                let mut new_content = content;

                                if new_content.contains("window_x =") {
                                    new_content = new_content
                                        .lines()
                                        .map(|line| {
                                            if line.starts_with("window_x =") { format!("window_x = {}", win_x) }
                                            else if line.starts_with("window_y =") { format!("window_y = {}", win_y) }
                                            else if line.starts_with("window_width =") { format!("window_width = {}", win_w) }
                                            else if line.starts_with("window_height =") { format!("window_height = {}", win_h) }
                                            else { line.to_string() }
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                } else {
                                    new_content = format!("{}\nwindow_x = {}\nwindow_y = {}\nwindow_width = {}\nwindow_height = {}\n",
                                        new_content, win_x, win_y, win_w, win_h);
                                }

                                std::fs::write(&config_path, new_content).ok();
                                info!("Window position saved to config");
                            }
                        }
                    }
                }
            }
        });
        
        main_win.resize_callback({
            let last_resize_time = last_resize_time.clone();
            let save_pending = save_pending.clone();
            move |w, x, y, nw, nh| {
                tabs_clone.resize(5, 5, nw - 10, nh - 10);
                w.redraw();

                info!("resize callback: pos=({},{}) size={}x{}", x, y, nw, nh);
                info!("Setting save_pending flag");
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                last_resize_time.store(now, Ordering::Relaxed);
                save_pending.store(true, Ordering::Relaxed);
            }
        });

        main_win.end();
        main_win.show();
        std::thread::sleep(std::time::Duration::from_millis(100));
        main_win.set_pos(win_x, win_y);

        // Store main window for title updates
        crate::ui_state::UiState::set_main_window(main_win.clone());

        // Startup version check if autoupdate is enabled
        if autoupdate {
            info!("Auto-update enabled, checking for updates...");
            let shutdown_flag = self.shutdown_flag.clone();
            std::thread::spawn(move || {
                // Wait a bit for UI to be ready, but check shutdown flag
                for _ in 0..20 {
                    if shutdown_flag.load(Ordering::SeqCst) {
                        info!("Shutdown requested, aborting version check");
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                if shutdown_flag.load(Ordering::SeqCst) {
                    info!("Shutdown requested, aborting version check");
                    return;
                }

                crate::ui_state::append_settings_output("Checking for updates...");
                handle_version_check_result(Some(&shutdown_flag));
            });
        }

        // Apply window topmost setting
        if top_requested {
            // Use set_on_top to bring window to front initially
            main_win.set_on_top();
            info!("Window topmost enabled (window brought to front)");
        }
        
        // Apply system tray setting
        if systray_requested {
            info!("System tray enabled (note: full system tray integration requires platform-specific setup)");
            // TODO: Implement full system tray integration
        }
        
        // Apply autostart setting
        if let Err(e) = rabbit_platform::autostart::set_autostart(autostart_requested) {
            warn!("Failed to set autostart: {}", e);
        }
        
        // Apply HTTP shell integration at startup
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_path_str = exe_path.to_string_lossy().to_string();
            if let Err(e) = rabbit_platform::shell::set_shell_integration(http_shell_requested, &exe_path_str) {
                warn!("Failed to set HTTP shell integration at startup: {}", e);
            } else {
                info!("HTTP shell integration applied at startup: {}", http_shell_requested);
            }
        }
        
        // Restore business running states from config
        let config_for_restore = self.view_model.read().await.get_config();
        let ping_restore = config_for_restore.modules.get_bool("ping", "running").unwrap_or(false);
        let http_restore = config_for_restore.modules.get_bool("http", "running").unwrap_or(false);
        let tftp_restore = config_for_restore.modules.get_bool("tftpd", "running").unwrap_or(false);
        let chat_restore = config_for_restore.modules.get_bool("chat", "running").unwrap_or(false);
        drop(config_for_restore);
        
        info!("Business states to restore: ping={}, http={}, tftpd={}, chat={}", 
              ping_restore, http_restore, tftp_restore, chat_restore);
        
        // Store restore flags for use after event loop starts
        let ping_restore_flag = ping_restore;
        let http_restore_flag = http_restore;
        let tftp_restore_flag = tftp_restore;
        let chat_restore_flag = chat_restore;
        
        // Handle window close button - use set_callback which fires when the X button is clicked
        let mut win_for_close = main_win.clone();
        main_win.set_callback(move |_| {
            win_for_close.hide();
            crate::ui::ui_refresh::stop_refresh_loop();
            app::quit();
        });

        // Handle Ctrl+C: use awake_callback to safely quit from UI thread
        let shutdown_flag = self.shutdown_flag.clone();
        ctrlc::set_handler(move || {
            info!("Ctrl+C received, initiating shutdown...");
            shutdown_flag.store(true, Ordering::SeqCst);
            crate::ui::ui_refresh::stop_refresh_loop();
            // Use awake_callback to execute quit() on the UI thread (thread-safe)
            fltk::app::awake_callback(|| {
                info!("Executing quit() on UI thread...");
                fltk::app::quit();
            });
        }).ok();

        // Fallback: also catch close events at the app level
        app::add_handler(|ev| {
            if ev == fltk::enums::Event::Close {
                tracing::info!("Close event detected via add_handler, stopping refresh and quitting...");
                crate::ui::ui_refresh::stop_refresh_loop();
                app::quit();
                true
            } else {
                false
            }
        });

        // Spawn event handler task
        let config = self.view_model.read().await.get_config();
        let taskbar_enabled = config.modules.get_bool("ping", "taskbar").unwrap_or(true);
        drop(config);

        let app_clone = Arc::new(RwLock::new(AppHandle {
            view_model: self.view_model.clone(),
            ping_service: self.ping_service.clone(),
            http_service: self.http_service.clone(),
            tftp_server_service: self.tftp_server_service.clone(),
            tftp_client_service: self.tftp_client_service.clone(),
            plan_service: self.plan_service.clone(),
            chat_service: self.chat_service.clone(),
            scan_service: self.scan_service.clone(),
            ping_task: self.ping_task.clone(),
            scan_task: self.scan_task.clone(),
            ping_history: std::collections::HashMap::new(),
            taskbar_enabled,
        }));


        let event_handle = tokio::spawn(async move {
            Self::event_loop(app_clone, event_receiver).await;
        });

        // Restore business states after event loop is ready
        let config = rabbit_platform::config::load_config().unwrap_or_default();
        if config.modules.get_bool("ping", "auto_start").unwrap_or(false) {
            info!("Auto-starting ping service");
            send_event(UiEvent::ModuleToggle { module: "ping".into() });
        }
        if ping_restore_flag {
            info!("Restoring ping service state");
            send_event(UiEvent::ModuleToggle { module: "ping".into() });
        }
        
        if http_restore_flag {
            info!("Restoring HTTP server state");
            send_event(UiEvent::ModuleToggle { module: "http".into() });
        }
        
        if tftp_restore_flag {
            info!("Restoring TFTP server state");
            send_event(UiEvent::ModuleToggle { module: "tftpd".into() });
        }
        
        if chat_restore_flag {
            info!("Restoring chat state");
            send_event(UiEvent::ModuleToggle { module: "chat".into() });
        }

        // Run FLTK event loop
        fltk_app.run()?;

        // Abort the event loop task (it's blocked on receiver.recv() which will never return)
        event_handle.abort();

        // Cleanup
        self.cleanup().await?;

        // Force exit - FLTK may leave internal threads running
        std::process::exit(0);
    }


    /// Event loop for processing UI events
    async fn event_loop(
        app: Arc<RwLock<AppHandle>>,
        receiver: std::sync::mpsc::Receiver<UiEvent>,
    ) {
        while let Ok(event) = receiver.recv() {
            info!("Processing UI event: {:?}", event);
            let mut handle = app.write().await;
            if let Err(e) = handle.handle_event(event).await {
                error!("Failed to handle event: {}", e);
            }
        }
    }

    /// Cleanup resources
    async fn cleanup(&self) -> anyhow::Result<()> {
        info!("Cleaning up resources");

        // Get current running states before stopping services
        let ping_running = self.view_model.read().await.is_ping_running();
        let http_running = self.view_model.read().await.is_http_running();
        let tftp_running = self.view_model.read().await.is_tftp_server_running();
        let chat_running = self.view_model.read().await.is_chat_running();

        // Stop all services
        self.ping_service.write().await.update().await.ok();
        self.http_service.write().await.update().await.ok();
        self.tftp_server_service.write().await.update().await.ok();
        self.plan_service.write().await.update().await.ok();
        self.chat_service.write().await.update().await.ok();

        // Save running states to configuration
        use rabbit_models::config::ConfigValue;
        let mut config = self.view_model.read().await.get_config();
        config.modules.insert("ping", "running", ConfigValue::Boolean(ping_running));
        config.modules.insert("http", "running", ConfigValue::Boolean(http_running));
        config.modules.insert("tftpd", "running", ConfigValue::Boolean(tftp_running));
        config.modules.insert("chat", "running", ConfigValue::Boolean(chat_running));

        self.view_model.write().await.update_config(config.clone());

        rabbit_platform::config::update_config(|cfg| {
            cfg.modules.insert("ping", "running", ConfigValue::Boolean(ping_running));
            cfg.modules.insert("http", "running", ConfigValue::Boolean(http_running));
            cfg.modules.insert("tftpd", "running", ConfigValue::Boolean(tftp_running));
            cfg.modules.insert("chat", "running", ConfigValue::Boolean(chat_running));
        }).ok();
        
        info!("Saved business running states: ping={}, http={}, tftpd={}, chat={}", 
              ping_running, http_running, tftp_running, chat_running);

        Ok(())
    }
}

/// Perform upgrade during startup (used by auto-check)
fn perform_startup_upgrade(remote: &VersionsManifest, platform_info: &PlatformInfo) {
    use crate::upgrade::{self, DownloadProgress};

    info!("Starting upgrade download for version: {}", remote.version);

    // Create temporary download path
    let temp_dir = if cfg!(target_os = "windows") {
        std::env::temp_dir().join("rabbit_update")
    } else {
        std::path::PathBuf::from("/tmp/rabbit_update")
    };

    let _ = std::fs::create_dir_all(&temp_dir);

    let temp_exe = temp_dir.join(format!("rabbit-{}", remote.version));

    crate::ui_state::append_settings_output(&format!("Downloading: {:.1} MB", platform_info.size as f64 / 1024.0 / 1024.0));

    // Download with progress
    let result = upgrade::download_update(
        platform_info,
        &temp_exe,
        Some(&|progress: DownloadProgress| {
            let pct = progress.percentage;
            let downloaded_mb = progress.downloaded as f64 / 1024.0 / 1024.0;
            let total_mb = progress.total as f64 / 1024.0 / 1024.0;
            info!("Downloading: {:.1} MB / {:.1} MB ({:.0}%)", downloaded_mb, total_mb, pct);
            // Update progress in place (last line) instead of appending
            crate::ui_state::update_settings_line(-1, &format!("  {:.0}% - {:.1} MB / {:.1} MB", pct, downloaded_mb, total_mb));
        }),
    );

    match result {
        Ok(_) => {
            info!("Download complete. Verifying and installing...");
            crate::ui_state::append_settings_output("Download complete. Verifying and installing...");
            // Install (includes verification)
            match upgrade::install_update(&temp_exe, &platform_info.sha256) {
                Ok(_) => {
                    // install_update calls std::process::exit(), so we won't reach here
                }
                Err(e) => {
                    error!("Installation failed: {}", e);
                    crate::ui_state::append_settings_output(&format!("Installation failed: {}", e));
                }
            }
        }
        Err(e) => {
            error!("Download failed: {}", e);
            crate::ui_state::append_settings_output(&format!("Download failed: {}", e));
        }
    }
}

/// Handle version check result and show dialog (reused by both auto-check and manual check)
pub fn handle_version_check_result(shutdown_flag: Option<&std::sync::atomic::AtomicBool>) {
    match check_version_update() {
        upgrade::UpdateStatus::UpdateAvailable(remote, platform_info) => {
            // Check shutdown flag before outputting
            if let Some(flag) = shutdown_flag {
                if flag.load(Ordering::SeqCst) {
                    info!("Shutdown requested, skipping version update output");
                    return;
                }
            }
            
            info!("Update available: {}", remote.version);
            // Use format_prompt for consistent display
            let msg = remote.format_prompt();
            crate::ui_state::append_settings_output(&msg);
            crate::ui_state::append_settings_output("");
            
            // Show dialog on main thread
            let remote_clone = remote.clone();
            let platform_clone = platform_info.clone();
            fltk::app::awake_callback(move || {
                show_upgrade_dialog(&remote_clone, &platform_clone);
            });
        }
        upgrade::UpdateStatus::UpToDate => {
            // Check shutdown flag before outputting
            if let Some(flag) = shutdown_flag {
                if flag.load(Ordering::SeqCst) {
                    return;
                }
            }
            
            info!("Application is up to date");
            crate::ui_state::append_settings_output("Application is up to date");
        }
        upgrade::UpdateStatus::CheckError(e) => {
            // Check shutdown flag before outputting
            if let Some(flag) = shutdown_flag {
                if flag.load(Ordering::SeqCst) {
                    return;
                }
            }
            
            warn!("Failed to check for updates: {}", e);
            crate::ui_state::append_settings_output(&format!("Update check failed: {}", e));
        }
    }
}

/// Show upgrade dialog and handle user choice (reused by both auto-check and manual check)
fn show_upgrade_dialog(remote: &VersionsManifest, platform_info: &PlatformInfo) {
    let prompt = remote.format_prompt();
    let choice = fltk::dialog::choice2_default(
        &prompt,
        "Update",
        "Later",
        "Skip This Version",
    );
    
    if choice == Some(0) {
        crate::ui_state::append_settings_output("Downloading and installing update...");
        // Spawn background thread for download/install to avoid blocking UI
        let remote = remote.clone();
        let platform_info = platform_info.clone();
        std::thread::spawn(move || {
            perform_startup_upgrade(&remote, &platform_info);
        });
    } else if choice == Some(1) {
        crate::ui_state::append_settings_output("Update deferred");
    } else if choice == Some(2) {
        crate::ui_state::append_settings_output(&format!("Version {} skipped", remote.version));
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
    ping_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    scan_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Window handle for taskbar updates (Windows only)
    #[cfg(target_os = "windows")]
    window_handle: Option<usize>,
    /// Track last 5 ping results per target for taskbar
    ping_history: std::collections::HashMap<String, Vec<bool>>,
    /// Whether taskbar integration is enabled
    taskbar_enabled: bool,
}

#[async_trait::async_trait]
impl EventHandler for AppHandle {
    async fn handle_event(&mut self, event: UiEvent) -> anyhow::Result<()> {
        match event {
            // Module Toggle - 根据配置决定启动/停止
            UiEvent::ModuleToggle { module } => {
                match module.as_str() {
                    "ping" => {
                        let is_running = self.view_model.read().await.is_ping_running();
                        if is_running {
                            // Stop ping
                            info!("Stopping ping service");
                            if let Some(handle) = self.ping_task.write().await.take() {
                                handle.abort();
                            }
                            let _ = self.ping_service.write().await.update().await;
                            self.view_model.write().await.set_ping_running(false);
                            crate::ui_state::set_ping_running(false);
                            // Reset window title
                            if let Some(mut win) = crate::ui_state::UiState::get_main_window() {
                                fltk::app::awake_callback(move || {
                                    win.set_label("Rabbit");
                                });
                            }
                        } else {
                            // Start ping - service reads config internally
                            info!("Starting ping service");

                            // Cancel any existing ping task
                            if let Some(handle) = self.ping_task.write().await.take() {
                                handle.abort();
                            }

                            // Set window title from config
                            if let Some(target) = rabbit_platform::config::load_config()
                                .ok()
                                .and_then(|c| c.modules.get_string("ping", "target").filter(|s| !s.is_empty()))
                            {
                                if let Some(mut win) = crate::ui_state::UiState::get_main_window() {
                                    fltk::app::awake_callback(move || {
                                        win.set_label(&target);
                                    });
                                }
                            }

                            // Use unified update interface - service handles start/stop internally
                            let result = self.ping_service.write().await.update().await;
                            match result {
                                ServiceUpdateResult::Started(_) => {
                                    self.view_model.write().await.set_ping_running(true);
                                    crate::ui_state::set_ping_running(true);
                                }
                                ServiceUpdateResult::Stopped(_) => {
                                    self.view_model.write().await.set_ping_running(false);
                                    crate::ui_state::set_ping_running(false);
                                }
                                _ => {}
                            }
                        }
                    }
                    "scan" => {
                        let is_running = self.view_model.read().await.is_scan_running();
                        if is_running {
                            info!("Stopping scan service");
                            self.scan_service.write().await.cancel().await?;
                            self.view_model.write().await.set_scan_running(false);
                            crate::ui_state::set_scan_running(false);
                        } else {
                            // Use unified update interface - service handles start/stop internally
                            info!("Starting scan service");
                            let result = self.scan_service.write().await.update().await;
                            match result {
                                ServiceUpdateResult::Started(_) => {
                                    self.view_model.write().await.set_scan_running(true);
                                    crate::ui_state::set_scan_running(true);
                                }
                                ServiceUpdateResult::Stopped(_) => {
                                    self.view_model.write().await.set_scan_running(false);
                                    crate::ui_state::set_scan_running(false);
                                }
                                _ => {}
                            }
                            *self.scan_task.write().await = None;
                        }
                    }
"http" => {
                        // Use unified update interface
                        let result = self.http_service.write().await.update().await;
                        match result {
                            ServiceUpdateResult::Started(_) => {
                                self.view_model.write().await.set_http_running(true);
                                crate::ui_state::set_http_running(true);
                            }
                            ServiceUpdateResult::Stopped(_) => {
                                self.view_model.write().await.set_http_running(false);
                                crate::ui_state::set_http_running(false);
                                crate::ui_state::append_http_log("HTTP server stopped.\r\n");
                            }
                            _ => {}
                        }
                    }
                    "tftpd" => {
                        // Use unified update interface
                        let result = self.tftp_server_service.write().await.update().await;
                        match result {
                            ServiceUpdateResult::Started(_) => {
                                self.view_model.write().await.set_tftp_server_running(true);
                            }
                            ServiceUpdateResult::Stopped(_) => {
                                self.view_model.write().await.set_tftp_server_running(false);
                                crate::ui_state::append_tftpd_log("TFTP server stopped.\r\n");
                            }
                            _ => {}
                        }
                    }
                    "chat" => {
                        // Use unified update interface
                        let result = self.chat_service.write().await.update().await;
                        match result {
                            ServiceUpdateResult::Started(_) => {
                                self.view_model.write().await.set_chat_running(true);
                            }
                            ServiceUpdateResult::Stopped(_) => {
                                self.view_model.write().await.set_chat_running(false);
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
                
                // Parse options (for future use)
                let mut timeout = 200;
                let mut maxretry = 10;
                let mut blksize = 1024;
                
                for opt in options.split(';') {
                    let parts: Vec<&str> = opt.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        match parts[0].trim() {
                            "timeout" => {
                                if let Ok(val) = parts[1].parse::<i32>() { timeout = val; }
                            }
                            "retry" => {
                                if let Ok(val) = parts[1].parse::<i32>() { maxretry = val; }
                            }
                            "blksize" => {
                                if let Ok(val) = parts[1].parse::<i32>() { blksize = val; }
                            }
                            _ => {}
                        }
                    }
                }
                info!("TFTP client options parsed: timeout={}ms, retry={}, blksize={}", timeout, maxretry, blksize);
                
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
                
                // Parse options (for future use)
                let mut timeout = 200;
                let mut maxretry = 10;
                let mut blksize = 1024;
                
                for opt in options.split(';') {
                    let parts: Vec<&str> = opt.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        match parts[0].trim() {
                            "timeout" => {
                                if let Ok(val) = parts[1].parse::<i32>() { timeout = val; }
                            }
                            "retry" => {
                                if let Ok(val) = parts[1].parse::<i32>() { maxretry = val; }
                            }
                            "blksize" => {
                                if let Ok(val) = parts[1].parse::<i32>() { blksize = val; }
                            }
                            _ => {}
                        }
                    }
                }
                info!("TFTP client options parsed: timeout={}ms, retry={}, blksize={}", timeout, maxretry, blksize);
                
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
                self.plan_service.write().await.add_task(&date, &time, cycle, &unit, &msg).await?;
            }

            UiEvent::PlanRemove { id } => {
                info!("Removing plan {}", id);
                self.plan_service.write().await.remove_task(&id).await?;
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
                let disk_config = match rabbit_platform::config::load_config() {
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
                let autostart = modules.get_bool("global", "autostart").unwrap_or(false);
                let systray = modules.get_bool("global", "systray").unwrap_or(true);
                let top = modules.get_bool("global", "top").unwrap_or(false);
                let http_shell = modules.get_bool("http", "shell").unwrap_or(false);
                let new_ping_interval = modules.get_integer("ping", "interval").unwrap_or(1000);
                
                // Check if ping is currently running
                let ping_was_running = self.view_model.read().await.is_ping_running();
                
                // Apply autostart setting
                if let Err(e) = rabbit_platform::autostart::set_autostart(autostart) {
                    warn!("Failed to set autostart: {}", e);
                }
                
                // Apply systray setting
                if systray {
                    info!("System tray enabled (note: full integration requires platform-specific setup)");
                }
                
                // Apply window topmost setting
                if let Some(mut win) = crate::ui_state::UiState::get_main_window() {
                    let top_value = top;
                    fltk::app::awake_callback(move || {
                        if top_value {
                            win.set_on_top();
                        }
                    });
                }
                
                // Apply HTTP shell integration
                if let Ok(exe_path) = std::env::current_exe() {
                    let exe_path_str = exe_path.to_string_lossy().to_string();
                    match rabbit_platform::shell::set_shell_integration(http_shell, &exe_path_str) {
                        Ok(_) => {
                            info!("HTTP shell integration updated: {}", http_shell);
                        }
                        Err(e) => {
                            warn!("Failed to set HTTP shell integration: {}", e);
                        }
                    }
                }
                
                save_config(&config)?;
                
                // If ping was running, restart it with new interval
                if ping_was_running {
                    info!("Ping was running, restarting with new interval: {}ms", new_ping_interval);
                    send_event(UiEvent::ModuleToggle { module: "ping".into() });
                }
            }

            // Version Check
            UiEvent::VersionCheck => {
                info!("Checking for version updates");
            }
        }
        Ok(())
    }
}

async fn handle_ui_data(
    data: UiData,
    _view_model: &Arc<RwLock<AppViewModel>>,
) {
    match data {
        UiData::ServiceStatus(module, running) => {
            info!("Received ServiceStatus: {:?} running={}", module, running);
            match module {
                Module::Ping => {
                    crate::ui_state::set_ping_running(running);
                    let mut vm = _view_model.write().await;
                    vm.set_ping_running(running);
                    if !running {
                        if let Some(mut win) = crate::ui_state::UiState::get_main_window() {
                            fltk::app::awake_callback(move || {
                                win.set_label("Rabbit");
                            });
                        }
                    }
                }
                Module::Scan => {
                    crate::ui_state::set_scan_running(running);
                    let mut vm = _view_model.write().await;
                    vm.set_scan_running(running);
                }
                _ => {}
            }
            fltk::app::awake();
        }
        UiData::PingStats(stats) => {
            info!("app.rs: Received PingStats={}", stats);
            crate::ui_state::set_ping_stats(&stats);
            fltk::app::awake();
        }
        UiData::Log(module, msg) => {
            match module {
                Module::Ping => {
                    info!("app.rs: Received PingLog={}", msg);
                    crate::ui_state::append_ping_output(&msg);
                }
                Module::Http => crate::ui_state::append_http_log(&msg),
                Module::Scan => crate::ui_state::append_scan_output(&msg),
                Module::Tftpd => crate::ui_state::append_tftpd_log(&msg),
                Module::Tftpc => crate::ui_state::append_tftpc_log(&msg),
                Module::Chat => crate::ui_state::append_chat_message(&msg, ""),
                Module::Plan => {}
            }
            fltk::app::awake();
        }

        UiData::PingState { .. } => {}
        UiData::ScanProgress(msg) => {
            crate::ui_state::append_scan_output(&msg);
        }
        UiData::PlanReminder(_msg) => {}
        UiData::ChatMessage(username, msg) => {
            crate::ui_state::append_chat_message(&username, &msg);
        }
        UiData::ChatUserList(users) => {
            let user_vec: Vec<String> = users.split(',').map(|s| s.to_string()).collect();
            crate::ui_state::set_chat_users(&user_vec);
        }
        UiData::Error(module, msg) => {
            error!("[{:?}] {}", module, msg);
        }
    }
}

async fn handle_plan_data(data: UiData, _view_model: &Arc<RwLock<AppViewModel>>) {
    if let UiData::PlanReminder(msg) = data {
        info!("Plan reminder: {}", msg);
    }
}

async fn handle_http_data(data: UiData) {
    if let UiData::Log(_, msg) = data {
        crate::ui_state::append_http_log(&msg);
    }
}

async fn handle_tftp_data(data: UiData) {
    if let UiData::Log(module, msg) = data {
        match module {
            Module::Tftpd => crate::ui_state::append_tftpd_log(&msg),
            Module::Tftpc => crate::ui_state::append_tftpc_log(&msg),
            _ => {}
        }
    }
}


