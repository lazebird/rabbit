//! Common UI Styles and Constants
//!
//! Provides consistent styling across all tabs.

use fltk::enums::Color;

/// Color scheme - lighter theme similar to old version
#[derive(Clone)]
pub struct Colors {
    /// Window background - light gray
    pub background: Color,
    /// Text color - dark
    pub text: Color,
    /// Accent color - green for buttons
    pub accent: Color,
    /// Highlight color - green for active items
    pub highlight: Color,
    /// Input field background - white
    pub input_bg: Color,
    /// Border/frame color
    pub border: Color,
}

impl Colors {
    pub fn new() -> Self {
        Self {
            background: Color::from_hex(0xF5F5F5),
            text: Color::from_hex(0x333333),
            accent: Color::from_hex(0x4CAF50),
            highlight: Color::from_hex(0x90EE90),
            input_bg: Color::White,
            border: Color::from_hex(0xCCCCCC),
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self::new()
    }
}

/// Common spacing and sizing constants
pub struct Spacing {
    pub margin: i32,
    pub padding: i32,
    pub row_height: i32,
    pub button_width: i32,
}

impl Spacing {
    pub fn new() -> Self {
        Self {
            margin: 5,
            padding: 4,
            row_height: 28,
            button_width: 60,
        }
    }
}

impl Default for Spacing {
    fn default() -> Self {
        Self::new()
    }
}

/// Format ping statistics line
pub fn format_ping_stats(sent: u32, received: u32, min_ms: f64, max_ms: f64, avg_ms: f64) -> String {
    let now = chrono::Local::now();
    let lost = sent.saturating_sub(received);
    format!(
        "{} Tx {} Rx {} Loss {} Min {} Max {} Avg {:.6}",
        now.format("%Y/%m/%d %H:%M:%S"),
        sent, received, lost,
        if min_ms == f64::INFINITY { 0 } else { min_ms as u32 },
        max_ms as u32, avg_ms
    )
}

/// Format ping result line
pub fn format_ping_result(addr: &str, bytes: u32, duration_ms: f64, ttl: Option<u32>) -> String {
    let ttl_str = ttl.map(|t| format!(" TTL={}", t)).unwrap_or_default();
    if duration_ms > 0.0 {
        format!("来自 {} 的回复: 字节={} 毫秒={}{}", addr, bytes, duration_ms as u32, ttl_str)
    } else {
        format!("来自 {} 的回复: 请求超时", addr)
    }
}

/// Format bytes with proper unit
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    if bytes == 0 {
        return "0B".to_string();
    }
    let exp = (bytes as f64).log(1024.0).min(UNITS.len() as f64 - 1.0) as usize;
    let value = bytes as f64 / 1024f64.powi(exp as i32);
    format!("{:.0}{}", value, UNITS[exp])
}
