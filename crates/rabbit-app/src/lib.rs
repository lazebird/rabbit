//! Rabbit Application
//!
//! Main application logic including UI integration.

pub mod app;
pub mod ui;
pub mod view_model;

pub use app::App;
pub use view_model::*;

// Include Slint UI
slint::include_modules!();

// Re-export ComponentHandle for run() method
pub use slint::ComponentHandle;
