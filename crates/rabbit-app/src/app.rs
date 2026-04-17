//! Application Controller

use crate::view_model::AppViewModel;
use crate::AppWindow;
use rabbit_core::{ChatService, HttpService, PingService, PlanService, ScanService, TftpService};
use rabbit_models::config::Language;
use rabbit_models::ping::PingTarget;
use rabbit_models::scan::ScanRange;
use rabbit_platform::config::{load_config, save_config};
use slint::ComponentHandle;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use tokio::time::Duration;

/// Main application struct
pub struct App {
    view_model: Arc<RwLock<AppViewModel>>,
    ping_service: Arc<RwLock<PingService>>,
    http_service: Arc<RwLock<HttpService>>,
    tftp_service: Arc<RwLock<TftpService>>,
    plan_service: Arc<RwLock<PlanService>>,
    chat_service: Arc<RwLock<ChatService>>,
    scan_service: Arc<RwLock<ScanService>>,
}

impl App {
    /// Create a new application instance
    pub async fn new() -> anyhow::Result<Self> {
        info!("Initializing Rabbit application");

        // Load configuration
        let config = load_config().unwrap_or_default();

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
        })
    }

    /// Run the application
    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Running Rabbit application");

        // Create UI
        let ui = AppWindow::new()?;

        // Initialize UI with config defaults
        self.init_ui_defaults(&ui).await;

        // Setup UI callbacks and bindings
        self.setup_ui_callbacks(&ui).await?;

        // Start background update task
        self.start_ui_update_task(&ui);

        // Run the UI event loop
        ui.run()?;

        // Cleanup
        self.cleanup().await?;

        Ok(())
    }

    /// Start background task to update UI with service data
    fn start_ui_update_task(&self, ui: &AppWindow) {
        // Use slint::Timer for UI updates to avoid Send issues
        let ui_handle = ui.as_weak();
        let ping_service = Arc::clone(&self.ping_service);

        // Spawn a task that sends updates to UI thread via invoke_from_event_loop
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;

                // Collect data first
                let (targets, results_text, stats_text) = {
                    let service = ping_service.read().await;
                    let targets = service.get_targets().await;
                    let mut text = String::new();
                    let mut stats = String::new();
                    if !targets.is_empty() {
                        let target = &targets[0];
                        let results = service.get_results(&target.address).await;
                        
                        // Format ping results like old version: "来自 1.1.1.1 的回复: 字节=32 毫秒=1 TTL=253"
                        text = results.iter()
                            .map(|r| {
                                if let Some(duration) = r.duration_ms {
                                    let ttl_str = r.ttl.map(|t| format!(" TTL={}", t)).unwrap_or_default();
                                    format!("来自 {} 的回复: 字节={} 毫秒={}{}", 
                                        target.address, r.bytes, duration as u32, ttl_str)
                                } else {
                                    format!("来自 {} 的回复: 请求超时", target.address)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        
                        // Calculate statistics
                        let sent = results.len() as u32;
                        let received = results.iter().filter(|r| r.success).count() as u32;
                        let lost = sent - received;
                        let loss_rate = if sent > 0 { (lost as f64 / sent as f64) * 100.0 } else { 0.0 };
                        
                        let times: Vec<f64> = results.iter()
                            .filter_map(|r| r.duration_ms)
                            .collect();
                        
                        let min_ms = times.iter().cloned().fold(f64::INFINITY, f64::min);
                        let max_ms = times.iter().cloned().fold(0.0, f64::max);
                        let avg_ms = if !times.is_empty() { 
                            times.iter().sum::<f64>() / times.len() as f64 
                        } else { 0.0 };
                        
                        let now = chrono::Local::now();
                        stats = format!("{} Tx {} Rx {} Loss {} Min {} Max {} Avg {:.6}",
                            now.format("%Y/%m/%d %H:%M:%S"),
                            sent, received, lost,
                            if min_ms == f64::INFINITY { 0.0 } else { min_ms } as u32,
                            max_ms as u32, avg_ms
                        );
                    }
                    (targets, text, stats)
                };

                // Update UI from UI thread
                if !results_text.is_empty() {
                    let ui = ui_handle.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_ping_results(results_text.into());
                            ui.set_ping_stats(stats_text.into());
                        }
                    }).ok();
                }
            }
        });

        // HTTP log update task
        let ui_handle = ui.as_weak();
        let http_service = Arc::clone(&self.http_service);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(2));
            loop {
                interval.tick().await;

                let (logs_text, running) = {
                    let service = http_service.read().await;
                    let logs = service.get_recent_logs(10).await;
                    let text = logs.iter()
                        .map(|l| format!("{} {} {} - {}", l.timestamp.format("%H:%M:%S"), l.method, l.path, l.status_code))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let running = service.get_state().await == rabbit_models::http::HttpServerState::Running;
                    (text, running)
                };

                let ui = ui_handle.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui.upgrade() {
                        if !logs_text.is_empty() {
                            ui.set_http_log(logs_text.into());
                        }
                        ui.set_http_running(running);
                    }
                }).ok();
            }
        });

        // TFTP Server log update task
        let ui_handle = ui.as_weak();
        let tftp_service = Arc::clone(&self.tftp_service);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;

                let (server_running, logs_text, in_progress) = {
                    let service = tftp_service.read().await;
                    let server_running = service.is_server_running();
                    let logs = service.get_logs().await;
                    let text = logs.iter()
                        .map(|l| format!("{} {} {} - {}", l.timestamp.format("%H:%M:%S"), l.operation, l.filename, if l.success { "OK" } else { "FAIL" }))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let transfers = service.get_active_transfers().await;
                    let in_progress = !transfers.is_empty();
                    (server_running, text, in_progress)
                };

                let ui = ui_handle.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui.upgrade() {
                        ui.set_tftp_server_running(server_running);
                        ui.set_tftp_transfer_in_progress(in_progress);
                        if !logs_text.is_empty() {
                            ui.set_tftp_server_log(logs_text.clone().into());
                            ui.set_tftp_client_log(logs_text.into());
                        }
                    }
                }).ok();
            }
        });

        // Scan progress update task
        let ui_handle = ui.as_weak();
        let scan_service = Arc::clone(&self.scan_service);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            loop {
                interval.tick().await;

                let (progress, found, running, results_text) = {
                    let service = scan_service.read().await;
                    if let Some(progress) = service.get_progress().await {
                        let results_text = if let Some(results) = service.get_last_results().await {
                            results.iter()
                                .filter(|r| r.online)
                                .map(|r| {
                                    if let Some(time) = r.response_time_ms {
                                        format!("{} - {:.1}ms", r.ip, time)
                                    } else {
                                        format!("{} - Online", r.ip)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        } else {
                            String::new()
                        };
                        (progress.percentage as i32, progress.found_hosts as i32, progress.is_scanning, results_text)
                    } else {
                        (0, 0, false, String::new())
                    }
                };

                let ui = ui_handle.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui.upgrade() {
                        ui.set_scan_progress(progress);
                        ui.set_scan_found(found);
                        ui.set_scan_running(running);
                        if !results_text.is_empty() {
                            ui.set_scan_results(results_text.into());
                        }
                    }
                }).ok();
            }
        });

        // Chat messages update task
        let ui_handle = ui.as_weak();
        let chat_service = Arc::clone(&self.chat_service);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            let mut last_count = 0;
            loop {
                interval.tick().await;

                let (count, messages_text) = {
                    let service = chat_service.read().await;
                    let messages = service.get_messages().await;
                    let count = messages.len();
                    let text = messages.iter()
                        .map(|m| format!("[{}] {}: {}", m.timestamp.format("%H:%M:%S"), m.sender, m.content))
                        .collect::<Vec<_>>()
                        .join("\n");
                    (count, text)
                };

                if count != last_count {
                    last_count = count;
                    let ui = ui_handle.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_chat_messages(messages_text.into());
                        }
                    }).ok();
                }
            }
        });
    }

    /// Initialize UI with default values from config
    async fn init_ui_defaults(&self, ui: &AppWindow) {
        let view_model = self.view_model.read().await;
        let config = view_model.get_config();

        // Ping defaults
        ui.set_ping_target(config.modules.ping.target.clone().into());
        ui.set_ping_opts(config.modules.ping.opts_string().into());

        // Scan defaults
        ui.set_scan_start_ip(config.modules.scan.start_ip.clone().into());
        ui.set_scan_end_ip(config.modules.scan.end_ip.clone().into());
        ui.set_scan_opts(config.modules.scan.opts_string().into());

        // HTTP defaults
        ui.set_http_port(config.modules.http.port as i32);
        ui.set_http_opts(config.modules.http.opts_string().into());

        // TFTPD defaults
        ui.set_tftp_server_port(config.modules.tftpd.port as i32);
        ui.set_tftp_server_opts(config.modules.tftpd.opts_string().into());

        // TFTPC defaults
        ui.set_tftp_client_server_addr(config.modules.tftpc.server_addr.clone().into());
        ui.set_tftp_client_server_port(config.modules.tftpc.server_port as i32);
        ui.set_tftp_client_local_path(config.modules.tftpc.local_path.clone().into());
        ui.set_tftp_client_remote_file(config.modules.tftpc.remote_file.clone().into());
        ui.set_tftp_client_opts(config.modules.tftpc.opts_string().into());

        // Plan defaults
        ui.set_plan_date(config.modules.plan.date.clone().into());
        ui.set_plan_time(config.modules.plan.time.clone().into());
        ui.set_plan_cycle(config.modules.plan.cycle);
        ui.set_plan_unit(config.modules.plan.unit.clone().into());
        ui.set_plan_msg(config.modules.plan.msg.clone().into());
        ui.set_plan_override(config.modules.plan.override_conflicts);

        // Chat defaults
        ui.set_chat_username(config.modules.chat.username.clone().into());
        ui.set_chat_port(config.modules.chat.port as i32);
        ui.set_chat_broadcast_addr(config.modules.chat.broadcast_addr.clone().into());

        // Settings defaults
        ui.set_settings_language(match config.language {
            Language::Chinese => "zh".into(),
            Language::English => "en".into(),
            Language::System => "en".into(),
        });
        ui.set_settings_systray(config.systray);
        ui.set_settings_top(config.top);
        ui.set_settings_autostart(config.autostart);
        ui.set_settings_autoupdate(config.autoupdate);
    }

    /// Setup UI callbacks
    async fn setup_ui_callbacks(&self, ui: &AppWindow) -> anyhow::Result<()> {
        // ===== Ping callbacks =====
        let ping_service = Arc::clone(&self.ping_service);
        let ui_handle = ui.as_weak();
        ui.on_ping_start(move |target: slint::SharedString, opts: slint::SharedString| {
            let service = Arc::clone(&ping_service);
            let ui = ui_handle.clone();
            let target_str = target.to_string();
            let opts_str = opts.to_string();
            // Update UI immediately
            if let Some(ui) = ui.upgrade() {
                ui.set_ping_running(true);
                ui.set_status_text(format!("Pinging {}...", target_str).into());
            }
            // Spawn async task
            tokio::spawn(async move {
                let mut service = service.write().await;
                let mut ping_target = PingTarget::new(&target_str);
                // Parse opts string (format: key1=value1;key2=value2)
                for opt in opts_str.split(';') {
                    let parts: Vec<&str> = opt.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        match parts[0].trim() {
                            "interval" => {
                                if let Ok(val) = parts[1].parse::<u64>() {
                                    ping_target.interval_ms = val;
                                }
                            }
                            "count" => {
                                if let Ok(val) = parts[1].parse::<u32>() {
                                    ping_target.count = val;
                                }
                            }
                            "stoponloss" => {
                                ping_target.stop_on_loss = parts[1].trim().eq_ignore_ascii_case("true");
                            }
                            _ => {}
                        }
                    }
                }
                if let Err(e) = service.add_target(ping_target).await {
                    error!("Failed to add ping target: {}", e);
                }
                if let Err(e) = service.start().await {
                    error!("Failed to start ping: {}", e);
                }
            });
        });

        let ping_service = Arc::clone(&self.ping_service);
        let ui_handle = ui.as_weak();
        ui.on_ping_stop(move || {
            let service = Arc::clone(&ping_service);
            let ui = ui_handle.clone();
            if let Some(ui) = ui.upgrade() {
                ui.set_ping_running(false);
                ui.set_status_text("Ping stopped".into());
            }
            tokio::spawn(async move {
                let mut service = service.write().await;
                if let Err(e) = service.stop().await {
                    error!("Failed to stop ping: {}", e);
                }
            });
        });

        // ===== HTTP callbacks =====
        let http_service = Arc::clone(&self.http_service);
        let ui_handle = ui.as_weak();
        ui.on_http_toggle(move |port: i32, opts: slint::SharedString, _dirs: slint::ModelRc<slint::SharedString>| {
            let service = Arc::clone(&http_service);
            let ui = ui_handle.clone();
            let opts_str = opts.to_string();
            if let Some(ui) = ui.upgrade() {
                ui.set_status_text(format!("Toggling HTTP server on port {}...", port).into());
            }
            tokio::spawn(async move {
                let mut service = service.write().await;
                let state = service.get_state().await;
                let _result = match state {
                    rabbit_models::http::HttpServerState::Running => {
                        service.stop().await
                    }
                    _ => {
                        let mut config = service.get_config().await;
                        config.enabled = true;
                        config.port = port as u16;
                        // Parse opts string
                        for opt in opts_str.split(';') {
                            let parts: Vec<&str> = opt.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                match parts[0].trim() {
                                    "shell" => config.shell = parts[1].trim().eq_ignore_ascii_case("true"),
                                    "autoindex" => config.auto_index = parts[1].trim().eq_ignore_ascii_case("true"),
                                    "videoplay" => config.video_play = parts[1].trim().eq_ignore_ascii_case("true"),
                                    _ => {}
                                }
                            }
                        }
                        service.update_config(config).await
                    }
                };
            });
        });

        // ===== HTTP File Browser =====
        let ui_handle = ui.as_weak();
        ui.on_http_browse(move || {
            let ui = ui_handle.clone();
            std::thread::spawn(move || {
                match rabbit_platform::dialog::open_folder_dialog("Select HTTP Root Directory", None) {
                    Ok(Some(path)) => {
                        let path_str = path.to_string_lossy().to_string();
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Selected directory: {}", path_str).into());
                            // Note: Directory management would need proper ModelRc handling
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Failed to open dialog: {}", e).into());
                        }
                    }
                }
            });
        });

        // ===== HTTP Directory Management =====
        let ui_handle = ui.as_weak();
        ui.on_http_add_dir(move || {
            let ui = ui_handle.clone();
            std::thread::spawn(move || {
                match rabbit_platform::dialog::open_folder_dialog("Add HTTP Directory", None) {
                    Ok(Some(path)) => {
                        let path_str = path.to_string_lossy().to_string();
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Added directory: {}", path_str).into());
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Failed to open dialog: {}", e).into());
                        }
                    }
                }
            });
        });

        let ui_handle = ui.as_weak();
        ui.on_http_remove_dir(move |index: i32| {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text(format!("Remove directory at index: {}", index).into());
            }
        });

        // ===== TFTP Server callbacks =====
        let tftp_service = Arc::clone(&self.tftp_service);
        let ui_handle = ui.as_weak();
        ui.on_tftp_server_toggle(move |port: i32, opts: slint::SharedString, _work_dirs: slint::ModelRc<slint::SharedString>| {
            let service = Arc::clone(&tftp_service);
            let ui = ui_handle.clone();
            let opts_str = opts.to_string();
            if let Some(ui) = ui.upgrade() {
                ui.set_status_text(format!("Toggling TFTP server on port {}...", port).into());
            }
            tokio::spawn(async move {
                let mut service = service.write().await;
                let is_running = service.is_server_running();
                let _result = if is_running {
                    service.stop_server().await
                } else {
                    let mut config = rabbit_models::tftp::TftpServerConfig {
                        enabled: true,
                        bind_addr: format!("0.0.0.0:{}", port),
                        ..Default::default()
                    };
                    // Parse opts string (timeout, maxretry, blksize, qsize, qtout, override, fslog)
                    for opt in opts_str.split(';') {
                        let parts: Vec<&str> = opt.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            match parts[0].trim() {
                                "timeout" => {
                                    if let Ok(val) = parts[1].parse::<u64>() {
                                        config.timeout_secs = val;
                                    }
                                }
                                "blksize" => {
                                    if let Ok(val) = parts[1].parse::<usize>() {
                                        config.block_size = val;
                                    }
                                }
                                "override" => config.allow_overwrite = parts[1].trim().eq_ignore_ascii_case("true"),
                                _ => {}
                            }
                        }
                    }
                    service.update_server_config(config).await
                };
            });
        });

        // ===== TFTP Directory Management =====
        let ui_handle = ui.as_weak();
        ui.on_tftp_add_dir(move || {
            let ui = ui_handle.clone();
            std::thread::spawn(move || {
                match rabbit_platform::dialog::open_folder_dialog("Add TFTP Work Directory", None) {
                    Ok(Some(path)) => {
                        let path_str = path.to_string_lossy().to_string();
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Added TFTP directory: {}", path_str).into());
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Failed to open dialog: {}", e).into());
                        }
                    }
                }
            });
        });

        let ui_handle = ui.as_weak();
        ui.on_tftp_remove_dir(move |index: i32| {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text(format!("Remove TFTP directory at index: {}", index).into());
            }
        });

        // ===== TFTP Client callbacks =====
        let tftp_service = Arc::clone(&self.tftp_service);
        let ui_handle = ui.as_weak();
        ui.on_tftp_get(move |server: slint::SharedString, port: i32, local_path: slint::SharedString, remote_file: slint::SharedString, opts: slint::SharedString| {
            let service = Arc::clone(&tftp_service);
            let ui = ui_handle.clone();
            let server_str = server.to_string();
            let server_with_port = format!("{}:{}", server_str, port);
            let file_str = remote_file.to_string();
            let path_str = local_path.to_string();
            let opts_str = opts.to_string();

            if let Some(ui) = ui.upgrade() {
                ui.set_tftp_transfer_in_progress(true);
                ui.set_status_text(format!("Downloading {} from {}...", file_str, server_with_port).into());
            }

            tokio::spawn(async move {
                let svc = service.read().await;
                // Parse client opts (timeout, maxretry, blksize)
                let mut client_config = rabbit_models::tftp::TftpClientConfig::default();
                for opt in opts_str.split(';') {
                    let parts: Vec<&str> = opt.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        match parts[0].trim() {
                            "timeout" => {
                                if let Ok(val) = parts[1].parse::<u64>() {
                                    client_config.timeout_secs = val;
                                }
                            }
                            "blksize" => {
                                if let Ok(val) = parts[1].parse::<usize>() {
                                    client_config.block_size = val;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                match svc.download_from(&server_with_port, &file_str, &path_str).await {
                    Ok(id) => {
                        info!("Download started: {}", id);
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Download started: {}", file_str).into());
                        }
                    }
                    Err(e) => {
                        error!("Failed to start download: {}", e);
                        if let Some(ui) = ui.upgrade() {
                            ui.set_tftp_transfer_in_progress(false);
                            ui.set_status_text(format!("Download failed: {}", e).into());
                        }
                    }
                }
            });
        });

        let tftp_service = Arc::clone(&self.tftp_service);
        let ui_handle = ui.as_weak();
        ui.on_tftp_put(move |server: slint::SharedString, port: i32, local_path: slint::SharedString, remote_file: slint::SharedString, opts: slint::SharedString| {
            let service = Arc::clone(&tftp_service);
            let ui = ui_handle.clone();
            let server_str = server.to_string();
            let server_with_port = format!("{}:{}", server_str, port);
            let file_str = remote_file.to_string();
            let path_str = local_path.to_string();
            let opts_str = opts.to_string();

            if let Some(ui) = ui.upgrade() {
                ui.set_tftp_transfer_in_progress(true);
                ui.set_status_text(format!("Uploading {} to {}...", file_str, server_with_port).into());
            }

            tokio::spawn(async move {
                let svc = service.read().await;
                // Parse client opts (timeout, maxretry, blksize)
                let mut client_config = rabbit_models::tftp::TftpClientConfig::default();
                for opt in opts_str.split(';') {
                    let parts: Vec<&str> = opt.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        match parts[0].trim() {
                            "timeout" => {
                                if let Ok(val) = parts[1].parse::<u64>() {
                                    client_config.timeout_secs = val;
                                }
                            }
                            "blksize" => {
                                if let Ok(val) = parts[1].parse::<usize>() {
                                    client_config.block_size = val;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                match svc.upload_to(&server_with_port, &path_str, &file_str).await {
                    Ok(id) => {
                        info!("Upload started: {}", id);
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Upload started: {}", file_str).into());
                        }
                    }
                    Err(e) => {
                        error!("Failed to start upload: {}", e);
                        if let Some(ui) = ui.upgrade() {
                            ui.set_tftp_transfer_in_progress(false);
                            ui.set_status_text(format!("Upload failed: {}", e).into());
                        }
                    }
                }
            });
        });

        // ===== TFTP Client File Browser =====
        let ui_handle = ui.as_weak();
        ui.on_tftp_browse_local(move || {
            let ui = ui_handle.clone();
            std::thread::spawn(move || {
                match rabbit_platform::dialog::open_file_dialog(
                    "Select File for TFTP Transfer",
                    None,
                    &[("All Files", &["*"])]
                ) {
                    Ok(Some(path)) => {
                        let path_str = path.to_string_lossy().to_string();
                        if let Some(ui) = ui.upgrade() {
                            ui.set_tftp_client_local_path(path_str.into());
                            // Also set remote filename to the file name
                            if let Some(filename) = path.file_name() {
                                ui.set_tftp_client_remote_file(filename.to_string_lossy().to_string().into());
                            }
                            ui.set_status_text("File selected".into());
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Failed to open dialog: {}", e).into());
                        }
                    }
                }
            });
        });

        // ===== Scan callbacks =====
        let scan_service = Arc::clone(&self.scan_service);
        let ui_handle = ui.as_weak();
        ui.on_scan_start(move |start_ip: slint::SharedString, end_ip: slint::SharedString| {
            let service = Arc::clone(&scan_service);
            let ui = ui_handle.clone();
            let start = start_ip.to_string();
            let end = end_ip.to_string();
            let start_addr: Option<Ipv4Addr> = start.parse().ok();
            let end_addr: Option<Ipv4Addr> = end.parse().ok();
            match (start_addr, end_addr) {
                (Some(s), Some(e)) => {
                    if let Some(ui) = ui.upgrade() {
                        ui.set_scan_running(true);
                        ui.set_status_text(format!("Scanning {} - {}...", s, e).into());
                    }
                    tokio::spawn(async move {
                        let mut service = service.write().await;
                        let range = ScanRange::new(s, e);
                        if let Err(e) = service.scan(range).await {
                            error!("Failed to start scan: {}", e);
                        }
                    });
                }
                _ => {
                    if let Some(ui) = ui.upgrade() {
                        ui.set_status_text("Invalid IP range".into());
                    }
                }
            }
        });

        let scan_service = Arc::clone(&self.scan_service);
        let ui_handle = ui.as_weak();
        ui.on_scan_stop(move || {
            let service = Arc::clone(&scan_service);
            let ui = ui_handle.clone();
            if let Some(ui) = ui.upgrade() {
                ui.set_scan_running(false);
                ui.set_status_text("Scan stopped".into());
            }
            tokio::spawn(async move {
                let mut service = service.write().await;
                if let Err(e) = service.cancel().await {
                    error!("Failed to stop scan: {}", e);
                }
            });
        });

        // Scan export callback
        let scan_service = Arc::clone(&self.scan_service);
        let ui_handle = ui.as_weak();
        ui.on_scan_export(move |filename: slint::SharedString| {
            let service = Arc::clone(&scan_service);
            let ui = ui_handle.clone();
            let filename_str = filename.to_string();

            if let Some(ui) = ui.upgrade() {
                ui.set_status_text(format!("Exporting to {}...", filename_str).into());
            }

            tokio::spawn(async move {
                let svc = service.read().await;
                let results = svc.get_online_hosts().await;

                if results.is_empty() {
                    if let Some(ui) = ui.upgrade() {
                        ui.set_status_text("No results to export".into());
                    }
                    return;
                }

                let content = results.iter()
                    .map(|r| {
                        let hostname = r.hostname.as_deref().unwrap_or("N/A");
                        let time = r.response_time_ms.map(|t| format!("{:.1}ms", t)).unwrap_or_else(|| "N/A".to_string());
                        format!("{},{},{},{}", r.ip, if r.online { "Online" } else { "Offline" }, hostname, time)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                let header = "IP,Status,Hostname,ResponseTime\n".to_string();
                let content = header + &content;

                match std::fs::write(&filename_str, content) {
                    Ok(_) => {
                        info!("Exported {} hosts to {}", results.len(), filename_str);
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Exported {} hosts to {}", results.len(), filename_str).into());
                        }
                    }
                    Err(e) => {
                        error!("Failed to export: {}", e);
                        if let Some(ui) = ui.upgrade() {
                            ui.set_status_text(format!("Export failed: {}", e).into());
                        }
                    }
                }
            });
        });

        // ===== Chat callbacks =====
        let chat_service = Arc::clone(&self.chat_service);
        let ui_handle = ui.as_weak();
        ui.on_chat_toggle(move |enable: bool, username: slint::SharedString, broadcast_addr: slint::SharedString, port: i32| {
            let service = Arc::clone(&chat_service);
            let ui = ui_handle.clone();
            let username_str = username.to_string();
            let broadcast_str = broadcast_addr.to_string();
            if let Some(ui) = ui.upgrade() {
                ui.set_chat_enabled(enable);
                ui.set_status_text(if enable { "Enabling chat..." } else { "Disabling chat..." }.into());
            }
            tokio::spawn(async move {
                let mut service = service.write().await;
                if enable {
                    let _ = service.start().await;
                } else {
                    let _ = service.stop().await;
                }
            });
        });

        let chat_service = Arc::clone(&self.chat_service);
        ui.on_chat_send(move |msg: slint::SharedString| {
            let service = Arc::clone(&chat_service);
            let message = msg.to_string();
            tokio::spawn(async move {
                let service = service.read().await;
                if let Err(e) = service.send_text(&message).await {
                    error!("Failed to send chat message: {}", e);
                }
            });
        });

        let ui_handle = ui.as_weak();
        ui.on_chat_refresh(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Refreshing user list...".into());
            }
        });

        let ui_handle = ui.as_weak();
        ui.on_chat_notify(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Sending broadcast notification...".into());
            }
        });

        // ===== Plan callbacks =====
        let plan_service = Arc::clone(&self.plan_service);
        let ui_handle = ui.as_weak();
        ui.on_plan_add(move |date: slint::SharedString, time: slint::SharedString, cycle: i32, unit: slint::SharedString, msg: slint::SharedString, override_conflicts: bool| {
            let service = Arc::clone(&plan_service);
            let ui = ui_handle.clone();
            let task_id = msg.to_string();
            let date_str = date.to_string();
            let time_str = time.to_string();
            let unit_str = unit.to_string();
            if let Some(ui) = ui.upgrade() {
                ui.set_status_text(format!("Adding task {}...", task_id).into());
            }
            tokio::spawn(async move {
                use rabbit_models::plan::{Task, Schedule, TaskState};
                // Parse date and time - for now use Once schedule
                let schedule = Schedule::Once {
                    datetime: chrono::Local::now() + chrono::Duration::hours(1),
                };
                let task = Task {
                    id: task_id.clone(),
                    title: task_id,
                    description: Some(format!("Date: {}, Time: {}, Cycle: {} {}", date_str, time_str, cycle, unit_str)),
                    schedule,
                    enabled: true,
                    created_at: chrono::Local::now(),
                    state: TaskState::Pending,
                    snooze_until: None,
                    last_triggered: None,
                };
                let svc = service.read().await;
                if let Err(e) = svc.add_task(task).await {
                    error!("Failed to add task: {}", e);
                }
            });
        });

        let plan_service = Arc::clone(&self.plan_service);
        let ui_handle = ui.as_weak();
        ui.on_plan_remove(move |task_id: i32| {
            let service = Arc::clone(&plan_service);
            let ui = ui_handle.clone();
            if task_id >= 0 {
                let id_str = format!("task_{}", task_id);
                if let Some(ui) = ui.upgrade() {
                    ui.set_status_text(format!("Removing task {}...", task_id).into());
                }
                tokio::spawn(async move {
                    let svc = service.read().await;
                    if let Err(e) = svc.remove_task(&id_str).await {
                        error!("Failed to remove task: {}", e);
                    }
                });
            } else if let Some(ui) = ui.upgrade() {
                ui.set_status_text("Select a task to remove".into());
            }
        });

        let ui_handle = ui.as_weak();
        ui.on_plan_select(move |task_id: i32| {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text(format!("Selected task {}", task_id).into());
            }
        });

        // ===== Settings callbacks =====
        let view_model = Arc::clone(&self.view_model);
        let ui_handle = ui.as_weak();
        ui.on_settings_save(move || {
            let vm = Arc::clone(&view_model);
            let ui = ui_handle.clone();
            if let Some(ui) = ui.upgrade() {
                ui.set_status_text("Saving settings...".into());
            }
            tokio::spawn(async move {
                let mut vm = vm.write().await;
                if let Err(e) = vm.save_settings().await {
                    error!("Failed to save settings: {}", e);
                } else {
                    info!("Settings saved");
                }
            });
        });

        let ui_handle = ui.as_weak();
        ui.on_settings_reset(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_settings_language("en".into());
                ui.set_settings_theme("light".into());
                ui.set_settings_systray(true);
                ui.set_settings_top(false);
                ui.set_settings_autostart(false);
                ui.set_settings_autoupdate(true);
                ui.set_status_text("Settings reset to defaults".into());
            }
        });

        // ===== Settings Links =====
        let ui_handle = ui.as_weak();
        ui.on_settings_open_home(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Opening home page...".into());
                if let Err(e) = rabbit_platform::dialog::open_url("https://github.com/lazebird/rabbit") {
                    error!("Failed to open URL: {}", e);
                }
            }
        });

        let ui_handle = ui.as_weak();
        ui.on_settings_open_config(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Opening config directory...".into());
                match rabbit_platform::config_dir() {
                    Ok(config_dir) => {
                        if let Err(e) = rabbit_platform::dialog::open_file_manager(
                            config_dir.to_str().unwrap_or(".")
                        ) {
                            error!("Failed to open config directory: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get config directory: {}", e);
                    }
                }
            }
        });

        let ui_handle = ui.as_weak();
        ui.on_settings_open_help(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Opening help...".into());
                if let Err(e) = rabbit_platform::dialog::open_url("https://github.com/lazebird/rabbit/blob/main/README.md") {
                    error!("Failed to open URL: {}", e);
                }
            }
        });

        let ui_handle = ui.as_weak();
        ui.on_settings_check_update(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_status_text("Checking for updates...".into());
                // For now, just open the releases page
                if let Err(e) = rabbit_platform::dialog::open_url("https://github.com/lazebird/rabbit/releases") {
                    error!("Failed to open URL: {}", e);
                }
            }
        });

        Ok(())
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
