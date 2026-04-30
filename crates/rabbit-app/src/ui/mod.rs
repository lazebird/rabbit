//! UI Components Module
//!
//! This module contains FLTK UI components organized by tab.
//! Each tab is implemented in its own file for better maintainability.

pub mod chat_tab;
pub mod defaults;
pub mod http_tab;
pub mod ping_tab;
pub mod plan_tab;
pub mod scan_tab;
pub mod settings_tab;
pub mod styles;
pub mod tftpc_tab;
pub mod tftpd_tab;
pub mod ui_refresh;

pub use chat_tab::ChatTab;
pub use defaults::*;
pub use http_tab::HttpTab;
pub use ping_tab::PingTab;
pub use plan_tab::PlanTab;
pub use scan_tab::ScanTab;
pub use settings_tab::check_version_update;
pub use settings_tab::SettingsTab;
pub use styles::{format_bytes, format_ping_result, format_ping_stats, Colors, Spacing};
pub use tftpc_tab::TftpcTab;
pub use tftpd_tab::TftpdTab;

use fltk::group::Flex;

/// Trait for tab components
pub trait TabComponent {
    /// Build the tab UI and return the container
    /// x, y, w, h are the position and size within the parent Tabs widget
    /// The label will be used as the tab title
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex;
}
