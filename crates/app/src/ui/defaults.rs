use adapter::config::load_config;

pub fn ping_target() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_string("ping", "target") {
            return v;
        }
    }
    "1.1.1.1".to_string()
}

pub fn ping_options() -> String {
    let interval = if let Ok(c) = load_config() {
        c.modules.get_integer("ping", "interval").unwrap_or(1000)
    } else {
        1000
    };
    let count = if let Ok(c) = load_config() { c.modules.get_integer("ping", "count").unwrap_or(-1) } else { -1 };
    let stoponloss = if let Ok(c) = load_config() {
        c.modules.get_bool("ping", "stoponloss").unwrap_or(false)
    } else {
        false
    };
    format!("interval={};count={};stoponloss={}", interval, count, stoponloss)
}

pub fn scan_start_ip() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_string("scan", "start_ip") {
            return v;
        }
    }
    "192.168.1.1".to_string()
}

pub fn scan_end_ip() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_string("scan", "end_ip") {
            return v;
        }
    }
    "254".to_string()
}

pub fn scan_options() -> String {
    let filter = if let Ok(c) = load_config() { c.modules.get_bool("scan", "filter").unwrap_or(true) } else { true };
    format!("filter={}", filter)
}

pub fn http_port() -> u16 {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_integer("http", "port") {
            return v as u16;
        }
    }
    8000
}

pub fn http_options() -> String {
    let autoindex = if let Ok(c) = load_config() {
        c.modules.get_bool("http", "autoindex").unwrap_or(true)
    } else {
        true
    };
    let videoplay = if let Ok(c) = load_config() {
        c.modules.get_bool("http", "videoplay").unwrap_or(true)
    } else {
        true
    };
    format!("autoindex={};videoplay={};", autoindex, videoplay)
}

pub fn tftpd_options() -> String {
    let timeout = if let Ok(c) = load_config() {
        c.modules.get_integer("tftpd", "timeout").unwrap_or(200)
    } else {
        200
    };
    let retry = if let Ok(c) = load_config() {
        c.modules.get_integer("tftpd", "maxretry").unwrap_or(10)
    } else {
        10
    };
    let blksize = if let Ok(c) = load_config() {
        c.modules.get_integer("tftpd", "blksize").unwrap_or(512)
    } else {
        512
    };
    let override_conflicts = if let Ok(c) = load_config() {
        c.modules.get_bool("tftpd", "override_conflicts").unwrap_or(false)
    } else {
        false
    };
    let qsize = if let Ok(c) = load_config() {
        c.modules.get_integer("tftpd", "qsize").unwrap_or(2000)
    } else {
        2000
    };
    let qtout = if let Ok(c) = load_config() {
        c.modules.get_integer("tftpd", "qtout").unwrap_or(1000)
    } else {
        1000
    };
    let fslog = if let Ok(c) = load_config() {
        c.modules.get_bool("tftpd", "fslog").unwrap_or(false)
    } else {
        false
    };
    format!(
        "timeout={};retry={};blksize={};override={};qsize={};qtout={};fslog={};",
        timeout, retry, blksize, override_conflicts, qsize, qtout, fslog
    )
}

pub fn tftpc_server() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_string("tftpc", "server_addr") {
            return v;
        }
    }
    "127.0.0.1".to_string()
}

pub fn tftpc_options() -> String {
    let timeout = if let Ok(c) = load_config() {
        c.modules.get_integer("tftpc", "timeout").unwrap_or(200)
    } else {
        200
    };
    let retry = if let Ok(c) = load_config() {
        c.modules.get_integer("tftpc", "maxretry").unwrap_or(10)
    } else {
        10
    };
    let blksize = if let Ok(c) = load_config() {
        c.modules.get_integer("tftpc", "blksize").unwrap_or(1024)
    } else {
        1024
    };
    format!("timeout={};retry={};blksize={};", timeout, retry, blksize)
}

pub fn chat_username() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_string("chat", "username") {
            return v;
        }
    }
    "User@PC".to_string()
}

pub fn chat_port() -> u16 {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_integer("chat", "port") {
            return v as u16;
        }
    }
    1314
}

pub fn chat_broadcast() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_string("chat", "broadcast_addr") {
            return v;
        }
    }
    "255.255.255.255".to_string()
}

pub fn plan_date() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_string("plan", "date") {
            return v;
        }
    }
    String::new()
}

pub fn plan_time() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_string("plan", "time") {
            return v;
        }
    }
    String::new()
}

pub fn plan_cycle() -> String {
    if let Ok(c) = load_config() {
        if let Some(v) = c.modules.get_integer("plan", "cycle") {
            return v.to_string();
        }
    }
    "0".to_string()
}

pub fn plan_unit() -> i32 {
    let unit = if let Ok(c) = load_config() {
        c.modules.get_string("plan", "unit").unwrap_or_else(|| "minute".to_string())
    } else {
        "minute".to_string()
    };
    match unit.as_str() {
        "minutes" | "minute" => 0,
        "hours" | "hour" => 1,
        "days" | "day" => 2,
        _ => 0,
    }
}

pub fn plan_options() -> String {
    let override_conflicts = if let Ok(c) = load_config() {
        c.modules.get_bool("plan", "override_conflicts").unwrap_or(false)
    } else {
        false
    };
    format!("override={}", override_conflicts)
}
