//! Rabbit Application
//!
//! Main application logic including UI integration.

pub mod app;
pub mod ui;
pub mod view_model;
pub mod ui_events;
pub mod ui_state;
pub mod upgrade;

pub use app::App;
pub use app::handle_version_check_result;
pub use view_model::*;
pub use ui_events::{UiEvent, send_event, init_event_system};
pub use ui_state::*;
pub use rabbit_core::ui_channel::{UiData, Module};
