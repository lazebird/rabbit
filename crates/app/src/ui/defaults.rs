use schema::config::keys;

/// Default ping target address (used when config has none).
pub fn ping_target() -> String {
    rabbit_config::get_string("ping", keys::ping::TARGET).unwrap_or_else(|| "1.1.1.1".to_string())
}

/// Ping option string for UI display.
pub fn ping_options() -> String {
    let interval = rabbit_config::get_integer("ping", keys::ping::INTERVAL).unwrap_or(1000);
    let count = rabbit_config::get_integer("ping", keys::ping::COUNT).unwrap_or(-1);
    let stoponloss = rabbit_config::get_bool("ping", keys::ping::STOP_ON_LOSS).unwrap_or(false);
    format!("interval={};count={};stoponloss={}", interval, count, stoponloss)
}

/// Default scan start IP (used when config has none).
pub fn scan_start_ip() -> String {
    rabbit_config::get_string("scan", keys::scan::START_IP).unwrap_or_else(|| "192.168.1.1".to_string())
}

/// Default scan end IP (used when config has none).
pub fn scan_end_ip() -> String {
    rabbit_config::get_string("scan", keys::scan::END_IP).unwrap_or_else(|| "254".to_string())
}

/// Scan option string for UI display.
pub fn scan_options() -> String {
    let filter = rabbit_config::get_bool("scan", keys::scan::FILTER).unwrap_or(true);
    format!("filter={}", filter)
}

/// Default HTTP port (used when config has none).
pub fn http_port() -> u16 {
    rabbit_config::get_integer("http", keys::http::PORT).unwrap_or(8000) as u16
}

/// HTTP option string for UI display.
pub fn http_options() -> String {
    let autoindex = rabbit_config::get_bool("http", keys::http::AUTO_INDEX).unwrap_or(true);
    let videoplay = rabbit_config::get_bool("http", keys::http::VIDEO_PLAY).unwrap_or(true);
    format!("autoindex={};videoplay={};", autoindex, videoplay)
}

/// TFTP server option string for UI display.
pub fn tftpd_options() -> String {
    let timeout = rabbit_config::get_integer("tftpd", keys::tftpd::TIMEOUT).unwrap_or(200);
    let retry = rabbit_config::get_integer("tftpd", keys::tftpd::MAX_RETRY).unwrap_or(10);
    let blksize = rabbit_config::get_integer("tftpd", keys::tftpd::BLK_SIZE).unwrap_or(512);
    let override_opt = rabbit_config::get_bool("tftpd", keys::tftpd::OVERRIDE).unwrap_or(false);
    let qsize = rabbit_config::get_integer("tftpd", keys::tftpd::QSIZE).unwrap_or(2000);
    let qtout = rabbit_config::get_integer("tftpd", keys::tftpd::QTOUT).unwrap_or(1000);
    let fslog = rabbit_config::get_bool("tftpd", keys::tftpd::FSLOG).unwrap_or(false);
    format!(
        "timeout={};retry={};blksize={};override={};qsize={};qtout={};fslog={};",
        timeout, retry, blksize, override_opt, qsize, qtout, fslog
    )
}

/// Default TFTP client server address (used when config has none).
pub fn tftpc_server() -> String {
    rabbit_config::get_string("tftpc", keys::tftpc::SERVER_ADDR).unwrap_or_else(|| "127.0.0.1".to_string())
}

/// TFTP client option string for UI display.
pub fn tftpc_options() -> String {
    let timeout = rabbit_config::get_integer("tftpc", keys::tftpc::TIMEOUT).unwrap_or(200);
    let retry = rabbit_config::get_integer("tftpc", keys::tftpc::MAX_RETRY).unwrap_or(10);
    let blksize = rabbit_config::get_integer("tftpc", keys::tftpc::BLK_SIZE).unwrap_or(1024);
    format!("timeout={};retry={};blksize={};", timeout, retry, blksize)
}

/// Default chat username (used when config has none).
pub fn chat_username() -> String {
    rabbit_config::get_string("chat", keys::chat::USERNAME).unwrap_or_else(|| "User@PC".to_string())
}

/// Default chat port (used when config has none).
pub fn chat_port() -> u16 {
    rabbit_config::get_integer("chat", keys::chat::PORT).unwrap_or(1314) as u16
}

/// Default chat broadcast address (used when config has none).
pub fn chat_broadcast() -> String {
    rabbit_config::get_string("chat", keys::chat::BROADCAST_ADDR)
        .unwrap_or_else(|| "255.255.255.255".to_string())
}

/// Plan option string for UI display.
pub fn plan_options() -> String {
    let override_opt = rabbit_config::get_bool("plan", keys::plan::OVERRIDE).unwrap_or(false);
    format!("override={}", override_opt)
}
