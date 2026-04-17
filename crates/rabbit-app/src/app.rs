//! Application Controller - FLTK UI Implementation

use crate::view_model::AppViewModel;
use crate::ui::{TabComponent, PingTab, ScanTab, HttpTab, TftpdTab, TftpcTab, PlanTab, ChatTab, SettingsTab};
use crate::ui_events::{UiEvent, init_event_system, EventHandler};
use crate::ui_state::UiState;
use fltk::{
    app,
    group::Tabs,
    prelude::*,
    window::{Window, WindowType},
};
use rabbit_core::{ChatService, HttpService, PingService, PlanService, ScanService, TftpService};
use rabbit_models::{AppConfig, ping::PingTarget, scan::ScanRange};
use rabbit_platform::config::{load_config, save_config};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{info, warn, error};

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
        ping_service.init().await?;

        let mut http_service = HttpService::new();
        http_service.init(rabbit_models::http::HttpServerConfig {
            enabled: false,
            port: config.modules.http.port,
            root_path: String::from("."),
            allow_upload: false,
            allow_delete: false,
            shell: config.modules.http.shell,
            auto_index: config.modules.http.autoindex,
            video_play: config.modules.http.videoplay,
        }).await?;

        let mut tftp_service = TftpService::new();
        tftp_service.init(
            rabbit_models::tftp::TftpServerConfig {
                enabled: false,
                bind_addr: format!("0.0.0.0:{}", config.modules.tftpd.port),
                root_path: String::from("."),
                block_size: config.modules.tftpd.blksize as usize,
                timeout_secs: config.modules.tftpd.timeout as u64 / 1000,
                window_size: 1,
                allow_overwrite: config.modules.tftpd.override_conflicts,
            },
            rabbit_models::tftp::TftpClientConfig::default(),
        ).await?;

        let mut plan_service = PlanService::new();
        plan_service.init().await?;
        plan_service.start().await?;

        let mut chat_service = ChatService::new();
        chat_service.init(rabbit_models::chat::ChatConfig {
            enabled: false,
            username: config.modules.chat.username.clone(),
            port: config.modules.chat.port,
            multicast_addr: config.modules.chat.broadcast_addr.clone(),
        }).await?;

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
        })
    }

    /// Run the application
    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Running Rabbit application with FLTK UI");

        // Initialize UI state and event system
        let _ui_state = UiState::init();
        let event_receiver = init_event_system();

        // Create FLTK application
        let fltk_app = app::App::default();

        // Set application-wide colors (lighter theme - similar to old version)
        app::background(0xF0, 0xF0, 0xF0);      // Light gray background
        app::background2(0xFF, 0xFF, 0xFF);     // White for inputs
        app::foreground(0x00, 0x00, 0x00);       // Black text
        app::set_visible_focus(true);

        // Create main window
        let mut main_win = Window::new(100, 100, 650, 480, "Rabbit");
        main_win.set_type(WindowType::Double);
        main_win.make_resizable(true);

        // Create Tabs widget
        let mut tabs = Tabs::new(5, 5, 640, 420, "");

        // Build each tab - FLTK tabs need specific position/size
        let tab_x = 0;
        let tab_y = 30;
        let tab_w = 640;
        let tab_h = 390;

        let _ping_tab = PingTab::build(tab_x, tab_y, tab_w, tab_h);
        let _scan_tab = ScanTab::build(tab_x, tab_y, tab_w, tab_h);
        let _http_tab = HttpTab::build(tab_x, tab_y, tab_w, tab_h);
        let _tftpd_tab = TftpdTab::build(tab_x, tab_y, tab_w, tab_h);
        let _tftpc_tab = TftpcTab::build(tab_x, tab_y, tab_w, tab_h);
        let _plan_tab = PlanTab::build(tab_x, tab_y, tab_w, tab_h);
        let _chat_tab = ChatTab::build(tab_x, tab_y, tab_w, tab_h);
        let _settings_tab = SettingsTab::build(tab_x, tab_y, tab_w, tab_h);

        tabs.end();
        main_win.end();
        main_win.show();

        // Spawn event handler task
        let app_clone = Arc::new(RwLock::new(AppHandle {
            view_model: self.view_model.clone(),
            ping_service: self.ping_service.clone(),
            http_service: self.http_service.clone(),
            tftp_service: self.tftp_service.clone(),
            plan_service: self.plan_service.clone(),
            chat_service: self.chat_service.clone(),
            scan_service: self.scan_service.clone(),
            ping_task: self.ping_task.clone(),
        }));

        tokio::spawn(async move {
            Self::event_loop(app_clone, event_receiver).await;
        });

        // Run FLTK event loop
        fltk_app.run()?;

        // Cleanup
        self.cleanup().await?;

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

        // Stop all services
        self.ping_service.write().await.stop().await.ok();
        self.http_service.write().await.stop().await.ok();
        self.tftp_service.write().await.stop_server().await.ok();
        self.plan_service.write().await.stop().await.ok();
        self.chat_service.write().await.stop().await.ok();

        // Save configuration
        let config = self.view_model.read().await.get_config();
        save_config(&config).ok();

        Ok(())
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
}

#[async_trait::async_trait]
impl EventHandler for AppHandle {
    async fn handle_event(&mut self, event: UiEvent) -> anyhow::Result<()> {
        match event {
            // Ping
            UiEvent::PingStart { target, options: _ } => {
                info!("Starting ping to {}", target);

                // Cancel any existing ping task first
                if let Some(handle) = self.ping_task.write().await.take() {
                    handle.abort();
                }

                // Update UI state
                crate::ui_state::set_ping_output(&format!("Pinging {}...\n", target));

                let mut service = self.ping_service.write().await;
                // Start service first, then add target
                service.start().await?;
                let target_obj = PingTarget::new(&target);
                service.add_target(target_obj).await?;
                self.view_model.write().await.set_ping_running(true);

                // Spawn a task to periodically update ping results
                let ping_service = self.ping_service.clone();
                let target_clone = target.clone();
                let handle = tokio::spawn(async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        let service = ping_service.read().await;
                        let results = service.get_results(&target_clone).await;
                        let summary = service.get_summary(&target_clone).await;
                        drop(service);

                        // Update UI with results
                        for result in results {
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
                crate::ui_state::append_ping_output("Ping stopped.");
            }

            // Scan
            UiEvent::ScanStart { start_ip, end_ip, options: _ } => {
                info!("Starting scan from {} to {}", start_ip, end_ip);
                let mut service = self.scan_service.write().await;
                // Parse IP addresses
                let start: std::net::Ipv4Addr = start_ip.parse()
                    .map_err(|e| anyhow::anyhow!("Invalid start IP: {}", e))?;
                let end: std::net::Ipv4Addr = end_ip.parse()
                    .map_err(|e| anyhow::anyhow!("Invalid end IP: {}", e))?;
                let range = ScanRange::new(start, end);
                service.scan(range).await?;
                self.view_model.write().await.set_scan_running(true);
            }
            UiEvent::ScanStop => {
                info!("Stopping scan");
                self.scan_service.write().await.cancel().await?;
                self.view_model.write().await.set_scan_running(false);
            }

            // HTTP Server
            UiEvent::HttpToggle { port, options, shell } => {
                let running = self.view_model.read().await.is_http_running();
                if running {
                    info!("Stopping HTTP server");
                    self.http_service.write().await.stop().await?;
                    self.view_model.write().await.set_http_running(false);
                } else {
                    info!("Starting HTTP server on port {} with shell={}", port, shell);
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
                    service.start().await?;
                    self.view_model.write().await.set_http_running(true);
                }
            }

            // TFTP Server
            UiEvent::TftpServerToggle { options } => {
                let running = self.view_model.read().await.is_tftp_server_running();
                if running {
                    info!("Stopping TFTP server");
                    self.tftp_service.write().await.stop_server().await?;
                    self.view_model.write().await.set_tftp_server_running(false);
                } else {
                    info!("Starting TFTP server with options: {}", options);
                    self.tftp_service.write().await.start_server().await?;
                    self.view_model.write().await.set_tftp_server_running(true);
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
                // TODO: Implement TFTP client put
                warn!("TFTP client put not yet implemented");
            }
            UiEvent::TftpClientGet { server, local, remote, options } => {
                info!("TFTP get {}@{} -> {} with options: {}", remote, server, local, options);
                // TODO: Implement TFTP client get
                warn!("TFTP client get not yet implemented");
            }

            // Plan
            UiEvent::PlanAdd { date, time, cycle: _, unit: _, msg } => {
                info!("Adding plan for {} {}: {}", date, time, msg);
                use chrono::{Local, NaiveDate, NaiveTime, NaiveDateTime};

                // Parse datetime
                let datetime = if let (Ok(d), Ok(t)) = (
                    NaiveDate::parse_from_str(&date, "%Y/%m/%d"),
                    NaiveTime::parse_from_str(&time, "%H:%M")
                ) {
                    NaiveDateTime::new(d, t).and_local_timezone(Local).unwrap()
                } else {
                    Local::now()
                };

                let schedule = rabbit_models::plan::Schedule::Once { datetime };
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
                // TODO: Refresh user list
            }
            UiEvent::ChatNotify => {
                info!("Sending chat notification");
                // TODO: Send notification
            }

            // Settings
            UiEvent::SettingsSave => {
                info!("Saving settings");
                let config = self.view_model.read().await.get_config();
                save_config(&config)?;
            }
        }
        Ok(())
    }
}
