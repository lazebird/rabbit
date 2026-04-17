//! Settings Tab UI Component
//!
//! Layout:
//! - All content left-aligned
//! - Auto-save on change (no Save button)

use fltk::{
    button::CheckButton,
    frame::Frame,
    group::Flex,
    menu::Choice,
    prelude::*,
    enums::Align,
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

        // Row 1: Language
        let mut lang_row = Flex::default().row();
        lang_row.set_spacing(10);

        let mut lang_label = Frame::default().with_label("Language");
        lang_label.set_align(Align::Left | Align::Inside);
        lang_row.fixed(&lang_label, 70);

        let mut lang_choice = Choice::default();
        lang_choice.add_choice("English");
        lang_choice.add_choice("中文");
        lang_choice.add_choice("System");
        lang_choice.set_value(0);
        lang_row.fixed(&lang_choice, 100);

        Frame::default(); // Spacer
        lang_row.end();
        grp.fixed(&lang_row, spacing.row_height);

        // Row 2: Checkboxes (left-aligned, individual)
        let mut check_row = Flex::default().row();
        check_row.set_spacing(15);

        let mut tray_check = CheckButton::default().with_label("Tray");
        check_row.fixed(&tray_check, 55);

        let mut top_check = CheckButton::default().with_label("Top");
        check_row.fixed(&top_check, 50);

        let mut autostart_check = CheckButton::default().with_label("AutoStart");
        check_row.fixed(&autostart_check, 80);

        let mut autoupdate_check = CheckButton::default().with_label("AutoUpdate");
        autoupdate_check.set_checked(true);
        check_row.fixed(&autoupdate_check, 90);

        Frame::default(); // Spacer
        check_row.end();
        grp.fixed(&check_row, spacing.row_height);

        // Row 3: Version info (left-aligned)
        let mut version_row = Flex::default().row();
        let mut version_frame = Frame::default().with_label("Version: 1.0.0");
        version_frame.set_align(Align::Left | Align::Inside);
        Frame::default();
        version_row.end();
        grp.fixed(&version_row, spacing.row_height);

        // Row 4: Status (left-aligned)
        let mut status_row = Flex::default().row();
        let mut status_frame = Frame::default().with_label("Version is up to date.");
        status_frame.set_align(Align::Left | Align::Inside);
        Frame::default();
        status_row.end();
        grp.fixed(&status_row, spacing.row_height);

        // Spacer
        Frame::default();

        grp.end();

        // Apply styling
        grp.set_color(colors.background);

        // Auto-save callbacks - save immediately on change
        lang_choice.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        tray_check.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        top_check.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        autostart_check.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        autoupdate_check.set_callback(move |_| {
            send_event(UiEvent::SettingsSave);
        });

        grp
    }
}
