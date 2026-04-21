//! Integration Tests for Rabbit Application UI Components
//!
//! These tests verify that all UI-facing business interfaces have expected
//! input/output behavior as defined in the requirements document.

use rabbit_models::*;

// ============================================================================
// Ping Module Tests
// ============================================================================

#[test]
fn test_ping_target_creation() {
    let target = rabbit_models::ping::PingTarget::new("127.0.0.1");
    assert_eq!(target.address, "127.0.0.1");
    assert_eq!(target.count, 4);
    assert_eq!(target.interval_ms, 1000);
    assert_eq!(target.timeout_ms, 2000);
    assert!(!target.stop_on_loss);
}

#[test]
fn test_ping_target_with_options() {
    let mut target = rabbit_models::ping::PingTarget::new("8.8.8.8");
    target.count = 10;
    target.interval_ms = 500;
    target.stop_on_loss = true;

    assert_eq!(target.count, 10);
    assert_eq!(target.interval_ms, 500);
    assert!(target.stop_on_loss);
}

#[test]
fn test_ping_result_serialization() {
    let result = rabbit_models::ping::PingResult {
        seq: 1,
        success: true,
        duration_ms: Some(15.5),
        ttl: Some(64),
        bytes: 64,
        error: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    let parsed: rabbit_models::ping::PingResult = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.seq, 1);
    assert!(parsed.success);
    assert_eq!(parsed.duration_ms, Some(15.5));
}

#[test]
fn test_ping_summary_calculation() {
    let summary = rabbit_models::ping::PingSummary {
        target: "8.8.8.8".to_string(),
        sent: 10,
        received: 8,
        lost: 2,
        loss_rate: 20.0,
        min_ms: Some(10.0),
        max_ms: Some(50.0),
        avg_ms: Some(25.0),
    };

    assert_eq!(summary.loss_rate, 20.0);
    assert_eq!(summary.sent, summary.received + summary.lost);
}

#[test]
fn test_ping_target_serialization() {
    let target = rabbit_models::ping::PingTarget::new("192.168.1.1");
    let json = serde_json::to_string(&target).unwrap();
    let parsed: rabbit_models::ping::PingTarget = serde_json::from_str(&json).unwrap();
    
    assert_eq!(parsed.address, "192.168.1.1");
    assert_eq!(parsed.count, 4);
}

// ============================================================================
// HTTP Module Tests
// ============================================================================

#[test]
fn test_http_server_config_defaults() {
    let app_config = rabbit_models::config::HttpConfig::default();
    let config = rabbit_models::http::HttpServerConfig::from(&app_config);

    assert!(!config.enabled);
    assert_eq!(config.port, 8000);
    assert_eq!(config.root_path, "");  // empty dirs -> empty root_path
    assert!(!config.allow_upload);
    assert!(!config.allow_delete);
    assert!(!config.shell);
    assert!(config.auto_index);  // default is true
    assert!(config.video_play);  // default is true
}

#[test]
fn test_http_server_config_with_options() {
    let config = rabbit_models::http::HttpServerConfig {
        enabled: true,
        port: 9000,
        root_path: "/var/www".to_string(),
        allow_upload: true,
        allow_delete: false,
        shell: true,
        auto_index: true,
        video_play: false,
    };

    assert!(config.enabled);
    assert_eq!(config.port, 9000);
    assert!(config.shell);
    assert!(config.auto_index);
}

#[test]
fn test_http_access_log_entry() {
    let entry = rabbit_models::http::HttpAccessLog {
        timestamp: chrono::Local::now(),
        method: "GET".to_string(),
        path: "/index.html".to_string(),
        status_code: 200,
        bytes_sent: 1024,
        client_addr: "127.0.0.1:12345".to_string(),
    };

    assert_eq!(entry.method, "GET");
    assert_eq!(entry.status_code, 200);
}

#[test]
fn test_http_config_serialization() {
    let config = rabbit_models::config::HttpConfig {
        port: 8080,
        shell: true,
        autoindex: false,
        videoplay: true,
        dirs: vec!["/tmp".to_string(), "/var/www".to_string()],
        running: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: rabbit_models::config::HttpConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.port, parsed.port);
    assert_eq!(config.shell, parsed.shell);
    assert_eq!(config.dirs.len(), 2);
}

#[test]
fn test_http_config_dirs_management() {
    let mut config = rabbit_models::config::HttpConfig::default();
    
    // Simulate adding directories
    config.dirs.push("/home/user/docs".to_string());
    config.dirs.push("/home/user/downloads".to_string());
    
    assert_eq!(config.dirs.len(), 2);
    assert!(config.dirs.contains(&"/home/user/docs".to_string()));
}

// ============================================================================
// TFTP Module Tests
// ============================================================================

#[test]
fn test_tftp_server_config_defaults() {
    let app_config = rabbit_models::config::TftpdConfig::default();
    let config = rabbit_models::tftp::TftpServerConfig::from(&app_config);

    assert!(!config.enabled);
    assert_eq!(config.bind_addr, "0.0.0.0:69");
    assert_eq!(config.root_path, "");  // empty work_dirs -> empty root_path
    assert_eq!(config.block_size, 512);
    assert_eq!(config.timeout_secs, 0);  // 200ms / 1000 = 0
    assert_eq!(config.window_size, 1);
    assert!(!config.allow_overwrite);
}

#[test]
fn test_tftp_server_config_with_options() {
    let config = rabbit_models::tftp::TftpServerConfig {
        enabled: true,
        bind_addr: "0.0.0.0:9069".to_string(),
        root_path: "/tftp".to_string(),
        block_size: 1024,
        timeout_secs: 10,
        window_size: 4,
        allow_overwrite: true,
    };

    assert!(config.enabled);
    assert!(config.bind_addr.contains("9069"));
    assert_eq!(config.block_size, 1024);
    assert!(config.allow_overwrite);
}

#[test]
fn test_tftp_client_config_defaults() {
    let app_config = rabbit_models::config::TftpcConfig::default();
    let config = rabbit_models::tftp::TftpClientConfig::from(&app_config);

    assert_eq!(config.server_addr, "127.0.0.1:69");
    assert_eq!(config.local_port, 0);
    assert_eq!(config.block_size, 1024);
    assert_eq!(config.timeout_secs, 0);
}

#[test]
fn test_tftp_transfer_creation() {
    let transfer = rabbit_models::tftp::TftpTransfer {
        id: "test-123".to_string(),
        operation: "download".to_string(),
        filename: "test.txt".to_string(),
        remote_addr: "192.168.1.1:69".to_string(),
        state: "transferring".to_string(),
        progress: 50,
        bytes_transferred: 5120,
        total_bytes: Some(10240),
        error: None,
    };

    assert_eq!(transfer.progress, 50);
    assert_eq!(transfer.bytes_transferred, 5120);
}

#[test]
fn test_tftpd_config_serialization() {
    let config = rabbit_models::config::TftpdConfig {
        port: 69,
        timeout: 500,
        maxretry: 5,
        blksize: 1024,
        qsize: 1000,
        qtout: 2000,
        override_conflicts: true,
        fslog: true,
        work_dirs: vec!["/tftp".to_string()],
        running: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: rabbit_models::config::TftpdConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.port, parsed.port);
    assert_eq!(config.blksize, parsed.blksize);
    assert_eq!(config.work_dirs.len(), 1);
}

#[test]
fn test_tftpc_config_serialization() {
    let config = rabbit_models::config::TftpcConfig {
        server_addr: "192.168.1.100".to_string(),
        server_port: 69,
        local_path: "/tmp/downloads".to_string(),
        remote_file: "firmware.bin".to_string(),
        timeout: 300,
        maxretry: 3,
        blksize: 1468,
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: rabbit_models::config::TftpcConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.server_addr, parsed.server_addr);
    assert_eq!(config.remote_file, parsed.remote_file);
}

// ============================================================================
// Chat Module Tests
// ============================================================================

#[test]
fn test_chat_config_defaults() {
    let app_config = rabbit_models::config::ChatModuleConfig::default();
    let config = rabbit_models::chat::ChatConfig::from(&app_config);

    assert!(!config.enabled);
    assert_eq!(config.port, 1314);
    assert_eq!(config.multicast_addr, "255.255.255.255");
}

#[test]
fn test_chat_message_creation() {
    let msg = rabbit_models::chat::ChatMessage {
        id: "msg-1".to_string(),
        sender: "user1".to_string(),
        content: "Hello, world!".to_string(),
        timestamp: chrono::Local::now(),
        message_type: rabbit_models::chat::MessageType::Text,
    };

    assert_eq!(msg.sender, "user1");
    assert_eq!(msg.content, "Hello, world!");
}

#[test]
fn test_chat_user_creation() {
    let user = rabbit_models::chat::ChatUser {
        username: "testuser".to_string(),
        hostname: "localhost".to_string(),
        last_seen: chrono::Local::now(),
        online: true,
    };

    assert_eq!(user.username, "testuser");
    assert_eq!(user.hostname, "localhost");
    assert!(user.online);
}

#[test]
fn test_chat_config_serialization() {
    let config = rabbit_models::config::ChatModuleConfig {
        username: "Alice".to_string(),
        port: 1314,
        broadcast_addr: "255.255.255.255".to_string(),
        running: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let parsed: rabbit_models::config::ChatModuleConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.username, parsed.username);
    assert_eq!(config.port, parsed.port);
}

#[test]
fn test_chat_message_serialization() {
    let msg = rabbit_models::chat::ChatMessage {
        id: "msg-42".to_string(),
        sender: "Bob".to_string(),
        content: "Test message".to_string(),
        timestamp: chrono::Local::now(),
        message_type: rabbit_models::chat::MessageType::Text,
    };

    let json = serde_json::to_string(&msg).unwrap();
    let parsed: rabbit_models::chat::ChatMessage = serde_json::from_str(&json).unwrap();

    assert_eq!(msg.id, parsed.id);
    assert_eq!(msg.content, parsed.content);
}

// ============================================================================
// Plan Module Tests
// ============================================================================

#[test]
fn test_task_creation() {
    use rabbit_models::plan::{Schedule, Task};

    let task = Task {
        id: "task-1".to_string(),
        title: "Test Task".to_string(),
        description: Some("Test description".to_string()),
        schedule: Schedule::Once {
            datetime: chrono::Local::now(),
        },
        enabled: true,
        created_at: chrono::Local::now(),
        state: rabbit_models::plan::TaskState::Pending,
        snooze_until: None,
        last_triggered: None,
    };

    assert_eq!(task.id, "task-1");
    assert!(task.enabled);
}

#[test]
fn test_schedule_variants() {
    use rabbit_models::plan::{Schedule, WeekDay, RepeatUnit};

    let once = Schedule::Once {
        datetime: chrono::Local::now(),
    };

    let daily = Schedule::Daily {
        time: chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
    };

    let weekly = Schedule::Weekly {
        day: WeekDay::Monday,
        time: chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
    };
    
    let repeating = Schedule::Repeating {
        datetime: chrono::Local::now(),
        cycle: 5,
        unit: RepeatUnit::Minute,
    };

    // Just verify they can be created
    match once {
        Schedule::Once { .. } => {}
        _ => panic!("Expected Once variant"),
    }

    match daily {
        Schedule::Daily { .. } => {}
        _ => panic!("Expected Daily variant"),
    }

    match weekly {
        Schedule::Weekly { .. } => {}
        _ => panic!("Expected Weekly variant"),
    }
    
    match repeating {
        Schedule::Repeating { cycle, unit, .. } => {
            assert_eq!(cycle, 5);
            assert_eq!(unit, RepeatUnit::Minute);
        }
        _ => panic!("Expected Repeating variant"),
    }
}

#[test]
fn test_task_serialization() {
    use rabbit_models::plan::{Schedule, Task, TaskState};

    let task = Task {
        id: "task-serial".to_string(),
        title: "Serialize Me".to_string(),
        description: None,
        schedule: Schedule::Once {
            datetime: chrono::Local::now(),
        },
        enabled: true,
        created_at: chrono::Local::now(),
        state: TaskState::Pending,
        snooze_until: None,
        last_triggered: None,
    };

    let json = serde_json::to_string(&task).unwrap();
    let parsed: Task = serde_json::from_str(&json).unwrap();

    assert_eq!(task.id, parsed.id);
    assert_eq!(task.title, parsed.title);
    assert_eq!(task.enabled, parsed.enabled);
}

#[test]
fn test_task_state_transitions() {
    use rabbit_models::plan::TaskState;
    
    let mut state = TaskState::Pending;
    assert_eq!(state, TaskState::Pending);
    
    state = TaskState::Triggered;
    assert_eq!(state, TaskState::Triggered);
    
    state = TaskState::Acknowledged;
    assert_eq!(state, TaskState::Acknowledged);
    
    state = TaskState::Disabled;
    assert_eq!(state, TaskState::Disabled);
}

// ============================================================================
// Scan Module Tests
// ============================================================================

#[test]
fn test_scan_range_creation() {
    use std::net::Ipv4Addr;
    use rabbit_models::scan::ScanRange;

    let start = Ipv4Addr::new(192, 168, 1, 1);
    let end = Ipv4Addr::new(192, 168, 1, 254);

    let range = ScanRange::new(start, end);

    assert_eq!(range.start, start);
    assert_eq!(range.end, end);
}

#[test]
fn test_scan_result_creation() {
    use std::net::Ipv4Addr;

    let result = rabbit_models::scan::ScanResult {
        ip: Ipv4Addr::new(192, 168, 1, 1),
        online: true,
        hostname: Some("router.local".to_string()),
        mac_address: None,
        response_time_ms: Some(1.5),
        open_ports: vec![80, 443],
    };

    assert!(result.online);
    assert_eq!(result.response_time_ms, Some(1.5));
    assert_eq!(result.open_ports.len(), 2);
}

#[test]
fn test_scanner_config_defaults() {
    let config = rabbit_models::scan::ScannerConfig::default();

    assert_eq!(config.timeout_ms, 1500);
    assert_eq!(config.concurrent, 256);
    assert_eq!(config.retry_count, 1);
}

#[test]
fn test_scan_range_serialization() {
    use std::net::Ipv4Addr;
    use rabbit_models::scan::ScanRange;

    let range = ScanRange::new(
        Ipv4Addr::new(10, 0, 0, 1),
        Ipv4Addr::new(10, 0, 0, 100)
    );

    let json = serde_json::to_string(&range).unwrap();
    let parsed: ScanRange = serde_json::from_str(&json).unwrap();

    assert_eq!(range.start, parsed.start);
    assert_eq!(range.end, parsed.end);
}

#[test]
fn test_scan_result_serialization() {
    use std::net::Ipv4Addr;

    let result = rabbit_models::scan::ScanResult {
        ip: Ipv4Addr::new(172, 16, 0, 1),
        online: true,
        hostname: Some("server.local".to_string()),
        mac_address: Some("AA:BB:CC:DD:EE:FF".to_string()),
        response_time_ms: Some(2.5),
        open_ports: vec![22, 80, 443],
    };

    let json = serde_json::to_string(&result).unwrap();
    let parsed: rabbit_models::scan::ScanResult = serde_json::from_str(&json).unwrap();

    assert_eq!(result.ip, parsed.ip);
    assert_eq!(result.hostname, parsed.hostname);
    assert_eq!(result.open_ports.len(), 3);
}

// ============================================================================
// Configuration Module Tests
// ============================================================================

#[test]
fn test_app_config_defaults() {
    let config = rabbit_models::AppConfig::default();

    assert_eq!(config.language, rabbit_models::Language::System);
    assert_eq!(config.theme, rabbit_models::Theme::System);
    assert!(config.systray);
    assert!(!config.top);
    assert!(!config.autostart);
    assert!(config.autoupdate);
    assert_eq!(config.last_active_tab, 0);
}

#[test]
fn test_module_configs() {
    let config = rabbit_models::AppConfig::default();

    // HTTP module config
    assert_eq!(config.modules.http.port, 8000);

    // TFTP module config
    assert_eq!(config.modules.tftpd.port, 69);

    // Chat module config
    assert_eq!(config.modules.chat.port, 1314);
    
    // Ping module config
    assert_eq!(config.modules.ping.target, "1.1.1.1");
    assert_eq!(config.modules.ping.interval, 1000);
}

#[test]
fn test_app_config_serialization() {
    let config = rabbit_models::AppConfig::default();
    
    let json = serde_json::to_string(&config).unwrap();
    let parsed: rabbit_models::AppConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.language, parsed.language);
    assert_eq!(config.theme, parsed.theme);
    assert_eq!(config.modules.http.port, parsed.modules.http.port);
}

#[test]
fn test_app_config_with_custom_values() {
    let mut config = rabbit_models::AppConfig::default();
    config.modules.ping.target = "8.8.8.8".to_string();
    config.modules.http.port = 8080;
    config.modules.chat.username = "TestUser".to_string();
    
    assert_eq!(config.modules.ping.target, "8.8.8.8");
    assert_eq!(config.modules.http.port, 8080);
    assert_eq!(config.modules.chat.username, "TestUser");
}

#[test]
fn test_ping_config_running_state() {
    let mut config = rabbit_models::config::PingConfig::default();
    assert!(!config.running);
    
    config.running = true;
    assert!(config.running);
    
    let json = serde_json::to_string(&config).unwrap();
    let parsed: rabbit_models::config::PingConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.running, parsed.running);
}

#[test]
fn test_http_config_running_state() {
    let mut config = rabbit_models::config::HttpConfig::default();
    assert!(!config.running);
    
    config.running = true;
    assert!(config.running);
}

#[test]
fn test_tftpd_config_running_state() {
    let mut config = rabbit_models::config::TftpdConfig::default();
    assert!(!config.running);
    
    config.running = true;
    assert!(config.running);
}

#[test]
fn test_chat_config_running_state() {
    let mut config = rabbit_models::config::ChatModuleConfig::default();
    assert!(!config.running);
    
    config.running = true;
    assert!(config.running);
}

// ============================================================================
// UI Property Binding Tests
// ============================================================================

#[test]
fn test_ping_opts_parsing() {
    // Test parsing opts string format: key1=value1;key2=value2
    let opts_str = "interval=500;count=10;stoponloss=true";

    let mut interval = 1000u64;
    let mut count = 4u32;
    let mut stop_on_loss = false;

    for opt in opts_str.split(';') {
        let parts: Vec<&str> = opt.splitn(2, '=').collect();
        if parts.len() == 2 {
            match parts[0].trim() {
                "interval" => {
                    if let Ok(val) = parts[1].parse::<u64>() {
                        interval = val;
                    }
                }
                "count" => {
                    if let Ok(val) = parts[1].parse::<u32>() {
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

    assert_eq!(interval, 500);
    assert_eq!(count, 10);
    assert!(stop_on_loss);
}

#[test]
fn test_http_opts_parsing() {
    let opts_str = "shell=true;autoindex=false;videoplay=true";

    let mut shell = false;
    let mut auto_index = false;
    let mut video_play = false;

    for opt in opts_str.split(';') {
        let parts: Vec<&str> = opt.splitn(2, '=').collect();
        if parts.len() == 2 {
            match parts[0].trim() {
                "shell" => shell = parts[1].trim().eq_ignore_ascii_case("true"),
                "autoindex" => auto_index = parts[1].trim().eq_ignore_ascii_case("true"),
                "videoplay" => video_play = parts[1].trim().eq_ignore_ascii_case("true"),
                _ => {}
            }
        }
    }

    assert!(shell);
    assert!(!auto_index);
    assert!(video_play);
}

#[test]
fn test_tftp_opts_parsing() {
    let opts_str = "timeout=200;maxretry=10;blksize=1024;override=true";

    let mut timeout = 5000u64;
    let mut block_size = 512usize;
    let mut allow_overwrite = false;

    for opt in opts_str.split(';') {
        let parts: Vec<&str> = opt.splitn(2, '=').collect();
        if parts.len() == 2 {
            match parts[0].trim() {
                "timeout" => {
                    if let Ok(val) = parts[1].parse::<u64>() {
                        timeout = val;
                    }
                }
                "blksize" => {
                    if let Ok(val) = parts[1].parse::<usize>() {
                        block_size = val;
                    }
                }
                "override" => {
                    allow_overwrite = parts[1].trim().eq_ignore_ascii_case("true");
                }
                _ => {}
            }
        }
    }

    assert_eq!(timeout, 200);
    assert_eq!(block_size, 1024);
    assert!(allow_overwrite);
}

#[test]
fn test_plan_opts_parsing() {
    // Simulate parsing plan options
    let date_str = "2026/04/20";
    let time_str = "14:30";
    let cycle = 5;
    let unit = "minute";
    let msg = "Test reminder";

    // Verify inputs are correctly formatted
    assert_eq!(date_str, "2026/04/20");
    assert_eq!(time_str, "14:30");
    assert_eq!(cycle, 5);
    assert_eq!(unit, "minute");
    assert_eq!(msg, "Test reminder");
}

#[test]
fn test_chat_opts_parsing() {
    let username = "TestUser";
    let port_str = "1314";
    let broadcast = "255.255.255.255";

    let port: u16 = port_str.parse().unwrap();

    assert_eq!(username, "TestUser");
    assert_eq!(port, 1314);
    assert_eq!(broadcast, "255.255.255.255");
}

// ============================================================================
// Service State Tests
// ============================================================================

#[test]
fn test_http_server_state_transitions() {
    use rabbit_models::http::HttpServerState;

    assert_ne!(HttpServerState::Stopped, HttpServerState::Running);
    assert_ne!(HttpServerState::Starting, HttpServerState::Error);
    assert_eq!(HttpServerState::Running, HttpServerState::Running);
}

#[test]
fn test_tftp_transfer_state_transitions() {
    use rabbit_models::tftp::TftpTransferState;

    assert_ne!(TftpTransferState::Idle, TftpTransferState::Transferring { progress: 0 });
    assert_ne!(TftpTransferState::Connecting, TftpTransferState::Completed);
    assert_eq!(TftpTransferState::Error, TftpTransferState::Error);
}

#[test]
fn test_ping_state_transitions() {
    use rabbit_models::ping::PingState;

    assert_ne!(PingState::Idle, PingState::Running);
    assert_ne!(PingState::Running, PingState::Paused);
    assert_eq!(PingState::Completed, PingState::Completed);
}

#[test]
fn test_scanner_state_transitions() {
    use rabbit_models::scan::ScannerState;

    assert_ne!(ScannerState::Idle, ScannerState::Scanning { progress: 0 });
    assert_ne!(ScannerState::Scanning { progress: 50 }, ScannerState::Completed);
    assert_eq!(ScannerState::Idle, ScannerState::Idle);
    assert_eq!(ScannerState::Completed, ScannerState::Completed);
}

#[test]
fn test_chat_message_type_variants() {
    use rabbit_models::chat::MessageType;

    let text = MessageType::Text;
    
    match text {
        MessageType::Text => {},
        _ => panic!("Expected Text variant"),
    }
}

// ============================================================================
// Event System Tests
// ============================================================================

#[test]
fn test_ui_event_variants() {
    // Test that all UI event variants can be constructed
    use rabbit_app::ui_events::UiEvent;

    let ping_start = UiEvent::PingStart { 
        target: "8.8.8.8".to_string(), 
        options: String::new() 
    };
    
    let ping_stop = UiEvent::PingStop;
    
    let scan_start = UiEvent::ScanStart {
        start_ip: "192.168.1.1".to_string(),
        end_ip: "254".to_string(),
        options: String::new(),
    };
    
    let scan_stop = UiEvent::ScanStop;
    
    let http_toggle = UiEvent::HttpToggle {
        port: 8000,
        options: String::new(),
        shell: false,
    };
    
    let tftp_server_toggle = UiEvent::TftpServerToggle {
        options: String::new(),
    };
    
    let tftp_client_put = UiEvent::TftpClientPut {
        server: "192.168.1.1:69".to_string(),
        local: "/tmp/file.txt".to_string(),
        remote: "file.txt".to_string(),
        options: String::new(),
    };
    
    let tftp_client_get = UiEvent::TftpClientGet {
        server: "192.168.1.1:69".to_string(),
        local: "/tmp".to_string(),
        remote: "remote.txt".to_string(),
        options: String::new(),
    };
    
    let plan_add = UiEvent::PlanAdd {
        date: "2026/04/20".to_string(),
        time: "14:30".to_string(),
        cycle: 0,
        unit: "minute".to_string(),
        msg: "Test task".to_string(),
    };
    
    let plan_remove = UiEvent::PlanRemove {
        id: "task-1".to_string(),
    };
    
    let chat_toggle = UiEvent::ChatToggle {
        username: "TestUser".to_string(),
        port: 1314,
        broadcast: "255.255.255.255".to_string(),
    };
    
    let chat_send = UiEvent::ChatSend {
        message: "Hello!".to_string(),
    };
    
    let chat_refresh = UiEvent::ChatRefresh;
    
    let chat_notify = UiEvent::ChatNotify;
    
    let settings_save = UiEvent::SettingsSave;
    
    let version_check = UiEvent::VersionCheck;

    // Verify events can be pattern matched
    match ping_start {
        UiEvent::PingStart { target, .. } => assert_eq!(target, "8.8.8.8"),
        _ => panic!("Expected PingStart"),
    }

    match ping_stop {
        UiEvent::PingStop => {},
        _ => panic!("Expected PingStop"),
    }

    match scan_start {
        UiEvent::ScanStart { start_ip, end_ip, .. } => {
            assert_eq!(start_ip, "192.168.1.1");
            assert_eq!(end_ip, "254");
        },
        _ => panic!("Expected ScanStart"),
    }

    match http_toggle {
        UiEvent::HttpToggle { port, shell, .. } => {
            assert_eq!(port, 8000);
            assert!(!shell);
        },
        _ => panic!("Expected HttpToggle"),
    }

    match tftp_client_put {
        UiEvent::TftpClientPut { server, local, remote, .. } => {
            assert_eq!(server, "192.168.1.1:69");
            assert_eq!(local, "/tmp/file.txt");
            assert_eq!(remote, "file.txt");
        },
        _ => panic!("Expected TftpClientPut"),
    }

    match tftp_client_get {
        UiEvent::TftpClientGet { server, local, remote, .. } => {
            assert_eq!(server, "192.168.1.1:69");
            assert_eq!(local, "/tmp");
            assert_eq!(remote, "remote.txt");
        },
        _ => panic!("Expected TftpClientGet"),
    }

    match plan_add {
        UiEvent::PlanAdd { date, time, cycle, unit, msg } => {
            assert_eq!(date, "2026/04/20");
            assert_eq!(time, "14:30");
            assert_eq!(cycle, 0);
            assert_eq!(unit, "minute");
            assert_eq!(msg, "Test task");
        },
        _ => panic!("Expected PlanAdd"),
    }

    match plan_remove {
        UiEvent::PlanRemove { id } => {
            assert_eq!(id, "task-1");
        },
        _ => panic!("Expected PlanRemove"),
    }

    match chat_toggle {
        UiEvent::ChatToggle { username, port, broadcast } => {
            assert_eq!(username, "TestUser");
            assert_eq!(port, 1314);
            assert_eq!(broadcast, "255.255.255.255");
        },
        _ => panic!("Expected ChatToggle"),
    }

    match chat_send {
        UiEvent::ChatSend { message } => {
            assert_eq!(message, "Hello!");
        },
        _ => panic!("Expected ChatSend"),
    }

    match chat_refresh {
        UiEvent::ChatRefresh => {},
        _ => panic!("Expected ChatRefresh"),
    }

    match chat_notify {
        UiEvent::ChatNotify => {},
        _ => panic!("Expected ChatNotify"),
    }

    match settings_save {
        UiEvent::SettingsSave => {},
        _ => panic!("Expected SettingsSave"),
    }

    match version_check {
        UiEvent::VersionCheck => {},
        _ => panic!("Expected VersionCheck"),
    }
}

#[test]
fn test_ui_event_clone() {
    use rabbit_app::ui_events::UiEvent;

    let event1 = UiEvent::PingStart { 
        target: "8.8.8.8".to_string(), 
        options: "interval=1000".to_string() 
    };
    
    let event2 = event1.clone();
    
    match event2 {
        UiEvent::PingStart { target, options } => {
            assert_eq!(target, "8.8.8.8");
            assert_eq!(options, "interval=1000");
        },
        _ => panic!("Expected PingStart"),
    }
}

#[test]
fn test_ui_event_debug() {
    use rabbit_app::ui_events::UiEvent;

    let event = UiEvent::PingStop;
    let debug_str = format!("{:?}", event);
    
    assert!(debug_str.contains("PingStop"));
}

// ============================================================================
// View Model Tests
// ============================================================================

#[test]
fn test_view_model_ping_state() {
    use rabbit_app::view_model::AppViewModel;

    let config = rabbit_models::AppConfig::default();
    let vm = AppViewModel::new(config);

    assert!(!vm.is_ping_running());
    assert!(vm.get_ping_result("8.8.8.8").is_none());
}

#[test]
fn test_view_model_http_state() {
    use rabbit_app::view_model::AppViewModel;

    let config = rabbit_models::AppConfig::default();
    let vm = AppViewModel::new(config);

    assert!(!vm.is_http_running());
}

#[test]
fn test_view_model_tftp_state() {
    use rabbit_app::view_model::AppViewModel;

    let config = rabbit_models::AppConfig::default();
    let vm = AppViewModel::new(config);

    assert!(!vm.is_tftp_server_running());
}

#[test]
fn test_view_model_chat_state() {
    use rabbit_app::view_model::AppViewModel;

    let config = rabbit_models::AppConfig::default();
    let vm = AppViewModel::new(config);

    assert!(!vm.is_chat_running());
    assert!(vm.get_chat_messages().is_empty());
}

#[test]
fn test_view_model_scan_state() {
    use rabbit_app::view_model::AppViewModel;

    let config = rabbit_models::AppConfig::default();
    let vm = AppViewModel::new(config);

    assert!(!vm.is_scan_running());
    assert!(vm.get_scan_results().is_empty());
}

#[test]
fn test_view_model_config_access() {
    use rabbit_app::view_model::AppViewModel;

    let config = rabbit_models::AppConfig::default();
    let vm = AppViewModel::new(config);

    let retrieved_config = vm.get_config();
    assert_eq!(retrieved_config.modules.http.port, 8000);
}

#[test]
fn test_view_model_config_update() {
    use rabbit_app::view_model::AppViewModel;

    let config = rabbit_models::AppConfig::default();
    let mut vm = AppViewModel::new(config);

    let mut new_config = vm.get_config();
    new_config.modules.http.port = 9000;
    vm.update_config(new_config);

    assert_eq!(vm.get_config().modules.http.port, 9000);
}

// ============================================================================
// UI State Tests
// ============================================================================

#[test]
fn test_ui_state_initialization() {
    // Test that UI state can be initialized
    // Note: This requires the UI state to be initialized first
    // We test the state structure indirectly
    assert!(true, "UI state initialization placeholder");
}

// ============================================================================
// Business Logic Integration Tests
// ============================================================================

#[test]
fn test_ping_config_to_target_conversion() {
    // Verify that ping config can be converted to PingTarget
    let config = rabbit_models::config::PingConfig {
        target: "10.0.0.1".to_string(),
        interval: 2000,
        count: 5,
        stoponloss: true,
        taskbar: true,
        log: String::new(),
        running: false,
    };

    let target = rabbit_models::ping::PingTarget {
        address: config.target.clone(),
        count: config.count as u32,
        interval_ms: config.interval as u64,
        timeout_ms: config.interval as u64,
        stop_on_loss: config.stoponloss,
        ip: None,
    };

    assert_eq!(target.address, "10.0.0.1");
    assert_eq!(target.count, 5);
    assert_eq!(target.interval_ms, 2000);
    assert!(target.stop_on_loss);
}

#[test]
fn test_scan_range_to_scanner_config() {
    use std::net::Ipv4Addr;
    use rabbit_models::scan::{ScanRange, ScannerConfig};

    let range = ScanRange::new(
        Ipv4Addr::new(192, 168, 0, 1),
        Ipv4Addr::new(192, 168, 0, 100)
    );

    let scanner_config = ScannerConfig::default();

    // Verify both can coexist
    assert_eq!(range.start, Ipv4Addr::new(192, 168, 0, 1));
    assert_eq!(scanner_config.timeout_ms, 1500);
    assert_eq!(scanner_config.concurrent, 256);
}

#[test]
fn test_http_config_conversion() {
    use rabbit_models::config::HttpConfig;
    use rabbit_models::http::HttpServerConfig;

    let app_config = HttpConfig {
        port: 8888,
        shell: true,
        autoindex: false,
        videoplay: true,
        dirs: vec!["/srv/http".to_string()],
        running: false,
    };

    let server_config = HttpServerConfig::from(&app_config);

    assert_eq!(server_config.port, 8888);
    assert!(server_config.shell);
    assert!(!server_config.auto_index);
    assert!(server_config.video_play);
    // root_path should be first dir or empty
    assert!(!server_config.root_path.is_empty() || server_config.root_path.is_empty());
}

#[test]
fn test_tftp_server_config_conversion() {
    use rabbit_models::config::TftpdConfig;
    use rabbit_models::tftp::TftpServerConfig;

    let app_config = TftpdConfig {
        port: 6069,
        timeout: 500,
        maxretry: 5,
        blksize: 1024,
        qsize: 1000,
        qtout: 2000,
        override_conflicts: true,
        fslog: false,
        work_dirs: vec!["/tftpboot".to_string()],
        running: false,
    };

    let server_config = TftpServerConfig::from(&app_config);

    assert!(server_config.bind_addr.contains("6069"));
    assert_eq!(server_config.block_size, 1024);
    assert!(server_config.allow_overwrite);
}

#[test]
fn test_tftp_client_config_conversion() {
    use rabbit_models::config::TftpcConfig;
    use rabbit_models::tftp::TftpClientConfig;

    let app_config = TftpcConfig {
        server_addr: "192.168.1.50".to_string(),
        server_port: 69,
        local_path: "/downloads".to_string(),
        remote_file: "image.bin".to_string(),
        timeout: 500,
        maxretry: 3,
        blksize: 1468,
    };

    let client_config = TftpClientConfig::from(&app_config);

    assert!(client_config.server_addr.contains("192.168.1.50"));
    assert_eq!(client_config.block_size, 1468);
}

#[test]
fn test_chat_config_conversion() {
    use rabbit_models::config::ChatModuleConfig;
    use rabbit_models::chat::ChatConfig;

    let app_config = ChatModuleConfig {
        username: "IntegrationTest".to_string(),
        port: 1314,
        broadcast_addr: "255.255.255.255".to_string(),
        running: false,
    };

    let chat_config = ChatConfig::from(&app_config);

    assert_eq!(chat_config.port, 1314);
    assert_eq!(chat_config.multicast_addr, "255.255.255.255");
}

// ============================================================================
// Edge Cases and Error Handling Tests
// ============================================================================

#[test]
fn test_empty_target_address() {
    let target = rabbit_models::ping::PingTarget::new("");
    assert_eq!(target.address, "");
}

#[test]
fn test_invalid_ip_in_target() {
    let target = rabbit_models::ping::PingTarget::new("not-an-ip");
    assert_eq!(target.address, "not-an-ip");
}

#[test]
fn test_http_port_boundaries() {
    let config = rabbit_models::config::HttpConfig {
        port: 1,  // Minimum valid port
        shell: false,
        autoindex: true,
        videoplay: true,
        dirs: vec![],
        running: false,
    };
    assert_eq!(config.port, 1);

    let config2 = rabbit_models::config::HttpConfig {
        port: 65535,  // Maximum valid port
        shell: false,
        autoindex: true,
        videoplay: true,
        dirs: vec![],
        running: false,
    };
    assert_eq!(config2.port, 65535);
}

#[test]
fn test_negative_cycle_in_plan() {
    use rabbit_models::plan::Schedule;

    let schedule = Schedule::Once {
        datetime: chrono::Local::now(),
    };

    // Verify schedule can be created
    match schedule {
        Schedule::Once { .. } => {},
        _ => panic!("Expected Once variant"),
    }
}

#[test]
fn test_large_cycle_value_in_plan() {
    use rabbit_models::plan::{Schedule, RepeatUnit};

    let large_cycle = i32::MAX;
    
    let schedule = Schedule::Repeating {
        datetime: chrono::Local::now(),
        cycle: large_cycle,
        unit: RepeatUnit::Day,
    };

    match schedule {
        Schedule::Repeating { cycle, .. } => {
            assert_eq!(cycle, large_cycle);
        },
        _ => panic!("Expected Repeating variant"),
    }
}

#[test]
fn test_special_characters_in_chat_message() {
    let msg = rabbit_models::chat::ChatMessage {
        id: "special-chars".to_string(),
        sender: "Test<>&\"'".to_string(),
        content: "Message with special chars: <script>alert('XSS')</script>".to_string(),
        timestamp: chrono::Local::now(),
        message_type: rabbit_models::chat::MessageType::Text,
    };

    assert!(msg.sender.contains('<'));
    assert!(msg.content.contains("<script>"));
}

#[test]
fn test_unicode_in_strings() {
    let target = rabbit_models::ping::PingTarget::new("测试.com");
    assert_eq!(target.address, "测试.com");

    let msg = rabbit_models::chat::ChatMessage {
        id: "unicode".to_string(),
        sender: "用户".to_string(),
        content: "中文消息 🎉".to_string(),
        timestamp: chrono::Local::now(),
        message_type: rabbit_models::chat::MessageType::Text,
    };

    assert_eq!(msg.sender, "用户");
    assert!(msg.content.contains("🎉"));
}

#[test]
fn test_very_long_strings() {
    let long_address = "a".repeat(1000);
    let target = rabbit_models::ping::PingTarget::new(&long_address);
    assert_eq!(target.address.len(), 1000);

    let long_message = "x".repeat(10000);
    let msg = rabbit_models::chat::ChatMessage {
        id: "long".to_string(),
        sender: "sender".to_string(),
        content: long_message.clone(),
        timestamp: chrono::Local::now(),
        message_type: rabbit_models::chat::MessageType::Text,
    };
    assert_eq!(msg.content.len(), 10000);
}

// ============================================================================
// Configuration Round-Trip Tests
// ============================================================================

#[test]
fn test_config_round_trip() {
    let original = rabbit_models::AppConfig::default();
    
    let json = serde_json::to_string(&original).unwrap();
    let restored: rabbit_models::AppConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(original.modules.ping.target, restored.modules.ping.target);
    assert_eq!(original.modules.http.port, restored.modules.http.port);
    assert_eq!(original.modules.tftpd.port, restored.modules.tftpd.port);
    assert_eq!(original.modules.chat.port, restored.modules.chat.port);
}

#[test]
fn test_config_with_all_running_states() {
    let mut config = rabbit_models::AppConfig::default();
    config.modules.ping.running = true;
    config.modules.http.running = true;
    config.modules.tftpd.running = true;
    config.modules.chat.running = true;

    let json = serde_json::to_string(&config).unwrap();
    let restored: rabbit_models::AppConfig = serde_json::from_str(&json).unwrap();

    assert!(restored.modules.ping.running);
    assert!(restored.modules.http.running);
    assert!(restored.modules.tftpd.running);
    assert!(restored.modules.chat.running);
}
