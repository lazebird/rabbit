//! HTTP Server Tab UI Component
//!
//! Layout matching old version:
//! - Single row: Port [input] Opt. [long input] [shell checkbox] [Start/Stop button]
//! - Directory list (fills most space)
//! - Access log at bottom

use fltk::{
    button::{Button, CheckButton},
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
    browser::Browser,
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, defaults};

/// HTTP Tab Component
pub struct HttpTab;

impl TabComponent for HttpTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "HTTPD").column();
        grp.set_margin(5);
        grp.set_spacing(4);

        // Control row - matching old version layout
        // Port [input] Opt. [long input] [shell checkbox] [Start button]
        let mut ctrl_row = Flex::default().row();
        ctrl_row.set_spacing(5);

        // Port label (fixed width)
        let _port_label = Frame::default().with_label("Port");
        ctrl_row.fixed(&_port_label, 30);

        // Port input (small width)
        let mut port_input = IntInput::default();
        port_input.set_value(&defaults::http_port().to_string());
        ctrl_row.fixed(&port_input, 50);

        // Opt. label (fixed width)
        let _opt_label = Frame::default().with_label("Opt.");
        ctrl_row.fixed(&_opt_label, 30);

        // Options input (takes remaining space)
        let mut opt_input = Input::default();
        opt_input.set_value(&defaults::http_options());

        // Shell checkbox (fixed width)
        let shell_check = CheckButton::default().with_label("shell");
        ctrl_row.fixed(&shell_check, 60);

        // Start/Stop button (fixed width, right aligned)
        let mut toggle_btn = Button::default().with_label("Start");
        toggle_btn.set_color(colors.accent);
        toggle_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&toggle_btn, 70);

        ctrl_row.end();
        grp.fixed(&ctrl_row, 28);

        // Directory management buttons row
        // [+] [-] [Browse] directory list display
        let mut dir_ctrl_row = Flex::default().row();
        dir_ctrl_row.set_spacing(5);
        
        // Add directory button
        let mut add_dir_btn = Button::default().with_label("+");
        add_dir_btn.set_color(colors.accent);
        add_dir_btn.set_label_color(fltk::enums::Color::White);
        dir_ctrl_row.fixed(&add_dir_btn, 28);
        
        // Remove directory button
        let mut remove_dir_btn = Button::default().with_label("-");
        remove_dir_btn.set_color(fltk::enums::Color::from_hex(0xE57373));
        remove_dir_btn.set_label_color(fltk::enums::Color::White);
        dir_ctrl_row.fixed(&remove_dir_btn, 28);
        
        // Browse button
        let mut browse_btn = Button::default().with_label("Browse");
        dir_ctrl_row.fixed(&browse_btn, 60);
        
        dir_ctrl_row.end();
        grp.fixed(&dir_ctrl_row, 28);

        // Directory list area - fills most space
        let mut dir_browser = Browser::default();
        dir_browser.set_text_size(14);
        
        // Access log at bottom (smaller area)
        let mut log_display = TextDisplay::default();
        let log_buf = TextBuffer::default();
        log_display.set_buffer(Some(log_buf));
        log_display.wrap_mode(WrapMode::AtBounds, 0);

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        dir_browser.set_color(colors.input_bg);
        log_display.set_color(colors.input_bg);
        log_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_dirs(&mut dir_browser);
        Self::refresh_log(&mut log_display);

        // Clone inputs for callback
        let port_input_clone = port_input.clone();
        let opt_input_clone = opt_input.clone();
        let shell_check_clone = shell_check.clone();
        let toggle_btn_clone = toggle_btn.clone();
        let mut log_display_clone = log_display.clone();
        let mut dir_browser_add = dir_browser.clone();
        let mut dir_browser_remove = dir_browser.clone();

        // Add directory button callback
        add_dir_btn.set_callback(move |_| {
            use fltk::dialog::NativeFileChooser;
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
            dialog.set_title("Select HTTP Directory");
            dialog.show();
            if let Some(path) = dialog.filename().to_str() {
                if !path.is_empty() {
                    crate::ui_state::add_http_dir(path);
                    crate::ui_state::append_http_log(&format!("Added directory: {}\r\n", path));
                    Self::refresh_dirs(&mut dir_browser_add);
                }
            }
        });
        
        // Remove directory button callback
        remove_dir_btn.set_callback(move |_| {
            let selected_idx = dir_browser_remove.value();
            if selected_idx <= 0 {
                return;
            }
            
            if let Some(text) = dir_browser_remove.text(selected_idx) {
                let remove_path = text.trim().to_string();
                if !remove_path.is_empty() {
                    crate::ui_state::remove_http_dir(&remove_path);
                    crate::ui_state::append_http_log(&format!("Removed directory: {}\r\n", remove_path));
                    Self::refresh_dirs(&mut dir_browser_remove);
                }
            }
        });
        
        // Browse button callback - open root directory in file manager
        browse_btn.set_callback(|_| {
            // TODO: Open directory in file manager
            crate::ui_state::append_http_log("Browse: Open root directory in file manager\r\n");
        });

        // Toggle button callback - send event, let refresh loop update button based on actual state
        toggle_btn.set_callback(move |_| {
            let label = toggle_btn_clone.label();
            let port = port_input_clone.value().parse::<u16>().unwrap_or(8000);
            let options = opt_input_clone.value();
            let shell = shell_check_clone.is_checked();

            if label == "Start" {
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.http_log = format!("Starting HTTP server on port {}...\r\n", port);
                        s.updated.insert("http_log".to_string(), true);
                    }
                }
                Self::refresh_log(&mut log_display_clone);
                send_event(UiEvent::HttpToggle { port, options, shell });
            } else {
                send_event(UiEvent::HttpToggle { port, options, shell });
            }
        });

        // Register display with centralized refresh manager
        super::ui_refresh::register_display("http_log", log_display.clone());
        super::ui_refresh::register_http_button(toggle_btn.clone(), colors.accent);

        grp
    }
}

impl HttpTab {
    fn refresh_dirs(browser: &mut Browser) {
        if let Some(state) = UiState::global() {
            if let Ok(s) = state.lock() {
                browser.clear();
                for dir in &s.http_dirs {
                    browser.add(dir);
                }
                if s.http_dirs.is_empty() {
                    browser.add("(No directories configured)");
                }
            }
        }
    }
    
    fn refresh_log(display: &mut TextDisplay) {
        if let Some(state) = UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.http_log);
                    let lines = buf.count_lines(0, buf.length());
                    display.scroll(lines, 0);
                }
            }
        }
    }
}
