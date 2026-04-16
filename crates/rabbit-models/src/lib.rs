//! Rabbit Data Models
//!
//! This crate contains all data structures used across the Rabbit application.
//! It has no external dependencies on platform-specific code.

pub mod config;
pub mod ping;
pub mod http;
pub mod tftp;
pub mod plan;
pub mod chat;
pub mod scan;

pub use config::*;
pub use ping::*;
pub use http::*;
pub use tftp::*;
pub use plan::*;
pub use chat::*;
pub use scan::*;
