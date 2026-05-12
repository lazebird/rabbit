//! Rabbit Application
//!
//! Main application logic including UI integration.

pub mod app;
pub mod ui;
pub mod ui_events;
pub mod ui_state;
pub mod upgrade;
pub mod view_model;

pub use upgrade::handle_version_check_result;
pub use app::App;
pub use service::ui_channel::{Module, UiData};
pub use ui_events::{init_event_system, send_event, UiEvent};
pub use ui_state::*;
pub use view_model::*;
