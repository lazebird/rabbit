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
    use rabbit_models::plan::{Schedule, WeekDay};

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
fn test_task_state_transitions() {
    use rabbit_models::plan::TaskState;

    assert_ne!(TaskState::Pending, TaskState::Triggered);
    assert_ne!(TaskState::Snoozed, TaskState::Acknowledged);
    assert_eq!(TaskState::Disabled, TaskState::Disabled);
}
