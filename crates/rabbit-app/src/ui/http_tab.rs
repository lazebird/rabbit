//! HTTP Server Tab UI Component
//!
//! Layout matching old version:
//! - Single row: Port [input] Opt. [long input] [shell checkbox] [Start/Stop button]
//! - File/Directory list (fills most space)
//! - Access log at bottom

use fltk::{
    browser::{Browser, BrowserType},
    button::{Button, CheckButton},
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
};

use super::{defaults, Colors, TabComponent};
use crate::ui_events::{send_event, UiEvent};
use crate::ui_state::UiState;

/// HTTP Tab Component
pub struct HttpTab;

impl TabComponent for HttpTab {
    fn build(x: i32, y: i32, w: i32, h: i32, config: &rabbit_models::AppConfig) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "HTTPD").column();
        grp.set_margin(0); // Remove margin to match old version
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

        // Shell checkbox (fixed width) - load from config
        let shell_checked = rabbit_platform::config::load_config().map(|c| c.modules.get_bool("http", "shell").unwrap_or(false)).unwrap_or(false);
        let shell_check = CheckButton::default().with_label("shell");
        shell_check.set_checked(shell_checked);
        ctrl_row.fixed(&shell_check, 60);

        // Start/Stop button (fixed width, right aligned)
        let is_running = config.modules.get_bool("http", "running").unwrap_or(false);
        let mut toggle_btn = Button::default().with_label(if is_running { "Stop" } else { "Start" });
        toggle_btn.set_color(if is_running { fltk::enums::Color::from_hex(super::ui_refresh::HTTP_STOP_COLOR) } else { colors.accent });
        toggle_btn.set_label_color(fltk::enums::Color::White);
        ctrl_row.fixed(&toggle_btn, 70);

        ctrl_row.end();
        grp.fixed(&ctrl_row, 28);

        // File/Directory management buttons row
        // [+] [-] [Explore] file/directory list display
        let mut item_ctrl_row = Flex::default().row();
        item_ctrl_row.set_spacing(5);

        // Add file button
        let mut add_file_btn = Button::default().with_label("+F");
        add_file_btn.set_color(colors.accent);
        add_file_btn.set_label_color(fltk::enums::Color::White);
        add_file_btn.set_tooltip("Add file");
        item_ctrl_row.fixed(&add_file_btn, 28);

        // Add directory button
        let mut add_dir_btn = Button::default().with_label("+D");
        add_dir_btn.set_color(colors.accent);
        add_dir_btn.set_label_color(fltk::enums::Color::White);
        add_dir_btn.set_tooltip("Add directory");
        item_ctrl_row.fixed(&add_dir_btn, 28);

        // Remove button
        let mut remove_btn = Button::default().with_label("-");
        remove_btn.set_color(fltk::enums::Color::from_hex(0xE57373));
        remove_btn.set_label_color(fltk::enums::Color::White);
        item_ctrl_row.fixed(&remove_btn, 28);

        // Explore button - open selected item's parent directory
        let mut explore_btn = Button::default().with_label("  \u{1F4C2}");
        explore_btn.set_label_size(16);
        explore_btn.set_align(fltk::enums::Align::Center);
        explore_btn.set_tooltip("Open containing folder");
        item_ctrl_row.fixed(&explore_btn, 32);

        item_ctrl_row.end();
        grp.fixed(&item_ctrl_row, 28);

        // File/Directory list area - fills most space, using HoldBrowser for clear selection
        let mut item_browser = Browser::default();
        item_browser.set_type(BrowserType::Hold);
        item_browser.set_text_size(14);

        // Access log at bottom (smaller area)
        let mut log_display = TextDisplay::default();
        let log_buf = TextBuffer::default();
        log_display.set_buffer(Some(log_buf));
        log_display.wrap_mode(WrapMode::AtBounds, 0);
        log_display.set_frame(fltk::enums::FrameType::FlatBox); // Remove border

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        item_browser.set_color(colors.input_bg);
        log_display.set_color(colors.input_bg);
        log_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_items(&mut item_browser);
        Self::refresh_log(&mut log_display);

        // Clone inputs for callback
        let port_input_clone = port_input.clone();
        let opt_input_clone = opt_input.clone();
        let shell_check_clone = shell_check.clone();
        let toggle_btn_clone = toggle_btn.clone();
        let mut log_display_clone = log_display.clone();
        let mut item_browser_add = item_browser.clone();
        let mut item_browser_add_dir = item_browser.clone();
        let mut item_browser_remove = item_browser.clone();
        let item_browser_explore = item_browser.clone();

        // Add file button callback
        add_file_btn.set_callback(move |_| {
            use fltk::dialog::NativeFileChooser;
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseFile);
            dialog.set_title("Select HTTP File");
            dialog.show();
            if let Some(path) = dialog.filename().to_str() {
                if !path.is_empty() {
                    crate::ui_state::add_http_item(path);
                    crate::ui_state::append_http_log(&format!("Added file: {}\r\n", path));
                    Self::refresh_items(&mut item_browser_add);
                    crate::ui_state::sync_http_config();
                }
            }
        });

        // Add directory button callback
        add_dir_btn.set_callback(move |_| {
            use fltk::dialog::NativeFileChooser;
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseDir);
            dialog.set_title("Select HTTP Directory");
            dialog.show();
            if let Some(path) = dialog.filename().to_str() {
                if !path.is_empty() {
                    crate::ui_state::add_http_item(path);
                    crate::ui_state::append_http_log(&format!("Added directory: {}\r\n", path));
                    Self::refresh_items(&mut item_browser_add_dir);
                    crate::ui_state::sync_http_config();
                }
            }
        });

        // Remove button callback
        remove_btn.set_callback(move |_| {
            let selected_idx = item_browser_remove.value();
            if selected_idx <= 0 {
                return;
            }

            if let Some(text) = item_browser_remove.text(selected_idx) {
                // Strip the selection indicator if present
                let item_path = text.trim_start_matches("▶ ").trim().to_string();
                if !item_path.is_empty() {
                    crate::ui_state::remove_http_item(&item_path);
                    crate::ui_state::append_http_log(&format!("Removed: {}\r\n", item_path));
                    Self::refresh_items(&mut item_browser_remove);
                    crate::ui_state::sync_http_config();
                }
            }
        });

        // Explore button callback - open containing folder of selected item
        explore_btn.set_callback(move |_| {
            let selected_idx = item_browser_explore.value();
            if selected_idx <= 0 {
                crate::ui_state::append_http_log("No item selected.\r\n");
                return;
            }

            if let Some(text) = item_browser_explore.text(selected_idx) {
                let item_path = text.trim_start_matches("▶ ").trim().to_string();
                if item_path.is_empty() {
                    return;
                }

                let path = std::path::Path::new(&item_path);
                let parent = if path.is_dir() {
                    path.to_path_buf()
                } else {
                    path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf()
                };

                let parent_str = parent.to_string_lossy().to_string();
                match rabbit_platform::dialog::open_file_manager(&parent_str) {
                    Ok(_) => {
                        crate::ui_state::append_http_log(&format!("Opened: {}\r\n", parent_str));
                    }
                    Err(e) => {
                        crate::ui_state::append_http_log(&format!("Failed to open folder: {}\r\n", e));
                    }
                }
            }
        });

        // Toggle button callback - send event, let refresh loop update button based on actual state
        toggle_btn.set_callback(move |_| {
            let label = toggle_btn_clone.label();
            let port = port_input_clone.value().parse::<u16>().unwrap_or(8000);
            let options = opt_input_clone.value();
            let shell = shell_check_clone.is_checked();

            // Parse options string (format: "autoindex=true;videoplay=true;")
            let autoindex = options.contains("autoindex=true");
            let videoplay = options.contains("videoplay=true");

            // Always sync config on button click
            crate::ui_state::sync_http_start_config(port, shell, autoindex, videoplay);

            if label == "Start" {
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.http_log = format!("Starting HTTP server on port {}...\r\n", port);
                        s.updated.insert("http_log".to_string(), true);
                    }
                }
                Self::refresh_log(&mut log_display_clone);
            }

            send_event(UiEvent::ModuleToggle { module: "http".into() });
        });

        // Register display with centralized refresh manager
        super::ui_refresh::register_display("http_log", log_display.clone());
        super::ui_refresh::register_http_button(toggle_btn.clone(), colors.accent);
        super::ui_refresh::register_http_browser(item_browser.clone());

        grp
    }
}

impl HttpTab {
    fn refresh_items(browser: &mut Browser) {
        if let Some(state) = UiState::global() {
            if let Ok(s) = state.lock() {
                browser.clear();
                let selected_idx = s.http_selected_idx.unwrap_or(0);
                for (i, item) in s.http_items.iter().enumerate() {
                    let line_num = (i + 1) as i32;
                    // Add visual indicator for selected item
                    if line_num == selected_idx {
                        browser.add(&format!("▶ {}", item));
                    } else {
                        browser.add(item);
                    }
                }
                if s.http_items.is_empty() {
                    browser.add("(No files or directories configured)");
                } else {
                    // Restore selection
                    if selected_idx > 0 {
                        browser.select(selected_idx);
                    }
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
