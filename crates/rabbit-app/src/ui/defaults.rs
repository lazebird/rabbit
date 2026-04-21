//! Default values for UI components
//!
//! These defaults load from the saved configuration file if available,
//! otherwise fall back to the configuration model defaults in rabbit-models.
//! This ensures UI displays the saved values after restart.

use rabbit_models::config::*;
use rabbit_platform::config::load_config;

/// Get default ping target address
pub fn ping_target() -> String {
    // Try to load from saved config first
    if let Ok(config) = load_config() {
        return config.modules.ping.target;
    }
    // Fall back to model default
    PingConfig::default().target
}

/// Get default ping options string
pub fn ping_options() -> String {
    // Try to load from saved config first
    if let Ok(config) = load_config() {
        return config.modules.ping.opts_string();
    }
    // Fall back to model default
    PingConfig::default().opts_string()
}

/// Get default scan start IP
pub fn scan_start_ip() -> String {
    if let Ok(config) = load_config() {
        return config.modules.scan.start_ip;
    }
    ScanConfig::default().start_ip
}

/// Get default scan end IP
pub fn scan_end_ip() -> String {
    if let Ok(config) = load_config() {
        return config.modules.scan.end_ip;
    }
    ScanConfig::default().end_ip
}

/// Get default scan options string
pub fn scan_options() -> String {
    if let Ok(config) = load_config() {
        return config.modules.scan.opts_string();
    }
    ScanConfig::default().opts_string()
}

/// Get default HTTP port
pub fn http_port() -> u16 {
    if let Ok(config) = load_config() {
        return config.modules.http.port;
    }
    HttpConfig::default().port
}

/// Get default HTTP options string
pub fn http_options() -> String {
    if let Ok(config) = load_config() {
        return config.modules.http.opts_string();
    }
    HttpConfig::default().opts_string()
}

/// Get default TFTP server options string
pub fn tftpd_options() -> String {
    if let Ok(config) = load_config() {
        return config.modules.tftpd.opts_string();
    }
    TftpdConfig::default().opts_string()
}

/// Get default TFTP client server address
pub fn tftpc_server() -> String {
    if let Ok(config) = load_config() {
        return config.modules.tftpc.server_addr;
    }
    TftpcConfig::default().server_addr
}

/// Get default TFTP client options string
pub fn tftpc_options() -> String {
    if let Ok(config) = load_config() {
        return config.modules.tftpc.opts_string();
    }
    TftpcConfig::default().opts_string()
}

/// Get default chat username
pub fn chat_username() -> String {
    if let Ok(config) = load_config() {
        return config.modules.chat.username;
    }
    ChatModuleConfig::default().username
}

/// Get default chat port
pub fn chat_port() -> u16 {
    if let Ok(config) = load_config() {
        return config.modules.chat.port;
    }
    ChatModuleConfig::default().port
}

/// Get default chat broadcast address
pub fn chat_broadcast() -> String {
    if let Ok(config) = load_config() {
        return config.modules.chat.broadcast_addr;
    }
    ChatModuleConfig::default().broadcast_addr
}

/// Get default plan date
pub fn plan_date() -> String {
    if let Ok(config) = load_config() {
        return config.modules.plan.date;
    }
    PlanConfig::default().date
}

/// Get default plan time
pub fn plan_time() -> String {
    if let Ok(config) = load_config() {
        return config.modules.plan.time;
    }
    PlanConfig::default().time
}

/// Get default plan cycle
pub fn plan_cycle() -> String {
    if let Ok(config) = load_config() {
        return config.modules.plan.cycle.to_string();
    }
    PlanConfig::default().cycle.to_string()
}

/// Get default plan unit
pub fn plan_unit() -> i32 {
    if let Ok(config) = load_config() {
        return match config.modules.plan.unit.as_str() {
            "minutes" | "minute" => 0,
            "hours" | "hour" => 1,
            "days" | "day" => 2,
            _ => 0,
        };
    }
    0
}

/// Get default plan options
pub fn plan_options() -> String {
    if let Ok(config) = load_config() {
        return format!("override={}", config.modules.plan.override_conflicts);
    }
    format!("override={}", PlanConfig::default().override_conflicts)
}
