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
pub use ui_state::{UiState, append_ping_output, set_ping_stats, append_scan_output, append_http_log, append_tftpd_log, append_chat_message, set_settings_output, append_settings_output, update_settings_line};
