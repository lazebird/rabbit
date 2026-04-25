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

#![allow(dead_code)]

pub mod ping;
pub mod http;
pub mod tftpd;
pub mod tftpc;
pub mod plan;
pub mod chat;
pub mod scan;
pub mod ui_channel;

pub use ping::PingService;
pub use http::HttpService;
pub use tftpd::TftpdService;
pub use tftpc::TftpcService;
pub use plan::PlanService;
pub use chat::ChatService;
pub use scan::ScanService;

pub use ui_channel::{UiData, Module};

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
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Started(_) | Self::NoChange)
    }

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
    
    #[error("Platform error: {0}")]
    Platform(#[from] rabbit_platform::PlatformError),
    
    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ServiceError>;
