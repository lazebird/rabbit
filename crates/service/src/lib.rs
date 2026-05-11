//! Rabbit Business Logic Layer
//!
//! This crate contains all business services:
//! - PingService: ICMP ping functionality
//! - HttpService: HTTP server
//! - TftpdService: TFTP server
//! - TftpcService: TFTP client
//! - PlanService: Task planner
//! - ChatService: LAN chat
//! - ScanService: IP scanner

pub mod chat;
pub mod http;
pub mod ping;
pub mod plan;
pub mod scan;
pub mod tftpc;
pub mod tftpd;
pub mod ui_channel;

pub use chat::ChatService;
pub use http::HttpService;
pub use ping::PingService;
pub use plan::PlanService;
pub use scan::ScanService;
pub use tftpc::TftpcService;
pub use tftpd::TftpdService;

pub use ui_channel::{send_ui, Module, UiData};

pub type TftpService = TftpdService;

use thiserror::Error;

#[derive(Debug)]
pub enum ServiceUpdateResult {
    Started(String),
    Stopped(String),
    Error(String),
    NoChange,
}

impl ServiceUpdateResult {
    pub fn message(&self) -> &str {
        match self {
            Self::Started(m) => m,
            Self::Stopped(m) => m,
            Self::Error(m) => m,
            Self::NoChange => "",
        }
    }

    pub fn ok(&self) -> Option<String> {
        match self {
            Self::Error(_) => None,
            _ => Some(self.message().to_string()),
        }
    }
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service not started")]
    NotStarted,

    #[error("Service already running")]
    AlreadyRunning,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ServiceError>;
