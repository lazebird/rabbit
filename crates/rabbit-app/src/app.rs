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
use rabbit_core::{ChatService, HttpService, PingService, PlanService, ScanService, TftpService};
use rabbit_models::{AppConfig, ping::PingTarget, scan::ScanRange};
use rabbit_platform::config::{load_config, save_config};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{info, warn, error};
use ctrlc;

/// Main application struct
pub struct App {
    view_model: Arc<RwLock<AppViewModel>>,
    ping_service: Arc<RwLock<PingService>>,
    http_service: Arc<RwLock<HttpService>>,
    tftp_service: Arc<RwLock<TftpService>>,
    plan_service: Arc<RwLock<PlanService>>,
    chat_service: Arc<RwLock<ChatService>>,
    scan_service: Arc<RwLock<ScanService>>,
    ping_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    scan_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    shutdown_flag: Arc<AtomicBool>,
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

        // Create services
        let mut ping_service = PingService::new();
        let ping_log_file = config.modules.ping.log.clone();
        ping_service.init(ping_log_file).await?;

        let mut http_service = HttpService::new();
        http_service.init((&config.modules.http).into()).await?;

        let mut tftp_service = TftpService::new();
        tftp_service.init(
            (&config.modules.tftpd).into(),
            (&config.modules.tftpc).into(),
        ).await?;

        let mut plan_service = PlanService::new();
        plan_service.init().await?;
        plan_service.start().await?;

        let mut chat_service = ChatService::new();
        chat_service.init((&config.modules.chat).into()).await?;

        let mut scan_service = ScanService::new();
        scan_service.init(rabbit_models::scan::ScannerConfig::default()).await?;

        // Create view model
        let view_model = AppViewModel::new(config);

        Ok(Self {
            view_model: Arc::new(RwLock::new(view_model)),
            ping_service: Arc::new(RwLock::new(ping_service)),
            http_service: Arc::new(RwLock::new(http_service)),
            tftp_service: Arc::new(RwLock::new(tftp_service)),
            plan_service: Arc::new(RwLock::new(plan_service)),
chat_service: Arc::new(RwLock::new(chat_service)),
            scan_service: Arc::new(RwLock::new(scan_service)),
            ping_task: Arc::new(RwLock::new(None)),
            scan_task: Arc::new(RwLock::new(None)),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Run the application
    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Running Rabbit application with FLTK UI");

        // Initialize UI state and event system
        let _ui_state = UiState::init();
        
        // Restore HTTP and TFTP directories from config
        {
            let config = self.view_model.read().await.get_config();
            // Restore HTTP items
            for dir in &config.modules.http.dirs {
                crate::ui_state::add_http_item(dir);
            }
            // Restore TFTP directories
            for dir in &config.modules.tftpd.work_dirs {
                crate::ui_state::add_tftpd_dir(dir);
            }
            // Restore TFTP working directory selection
            if let Some(idx) = config.modules.tftpd.working_dir_index {
                // Convert 0-based config index to 1-based UI index
                if idx < config.modules.tftpd.work_dirs.len() {
                    crate::ui_state::set_tftpd_selected((idx + 1) as i32);
                }
            }
        }
        
        let event_receiver = init_event_system();

        // Create FLTK application
        let fltk_app = app::App::default();

        // Set application-wide colors (lighter theme - similar to old version)
        app::background(0xF0, 0xF0, 0xF0);      // Light gray background
        app::background2(0xFF, 0xFF, 0xFF);     // White for inputs
        app::foreground(0x00, 0x00, 0x00);       // Black text
        app::set_visible_focus(true);

        // Create main window
        let mut main_win = Window::new(100, 100, 748, 518, "Rabbit");
        main_win.set_type(WindowType::Double);
        main_win.make_resizable(true);

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
        let last_tab = config.last_active_tab;
        let autoupdate = config.autoupdate;
        let top_requested = config.top;
        let systray_requested = config.systray;
        let autostart_requested = config.autostart;
        let http_shell_requested = config.modules.http.shell;
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
                if let Ok(mut vm) = view_model_for_tab.try_write() {
                    let mut cfg = vm.get_config();
                    cfg.last_active_tab = idx;
                    vm.update_config(cfg.clone());
                    save_config(&cfg).ok();
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

        // Let tabs fill the window on resize
        let mut tabs_clone = tabs.clone();
        main_win.resize_callback(move |w, _x, _y, nw, nh| {
            tabs_clone.resize(5, 5, nw - 10, nh - 10);
            w.redraw();
        });

        main_win.end();
        main_win.show();

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
        let ping_restore = config_for_restore.modules.ping.running;
        let http_restore = config_for_restore.modules.http.running;
        let tftp_restore = config_for_restore.modules.tftpd.running;
        let chat_restore = config_for_restore.modules.chat.running;
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
        #[cfg(target_os = "windows")]
        let window_handle: Option<usize> = Some(main_win.raw_handle() as usize);
        #[cfg(not(target_os = "windows"))]
        let window_handle: Option<usize> = None;
        
        let config = self.view_model.read().await.get_config();
        let taskbar_enabled = config.modules.ping.taskbar;
        drop(config);
        
        let app_clone = Arc::new(RwLock::new(AppHandle {
            view_model: self.view_model.clone(),
            ping_service: self.ping_service.clone(),
            http_service: self.http_service.clone(),
            tftp_service: self.tftp_service.clone(),
            plan_service: self.plan_service.clone(),
            chat_service: self.chat_service.clone(),
            scan_service: self.scan_service.clone(),
            ping_task: self.ping_task.clone(),
            scan_task: self.scan_task.clone(),
            #[cfg(target_os = "windows")]
            window_handle,
            ping_history: std::collections::HashMap::new(),
            taskbar_enabled,
        }));

        let event_handle = tokio::spawn(async move {
            Self::event_loop(app_clone, event_receiver).await;
        });

        // Restore business states after event loop is ready
        if ping_restore_flag {
            info!("Restoring ping service state");
            // Get target and options from config and start ping
            let config = self.view_model.read().await.get_config();
            let target = config.modules.ping.target.clone();
            let options = config.modules.ping.opts_string();  // Use saved options including interval
            drop(config);
            info!("Restoring ping with options: {}", options);
            send_event(UiEvent::PingStart { target, options });
        }
        
        if http_restore_flag {
            info!("Restoring HTTP server state");
            let config = self.view_model.read().await.get_config();
            let port = config.modules.http.port;
            let autoindex = config.modules.http.autoindex;
            let videoplay = config.modules.http.videoplay;
            let shell = config.modules.http.shell;
            drop(config);
            let options = format!("autoindex={};videoplay={}", autoindex, videoplay);
            send_event(UiEvent::HttpToggle { port, options, shell });
        }
        
        if tftp_restore_flag {
            info!("Restoring TFTP server state");
            let config = self.view_model.read().await.get_config();
            let options = config.modules.tftpd.opts_string();
            drop(config);
            info!("Restoring TFTP server with options: {}", options);
            send_event(UiEvent::TftpServerToggle { options });
        }
        
        if chat_restore_flag {
            info!("Restoring chat state");
            let config = self.view_model.read().await.get_config();
            let username = config.modules.chat.username.clone();
            let port = config.modules.chat.port;
            let broadcast = config.modules.chat.broadcast_addr.clone();
            drop(config);
            send_event(UiEvent::ChatToggle { username, port, broadcast });
        }

        // Run FLTK event loop
        fltk_app.run()?;

        // Abort the event loop task (it's blocked on receiver.recv() which will never return)
        event_handle.abort();

        // Cleanup
        self.cleanup().await?;

        // Force exit - FLTK may leave internal threads running
        std::process::exit(0);

        Ok(())
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
        self.ping_service.write().await.stop().await.ok();
        self.http_service.write().await.stop().await.ok();
        self.tftp_service.write().await.stop_server().await.ok();
        self.plan_service.write().await.stop().await.ok();
        self.chat_service.write().await.stop().await.ok();

        // Save running states to configuration
        let mut config = self.view_model.read().await.get_config();
        config.modules.ping.running = ping_running;
        config.modules.http.running = http_running;
        config.modules.tftpd.running = tftp_running;
        config.modules.chat.running = chat_running;
        
        // Update view model with new config
        self.view_model.write().await.update_config(config.clone());
        
        // Save configuration
        save_config(&config).ok();
        
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
    tftp_service: Arc<RwLock<TftpService>>,
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
            // Ping
            UiEvent::PingStart { target, options } => {
                info!("Starting ping to {}", target);

                // Cancel any existing ping task first
                if let Some(handle) = self.ping_task.write().await.take() {
                    handle.abort();
                }

                // Update UI state

                // Set window title to target address
                let target_label = target.clone();
                if let Some(mut win) = crate::ui_state::UiState::get_main_window() {
                    fltk::app::awake_callback(move || {
                        win.set_label(&target_label);
                    });
                }

                // Parse options string to extract interval
                let mut interval = self.view_model.read().await.get_config().modules.ping.interval as u64;
                let mut count: i32 = -1;  // Default to infinite
                let mut stop_on_loss = false;

                for opt in options.split(';') {
                    let parts: Vec<&str> = opt.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        match parts[0].trim() {
                            "interval" => {
                                if let Ok(val) = parts[1].parse::<u64>() {
                                    interval = val;
                                }
                            }
                            "count" => {
                                // Parse as i32 to support -1 (infinite)
                                if let Ok(val) = parts[1].parse::<i32>() {
                                    count = val;
                                }
                            }
                            "stoponloss" => {
                                stop_on_loss = parts[1].trim().eq_ignore_ascii_case("true");
                            }
                            _ => {}
                        }
                    }
                }

                info!("Ping options parsed: interval={}ms, count={} (negative=infinite), stoponloss={}", interval, count, stop_on_loss);

                // Save ping configuration to persist settings
                {
                    let mut vm = self.view_model.write().await;
                    let mut config = vm.get_config();
                    config.modules.ping.interval = interval as i32;
                    config.modules.ping.count = count;
                    config.modules.ping.stoponloss = stop_on_loss;
                    config.modules.ping.target = target.clone();
                    vm.update_config(config.clone());
                    if let Err(e) = save_config(&config) {
                        warn!("Failed to save ping config: {}", e);
                    }
                    info!("Ping configuration saved: interval={}, count={}, stoponloss={}", 
                          config.modules.ping.interval, config.modules.ping.count, config.modules.ping.stoponloss);
                }

                let mut service = self.ping_service.write().await;
                service.start().await?;

                let mut target_obj = PingTarget::new(&target);
                target_obj.interval_ms = interval;
                // Convert i32 count to u32 (negative values mean infinite)
                target_obj.count = if count < 0 { u32::MAX } else { count as u32 };
                target_obj.stop_on_loss = stop_on_loss;
                service.add_target(target_obj).await?;
                self.view_model.write().await.set_ping_running(true);
                crate::ui_state::set_ping_running(true);
                
                info!("Ping target configured: interval={}ms, count={} ({})", 
                      interval, 
                      if count < 0 { -1 } else { count },
                      if count < 0 { "infinite" } else { "finite" });
                
                // Clear ping history for this target
                self.ping_history.insert(target.clone(), Vec::new());

                // Spawn a task to periodically update ping results
                let ping_service = self.ping_service.clone();
                let target_clone = target.clone();
                #[cfg(target_os = "windows")]
                let window_handle = self.window_handle;
                let taskbar_enabled = self.taskbar_enabled;
                
                let handle = tokio::spawn(async move {
                    // Track last 5 ping results for taskbar
                    let mut recent_results: Vec<bool> = Vec::with_capacity(5);
                    
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        let service = ping_service.read().await;
                        let results = service.get_results(&target_clone).await;
                        let summary = service.get_summary(&target_clone).await;
                        drop(service);

                        if results.is_empty() {
                            if let Some(s) = summary {
                                let stats = format!(
                                    "Tx {} Rx {} Loss {} Min {:.1}ms Max {:.1}ms Avg {:.1}ms",
                                    s.sent, s.received, s.lost,
                                    s.min_ms.unwrap_or(0.0),
                                    s.max_ms.unwrap_or(0.0),
                                    s.avg_ms.unwrap_or(0.0)
                                );
                                crate::ui_state::set_ping_stats(&stats);
                            }
                            continue;
                        }

                        for result in &results {
                            if result.success {
                                let line = if let Some(ttl) = result.ttl {
                                    format!(
                                        "Reply from {}: bytes={} time={:.1}ms TTL={}",
                                        target_clone,
                                        result.bytes,
                                        result.duration_ms.unwrap_or(0.0),
                                        ttl
                                    )
                                } else {
                                    format!(
                                        "Reply from {}: bytes={} time={:.1}ms",
                                        target_clone,
                                        result.bytes,
                                        result.duration_ms.unwrap_or(0.0)
                                    )
                                };
                                crate::ui_state::append_ping_output(&line);
                            } else {
                                crate::ui_state::append_ping_output("Request timed out.");
                            }
                            
                            // Track for taskbar (last 5 results)
                            if taskbar_enabled {
                                recent_results.push(result.success);
                                if recent_results.len() > 5 {
                                    recent_results.remove(0);
                                }
                            }
                        }
                        
                        // Update taskbar if enabled
                        #[cfg(target_os = "windows")]
                        if taskbar_enabled && !recent_results.is_empty() {
                            if let Some(hwnd) = window_handle {
                                let success_count = recent_results.iter().filter(|&&x| x).count() as u32;
                                let total_count = recent_results.len() as u32;
                                rabbit_platform::taskbar::windows::update_taskbar_for_ping(
                                    hwnd, success_count, total_count
                                );
                            }
                        }

                        // Update stats
                        if let Some(s) = summary {
                            let stats = format!(
                                "Tx {} Rx {} Loss {} Min {:.1}ms Max {:.1}ms Avg {:.1}ms",
                                s.sent, s.received, s.lost,
                                s.min_ms.unwrap_or(0.0),
                                s.max_ms.unwrap_or(0.0),
                                s.avg_ms.unwrap_or(0.0)
                            );
                            crate::ui_state::set_ping_stats(&stats);
                        }
                    }
                });

                // Store the task handle
                *self.ping_task.write().await = Some(handle);
            }
            UiEvent::PingStop => {
                info!("Stopping ping");
                // Cancel the background task first
                if let Some(handle) = self.ping_task.write().await.take() {
                    handle.abort();
                }
                self.ping_service.write().await.stop().await?;
                self.view_model.write().await.set_ping_running(false);
                crate::ui_state::set_ping_running(false);

                // Reset window title to "Rabbit"
                if let Some(mut win) = crate::ui_state::UiState::get_main_window() {
                    fltk::app::awake_callback(move || {
                        win.set_label("Rabbit");
                    });
                }

                // Clear taskbar progress
                #[cfg(target_os = "windows")]
                if self.taskbar_enabled {
                    if let Some(hwnd) = self.window_handle {
                        rabbit_platform::taskbar::windows::set_taskbar_state(
                            hwnd,
                            rabbit_platform::taskbar::windows::TaskbarState::None,
                            0
                        ).ok();
                    }
                }
            }

            // Scan
            UiEvent::ScanStart { start_ip, end_ip, options } => {
                info!("Starting scan from {} to {} with options: {}", start_ip, end_ip, options);

                // Cancel any existing scan task first
                if let Some(handle) = self.scan_task.write().await.take() {
                    handle.abort();
                }

                // Parse options string
                let mut filter = true;
                for opt in options.split(';') {
                    let parts: Vec<&str> = opt.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        match parts[0].trim() {
                            "filter" => {
                                filter = parts[1].trim().eq_ignore_ascii_case("true");
                            }
                            _ => {}
                        }
                    }
                }
                info!("Scan options parsed: filter={}", filter);

                // Save scan configuration
                crate::ui_state::sync_scan_config(start_ip.clone(), end_ip.clone(), filter);

                let mut service = self.scan_service.write().await;
                // Parse IP addresses
                let start: std::net::Ipv4Addr = start_ip.parse()
                    .map_err(|e| anyhow::anyhow!("Invalid start IP: {}", e))?;
                let end: std::net::Ipv4Addr = end_ip.parse()
                    .map_err(|e| anyhow::anyhow!("Invalid end IP: {}", e))?;
                let range = ScanRange::new(start, end);
                
                // Note: ScannerConfig controls internal performance parameters
                // The 'filter' option from ScanConfig is logged for future use
                info!("Scan will use filter={} (currently not applied to ScannerConfig)", filter);
                
                service.scan(range).await?;
                drop(service);
                crate::ui_state::set_scan_running(true);

                // Spawn a task to periodically update scan results
                let scan_service = self.scan_service.clone();
                let handle = tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                        let service = scan_service.read().await;
                        let progress = service.get_progress().await;
                        let results = service.get_online_hosts().await;
                        let state = service.get_state().await;
                        drop(service);

                        // Build output text
                        let mut output = String::new();
                        if let Some(p) = progress {
                            output.push_str(&format!("Scanning... {}% complete, {} hosts found\r\n", p.percentage, p.found_hosts));
                        }
                        for result in &results {
                            let mac_str = result.mac_address.as_deref().unwrap_or("N/A");
                            let latency = result.response_time_ms.unwrap_or(0.0);
                            output.push_str(&format!("{} (MAC: {}, latency: {:.2}ms)\r\n",
                                result.ip, mac_str, latency));
                        }

                        if matches!(state, rabbit_models::scan::ScannerState::Completed) {
                            output.push_str(&format!("\nScan completed. {} hosts found.\r\n", results.len()));
                            crate::ui_state::set_scan_output(&output);
                            crate::ui_state::set_scan_running(false);
                            break;
                        }

                        crate::ui_state::set_scan_output(&output);
                    }
                });

                // Store the task handle
                *self.scan_task.write().await = Some(handle);
            }
            UiEvent::ScanStop => {
                info!("Stopping scan");
                // Cancel the background task first
                if let Some(handle) = self.scan_task.write().await.take() {
                    handle.abort();
                }
                self.scan_service.write().await.cancel().await?;
                crate::ui_state::set_scan_running(false);
                crate::ui_state::append_scan_output("Scan stopped.\r\n");
            }

            // HTTP Server
            UiEvent::HttpToggle { port, options, shell } => {
                let running = self.view_model.read().await.is_http_running();
                if running {
                    info!("Stopping HTTP server");
                    self.http_service.write().await.stop().await?;
                    self.view_model.write().await.set_http_running(false);
                    crate::ui_state::set_http_running(false);
                    crate::ui_state::append_http_log("\r\nHTTP server stopped.\r\n");
                } else {
                    info!("Starting HTTP server on port {} with shell={}", port, shell);
                    
                    // Save HTTP configuration
                    crate::ui_state::sync_http_start_config(port, shell, 
                        options.contains("autoindex=true"),
                        options.contains("videoplay=true"));
                    
                    // Set running state first so button changes immediately
                    crate::ui_state::set_http_running(true);
                    self.view_model.write().await.set_http_running(true);

                    let mut service = self.http_service.write().await;
                    let config = rabbit_models::http::HttpServerConfig {
                        enabled: true,
                        port,
                        root_path: String::from("."),
                        allow_upload: false,
                        allow_delete: false,
                        shell,
                        auto_index: options.contains("autoindex=true"),
                        video_play: options.contains("videoplay=true"),
                    };
                    service.init(config).await?;
                    match service.start().await {
                        Ok(_) => {
                            crate::ui_state::append_http_log(&format!("HTTP server started on port {}.\r\n", port));
                        }
                        Err(e) => {
                            let msg = format!("Failed to start HTTP server: {}", e);
                            error!("{}", msg);
                            crate::ui_state::set_http_running(false);
                            self.view_model.write().await.set_http_running(false);
                            crate::ui_state::append_http_log(&format!("ERROR: {}\r\n", msg));
                        }
                    }
                }
            }

            // TFTP Server
            UiEvent::TftpServerToggle { options } => {
                let running = self.view_model.read().await.is_tftp_server_running();
                if running {
                    info!("Stopping TFTP server");
                    self.tftp_service.write().await.stop_server().await?;
                    self.view_model.write().await.set_tftp_server_running(false);
                    crate::ui_state::append_tftpd_log("\r\nTFTP server stopped.\r\n");
                } else {
                    info!("Starting TFTP server with options: {}", options);
                    
                    // Parse options string
                    let mut timeout = 200;
                    let mut maxretry = 10;
                    let mut blksize = 512;
                    let mut qsize = 2000;
                    let mut qtout = 1000;
                    let mut override_conflicts = false;
                    let mut fslog = false;
                    
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
                                        maxretry = val;
                                    }
                                }
                                "blksize" => {
                                    if let Ok(val) = parts[1].parse::<i32>() {
                                        blksize = val;
                                    }
                                }
                                "qsize" => {
                                    if let Ok(val) = parts[1].parse::<i32>() {
                                        qsize = val;
                                    }
                                }
                                "qtout" => {
                                    if let Ok(val) = parts[1].parse::<i32>() {
                                        qtout = val;
                                    }
                                }
                                "override" => {
                                    override_conflicts = parts[1].trim().eq_ignore_ascii_case("true");
                                }
                                "fslog" => {
                                    fslog = parts[1].trim().eq_ignore_ascii_case("true");
                                }
                                _ => {}
                            }
                        }
                    }
                    info!("TFTP server options parsed: timeout={}, retry={}, blksize={}, qsize={}, qtout={}, override={}, fslog={}",
                          timeout, maxretry, blksize, qsize, qtout, override_conflicts, fslog);
                    
                    self.tftp_service.write().await.start_server().await?;
                    self.view_model.write().await.set_tftp_server_running(true);
                    crate::ui_state::append_tftpd_log(&format!("TFTP server started (timeout={}ms, retry={}, blksize={}).\r\n", timeout, maxretry, blksize));
                }
            }
            UiEvent::TftpServerAddDir => {
                info!("Add TFTP directory (not yet implemented)");
                // TODO: Implement directory picker dialog
            }
            UiEvent::TftpServerRemoveDir => {
                info!("Remove TFTP directory (not yet implemented)");
                // TODO: Implement directory removal
            }

            // TFTP Client
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
                
                let tftp_service = self.tftp_service.write().await;
                match tftp_service.upload_to(&server, &local, &remote).await {
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
                
                let tftp_service = self.tftp_service.write().await;
                match tftp_service.download_from(&server, &remote, &local).await {
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
                use chrono::{Local, NaiveDate, NaiveTime, NaiveDateTime};
                use rabbit_models::plan::{Schedule, RepeatUnit};

                // Parse datetime
                let datetime = if let (Ok(d), Ok(t)) = (
                    NaiveDate::parse_from_str(&date, "%Y/%m/%d"),
                    NaiveTime::parse_from_str(&time, "%H:%M")
                ) {
                    NaiveDateTime::new(d, t).and_local_timezone(Local).unwrap()
                } else {
                    Local::now()
                };

                let schedule = if cycle > 0 {
                    // Repeating schedule
                    let repeat_unit = match unit.as_str() {
                        "hour" => RepeatUnit::Hour,
                        "day" => RepeatUnit::Day,
                        _ => RepeatUnit::Minute, // default to minute
                    };
                    Schedule::Repeating { datetime, cycle, unit: repeat_unit }
                } else {
                    // One-time schedule
                    Schedule::Once { datetime }
                };
                
                let id = format!("task-{}", uuid::Uuid::new_v4());
                let task = rabbit_models::plan::Task::new(id, msg, schedule);
                self.plan_service.write().await.add_task(task).await?;
            }
            UiEvent::PlanRemove { id } => {
                info!("Removing plan {}", id);
                self.plan_service.write().await.remove_task(&id).await?;
            }

            // Chat
            UiEvent::ChatToggle { username, port, broadcast } => {
                let running = self.view_model.read().await.is_chat_running();
                if running {
                    info!("Stopping chat");
                    self.chat_service.write().await.stop().await?;
                    self.view_model.write().await.set_chat_running(false);
                } else {
                    info!("Starting chat as {} on port {}", username, port);
                    let mut service = self.chat_service.write().await;
                    service.init(rabbit_models::chat::ChatConfig {
                        enabled: true,
                        username,
                        port,
                        multicast_addr: broadcast,
                    }).await?;
                    service.start().await?;
                    self.view_model.write().await.set_chat_running(true);
                }
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
                let autostart = config.autostart;
                let systray = config.systray;
                let top = config.top;
                let http_shell = config.modules.http.shell;
                let new_ping_interval = config.modules.ping.interval;
                
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
                    // Stop current ping
                    self.ping_service.write().await.stop().await.ok();
                    self.view_model.write().await.set_ping_running(false);
                    crate::ui_state::set_ping_running(false);
                    
                    // Restart ping with new config
                    let target = config.modules.ping.target.clone();
                    if !target.is_empty() {
                        // Use opts_string() to get all ping settings
                        let options = config.modules.ping.opts_string();
                        info!("Restarting ping with options: {}", options);
                        // Re-send PingStart event to restart with new interval
                        send_event(UiEvent::PingStart { target, options });
                    }
                }
            }

            // Version Check
            UiEvent::VersionCheck => {
                info!("Checking for version updates");
                // Version check is handled in settings_tab.rs UI thread
                // This event can be used for programmatic checks if needed
            }
        }
        Ok(())
    }
}
