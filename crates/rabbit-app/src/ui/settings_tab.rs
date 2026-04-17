//! Settings Tab UI Component
//!
//! Layout based on old version screenshot:
//! - Language dropdown
//! - Tray | Top | AutoStart | AutoUpdate checkboxes
//! - Home | Profile | Help links
//! - Version info
//! - Status message area

use fltk::{
    button::{Button, CheckButton},
    frame::Frame,
    group::Flex,
    menu::Choice,
    prelude::*,
};

use crate::ui_events::{UiEvent, send_event};
use super::{TabComponent, Colors, Spacing};

/// Settings Tab Component
pub struct SettingsTab;

impl TabComponent for SettingsTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "Setting").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Row 1: Language and checkboxes
        let mut row1 = Flex::default().row();
        row1.set_spacing(15);

        // Language
        let _lang_label = Frame::default().with_label("Language");

        let mut lang_choice = Choice::default();
        lang_choice.add_choice("English");
        lang_choice.add_choice("中文");
        lang_choice.add_choice("System");
        lang_choice.set_value(0);

        // Checkboxes
        let _tray_check = CheckButton::default().with_label("Tray");

        let _top_check = CheckButton::default().with_label("Top");

        let _autostart_check = CheckButton::default().with_label("AutoStart");

        let autoupdate_check = CheckButton::default().with_label("AutoUpdate");
        autoupdate_check.set_checked(true);

        // Links
        Frame::default(); // Spacer

        let _home_label = Frame::default().with_label("Home");

        let _profile_label = Frame::default().with_label("Profile");

        let _help_label = Frame::default().with_label("Help");

        row1.end();
        grp.fixed(&row1, spacing.row_height);

        // Version info
        let version_frame = Frame::default().with_label("Version: 1.0.0");
        grp.fixed(&version_frame, spacing.row_height);

        // Status area
        let status_frame = Frame::default().with_label("Version is up to date!");
        grp.fixed(&status_frame, spacing.row_height);

        // Save button row
        let mut save_row = Flex::default().row();
        save_row.set_spacing(spacing.padding);

        Frame::default(); // Spacer to center button

        let mut save_btn = Button::default().with_label("Save Settings");
        save_btn.set_color(colors.accent);
        save_btn.set_label_color(fltk::enums::Color::White);

        Frame::default(); // Spacer
        save_row.end();
        grp.fixed(&save_row, spacing.row_height + 10);

        // Spacer for remaining area
        Frame::default();

        grp.end();

        // Apply styling
        grp.set_color(colors.background);

        // Add button callback
        save_btn.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
            fltk::dialog::message_default("Settings saved successfully!");
        });

        grp
    }
}
