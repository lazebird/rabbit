//! Rabbit Data Models
//!
//! This crate contains all data structures used across the Rabbit application.
//! It has no external dependencies on platform-specific code.

use serde::{Deserialize, Serialize};

pub mod config;
pub mod network;
pub mod scan;

pub use config::*;
pub use network::*;
pub use scan::*;

/// 统一数据通道消息类型 - 所有模块使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiData {
    // ═══ 通用日志 ═══
    Log(Module, String),

    // ═══ Ping 专用 ═══
    PingStats(String),
    PingState { address: String, progress: u32, total: u32, color: String },

    // ═══ Scan 专用 ═══
    ScanProgress(String),

    // ═══ Plan 专用 ═══
    PlanReminder(String),

    // ═══ Chat 专用 ═══
    ChatMessage(String, String),
    ChatUserList(Vec<String>), // 改为数组，避免逗号分隔问题

    // ═══ 服务状态更新（核心：业务状态通知）
    // 第三个参数：None = 普通停止，Some(reason) = 带原因
    ServiceStatus(Module, bool, Option<String>),
}

/// 模块索引枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Module {
    Ping,
    Http,
    Tftpd,
    Tftpc,
    Scan,
    Chat,
    Plan,
}
