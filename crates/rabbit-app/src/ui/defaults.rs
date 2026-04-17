//! Default values for UI components
//!
//! These defaults should match the configuration model defaults in rabbit-models.
//! This ensures UI displays the same values as the configuration system.

use rabbit_models::config::*;

/// Get default ping target address
pub fn ping_target() -> String {
    PingConfig::default().target
}

/// Get default ping options string
pub fn ping_options() -> String {
    PingConfig::default().opts_string()
}

/// Get default scan start IP
pub fn scan_start_ip() -> String {
    ScanConfig::default().start_ip
}

/// Get default scan end IP
pub fn scan_end_ip() -> String {
    ScanConfig::default().end_ip
}

/// Get default scan options string
pub fn scan_options() -> String {
    ScanConfig::default().opts_string()
}

/// Get default HTTP port
pub fn http_port() -> u16 {
    HttpConfig::default().port
}

/// Get default HTTP options string
pub fn http_options() -> String {
    HttpConfig::default().opts_string()
}

/// Get default TFTP server options string
pub fn tftpd_options() -> String {
    TftpdConfig::default().opts_string()
}

/// Get default TFTP client server address
pub fn tftpc_server() -> String {
    TftpcConfig::default().server_addr
}

/// Get default TFTP client options string
pub fn tftpc_options() -> String {
    TftpcConfig::default().opts_string()
}

/// Get default chat username
pub fn chat_username() -> String {
    ChatModuleConfig::default().username
}

/// Get default chat port
pub fn chat_port() -> u16 {
    ChatModuleConfig::default().port
}

/// Get default chat broadcast address
pub fn chat_broadcast() -> String {
    ChatModuleConfig::default().broadcast_addr
}
