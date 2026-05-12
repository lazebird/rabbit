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

use async_trait::async_trait;
use thiserror::Error;

/// Common trait for all Rabbit business services.
///
/// Provides a uniform lifecycle and communication interface:
/// - [`update`](Service::update): toggle start/stop (sends status to UI)
/// - [`destroy`](Service::destroy): release resources at shutdown (no UI notifications)
/// - [`send`](Service::send): push a UI event through the service channel
///
/// All service types (PingService, HttpService, …) implement this trait,
/// enabling generic service management (batch destroy, collection iteration, …).
#[async_trait]
pub trait Service: Send + Sync {
    /// Toggle the service on/off.
    ///
    /// Sends [`UiData::ServiceStatus`] to the UI layer so the tab button
    /// reflects the new state.  Returns a [`ServiceUpdateResult`] describing
    /// what happened (Started / Stopped / Error / NoChange).
    async fn update(&mut self) -> ServiceUpdateResult;

    /// Release all service resources.
    ///
    /// Called during application shutdown.  Does **not** send any status
    /// notifications to the UI layer (the UI is being torn down anyway).
    async fn destroy(&mut self) -> Result<()>;

    /// Send a [`UiData`] message through the service's UI channel.
    async fn send(&self, data: UiData);
}

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
