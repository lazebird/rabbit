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

pub mod ping;
pub mod http;
pub mod tftpd;
pub mod tftpc;
pub mod plan;
pub mod chat;
pub mod scan;

pub use ping::PingService;
pub use http::HttpService;
pub use tftpd::TftpdService;
pub use tftpc::TftpcService;
pub use plan::PlanService;
pub use chat::ChatService;
pub use scan::ScanService;

pub type TftpService = TftpdService;

use thiserror::Error;

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
