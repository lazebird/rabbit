//! Chat Tab UI Component
//!
//! Layout matching old version:
//! - Row 1: Name [input] Port [input] [Start button]
//! - Row 2: Broadcast [input] [Refresh] [Notify]
//! - User list (left, fixed width) | Messages (right, flexible)
//! - Input row at bottom

use fltk::{
    button::Button,
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    prelude::*,
    text::{TextBuffer, TextDisplay, WrapMode},
};

use super::{defaults, Colors, TabComponent};
use crate::ui_events::{send_event, UiEvent};
use crate::ui_state::UiState;

/// Chat Tab Component
pub struct ChatTab;

impl TabComponent for ChatTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();

        let mut grp = Flex::new(x, y, w, h, "CHAT").column();
        grp.set_margin(0); // Remove margin to match old version
        grp.set_spacing(4);

        // Row 1: Name [input] Port [input] [Start button]
        let mut row1 = Flex::default().row();
        row1.set_spacing(5);

        let _user_label = Frame::default().with_label("Name");
        row1.fixed(&_user_label, 40);

        let mut username_input = Input::default();
        username_input.set_value(&defaults::chat_username());
        row1.fixed(&username_input, 100);

        let _port_label = Frame::default().with_label("Port");
        row1.fixed(&_port_label, 35);

        let mut port_input = IntInput::default();
        port_input.set_value(&defaults::chat_port().to_string());
        row1.fixed(&port_input, 50);

        // Spacer
        Frame::default();

        let mut toggle_btn = Button::default().with_label("Start");
        toggle_btn.set_color(colors.accent);
        toggle_btn.set_label_color(fltk::enums::Color::White);
        row1.fixed(&toggle_btn, 70);

        row1.end();
        grp.fixed(&row1, 28);

        // Row 2: Broadcast [input] [Refresh] [Notify]
        let mut row2 = Flex::default().row();
        row2.set_spacing(5);

        let _bcast_label = Frame::default().with_label("Broadcast");
        row2.fixed(&_bcast_label, 60);

        let mut broadcast_input = Input::default();
        broadcast_input.set_value(&defaults::chat_broadcast());

        let mut refresh_btn = Button::default().with_label("Refresh");
        row2.fixed(&refresh_btn, 60);

        let mut notify_btn = Button::default().with_label("Notify");
        row2.fixed(&notify_btn, 60);

        row2.end();
        grp.fixed(&row2, 28);

        // Main content area: Users list (left) | Messages (right)
        let mut content_row = Flex::default().row();
        content_row.set_spacing(5);

        // Users column (fixed width)
        let mut users_col = Flex::default().column();
        let users_label = Frame::default().with_label("Users:");
        users_col.fixed(&users_label, 20);
        let mut users_display = TextDisplay::default();
        let users_buf = TextBuffer::default();
        users_display.set_buffer(Some(users_buf));
        users_display.wrap_mode(WrapMode::AtBounds, 0);
        users_display.set_frame(fltk::enums::FrameType::FlatBox); // Remove border
        users_col.end();
        content_row.fixed(&users_col, 120);

        // Messages column (flexible)
        let mut msg_col = Flex::default().column();
        let msg_label = Frame::default().with_label("Messages:");
        msg_col.fixed(&msg_label, 20);
        let mut messages_display = TextDisplay::default();
        let msg_buf = TextBuffer::default();
        messages_display.set_buffer(Some(msg_buf));
        messages_display.wrap_mode(WrapMode::AtBounds, 0);
        messages_display.set_frame(fltk::enums::FrameType::FlatBox); // Remove border
        msg_col.end();

        content_row.end();

        // Row 3: Input
        let mut row3 = Flex::default().row();
        row3.set_spacing(5);

        let _input_label = Frame::default().with_label("Input:");
        row3.fixed(&_input_label, 40);

        let msg_input = Input::default();

        let mut send_btn = Button::default().with_label("Send");
        send_btn.set_color(colors.accent);
        send_btn.set_label_color(fltk::enums::Color::White);
        row3.fixed(&send_btn, 70);

        row3.end();
        grp.fixed(&row3, 28);

        grp.end();

        // Apply styling
        grp.set_color(colors.background);
        users_display.set_color(colors.input_bg);
        users_display.set_text_color(colors.text);
        messages_display.set_color(colors.input_bg);
        messages_display.set_text_color(colors.text);

        // Set initial content from global state
        Self::refresh_users(&mut users_display);
        Self::refresh_messages(&mut messages_display);

        // Clone inputs for callbacks
        let username_input_clone = username_input.clone();
        let port_input_clone = port_input.clone();
        let _broadcast_input_clone = broadcast_input.clone();
        let mut toggle_btn_clone = toggle_btn.clone();

        let mut msg_input_clone = msg_input.clone();
        let username_for_send = username_input.clone();

        // Add button callbacks
        toggle_btn.set_callback(move |_| {
            let label = toggle_btn_clone.label();

            if label == "Start" {
                let username = username_input_clone.value();
                if username.is_empty() {
                    fltk::dialog::alert_default("Please enter a username!");
                    return;
                }
                let port = port_input_clone.value().parse::<u16>().unwrap_or(1314);

                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.chat_messages = format!("[{}] Connected as {} on port {}\n\n", chrono::Local::now().format("%H:%M:%S"), username, port);
                        s.updated.insert("chat_messages".to_string(), true);
                    }
                }
            } else {
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.chat_messages.push_str(&format!("[{}] Disconnected from chat\n", chrono::Local::now().format("%H:%M:%S")));
                        s.chat_users.clear();
                        s.updated.insert("chat_messages".to_string(), true);
                        s.updated.insert("chat_users".to_string(), true);
                    }
                }
            }

            send_event(UiEvent::ModuleToggle { module: "chat".into() });
            toggle_btn_clone.set_label(if label == "Start" { "Stop" } else { "Start" });
            toggle_btn_clone.set_color(if label == "Start" { fltk::enums::Color::from_hex(0xE57373) } else { colors.accent });
        });

        refresh_btn.set_callback(move |_| {
            send_event(UiEvent::ChatRefresh);
            // Add refresh indicator to UI state
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    s.chat_messages.push_str(&format!("[{}] Refreshing user list...\n", chrono::Local::now().format("%H:%M:%S")));
                    s.updated.insert("chat_messages".to_string(), true);
                }
            }
        });

        notify_btn.set_callback(move |_| {
            send_event(UiEvent::ChatNotify);
            // Add notification indicator to UI state
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    s.chat_messages
                        .push_str(&format!("[{}] Sending notification to all users...\n", chrono::Local::now().format("%H:%M:%S")));
                    s.updated.insert("chat_messages".to_string(), true);
                }
            }
        });

        send_btn.set_callback(move |_| {
            let message = msg_input_clone.value();
            if message.is_empty() {
                fltk::dialog::alert_default("Please enter a message!");
                return;
            }

            let username = username_for_send.value();
            if username.is_empty() {
                fltk::dialog::alert_default("Please enter a username first!");
                return;
            }

            // Add message to UI state immediately (local echo)
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    s.append_chat(&username, &message);
                }
            }

            send_event(UiEvent::ChatSend { message });
            msg_input_clone.set_value(""); // Clear input after sending
        });

        // Register displays with centralized refresh manager
        super::ui_refresh::register_display("chat_users", users_display.clone());
        super::ui_refresh::register_display("chat_messages", messages_display.clone());

        // Register send button for Enter key support (primary action)
        super::ui_refresh::register_chat_button(send_btn.clone(), colors.accent);

        grp
    }
}

impl ChatTab {
    fn refresh_users(display: &mut TextDisplay) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.chat_users);
                }
            }
        }
    }

    fn refresh_messages(display: &mut TextDisplay) {
        if let Some(state) = crate::ui_state::UiState::global() {
            if let Ok(s) = state.lock() {
                if let Some(mut buf) = display.buffer() {
                    buf.set_text(&s.chat_messages);
                    let lines = buf.count_lines(0, buf.length());
                    display.scroll(lines, 0);
                }
            }
        }
    }
}
