//! Chat Tab UI Component
//!
//! Layout for LAN chat functionality:
//! - Row 1: Username | Port | Start button
//! - Row 2: Broadcast addr | Refresh | Notify
//! - User list (left) | Messages (right)
//! - Input row

use fltk::{
    button::Button,
    frame::Frame,
    group::Flex,
    input::{Input, IntInput},
    prelude::*,
    text::{TextBuffer, TextDisplay},
};

use crate::ui_events::{UiEvent, send_event};
use crate::ui_state::UiState;
use super::{TabComponent, Colors, Spacing, defaults};

/// Chat Tab Component
pub struct ChatTab;

impl TabComponent for ChatTab {
    fn build(x: i32, y: i32, w: i32, h: i32) -> Flex {
        let colors = Colors::new();
        let spacing = Spacing::new();

        let mut grp = Flex::new(x, y, w, h, "CHAT").column();
        grp.set_margin(spacing.margin);
        grp.set_spacing(spacing.padding);

        // Row 1: Username and Port
        let mut row1 = Flex::default().row();
        row1.set_spacing(spacing.padding);

        let _user_label = Frame::default().with_label("Name");

        let mut username_input = Input::default();
        username_input.set_value(&defaults::chat_username());

        let _port_label = Frame::default().with_label("Port");

        let mut port_input = IntInput::default();
        port_input.set_value(&defaults::chat_port().to_string());

        Frame::default(); // Spacer

        let mut toggle_btn = Button::default().with_label("Start");
        toggle_btn.set_color(colors.accent);
        toggle_btn.set_label_color(fltk::enums::Color::White);

        row1.end();
        grp.fixed(&row1, spacing.row_height);

        // Row 2: Broadcast and buttons
        let mut row2 = Flex::default().row();
        row2.set_spacing(spacing.padding);

        let _bcast_label = Frame::default().with_label("Broadcast");

        let mut broadcast_input = Input::default();
        broadcast_input.set_value(&defaults::chat_broadcast());

        let mut refresh_btn = Button::default().with_label("Refresh");

        let mut notify_btn = Button::default().with_label("Notify");

        Frame::default(); // Spacer
        row2.end();
        grp.fixed(&row2, spacing.row_height);

        // Main content area: Users list | Messages
        let mut content_row = Flex::default().row();

        // Users column
        let mut users_col = Flex::default().column();
        let users_label = Frame::default().with_label("Users:");
        users_col.fixed(&users_label, 20);
        let mut users_display = TextDisplay::default();
        let users_buf = TextBuffer::default();
        users_display.set_buffer(Some(users_buf));
        users_col.end();
        content_row.fixed(&users_col, 120);

        // Messages column
        let mut msg_col = Flex::default().column();
        let msg_label = Frame::default().with_label("Messages:");
        msg_col.fixed(&msg_label, 20);
        let mut messages_display = TextDisplay::default();
        let msg_buf = TextBuffer::default();
        messages_display.set_buffer(Some(msg_buf));
        msg_col.end();

        content_row.end();

        // Row 3: Input
        let mut row3 = Flex::default().row();
        row3.set_spacing(spacing.padding);

        let _input_label = Frame::default().with_label("Input:");

        let mut msg_input = Input::default();

        let mut send_btn = Button::default().with_label("Send");
        send_btn.set_color(colors.accent);
        send_btn.set_label_color(fltk::enums::Color::White);

        Frame::default(); // Spacer
        row3.end();
        grp.fixed(&row3, spacing.row_height);

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
        let broadcast_input_clone = broadcast_input.clone();
        let mut toggle_btn_clone = toggle_btn.clone();

        let mut msg_input_clone = msg_input.clone();
        let username_for_send = username_input.clone();

        // Add button callbacks
        toggle_btn.set_callback(move |_| {
            let label = toggle_btn_clone.label();
            let username = username_input_clone.value();
            let port = port_input_clone.value().parse::<u16>().unwrap_or(1314);
            let broadcast = broadcast_input_clone.value();

            if label == "Start" {
                if username.is_empty() {
                    fltk::dialog::alert_default("Please enter a username!");
                    return;
                }

                // Update UI state to show connected
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.chat_messages = format!(
                            "LAN Chat:\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n[{}] Connected as {} on port {}\n\n",
                            chrono::Local::now().format("%H:%M:%S"),
                            username, port
                        );
                        s.updated.insert("chat_messages".to_string(), true);
                    }
                }

                send_event(UiEvent::ChatToggle { username: username.clone(), port, broadcast });
                toggle_btn_clone.set_label("Stop");
                toggle_btn_clone.set_color(fltk::enums::Color::from_hex(0xE57373));
            } else {
                send_event(UiEvent::ChatToggle { username: username.clone(), port, broadcast });
                toggle_btn_clone.set_label("Start");
                toggle_btn_clone.set_color(colors.accent);

                // Update UI state to show disconnected
                if let Some(state) = UiState::global() {
                    if let Ok(mut s) = state.lock() {
                        s.chat_messages.push_str(&format!(
                            "[{}] Disconnected from chat\n",
                            chrono::Local::now().format("%H:%M:%S")
                        ));
                        s.chat_users = "Online Users:\n───────────\n(no users)\n".to_string();
                        s.updated.insert("chat_messages".to_string(), true);
                        s.updated.insert("chat_users".to_string(), true);
                    }
                }
            }
        });

        refresh_btn.set_callback(move |_| {
            send_event(UiEvent::ChatRefresh);
            // Add refresh indicator to UI state
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    s.chat_messages.push_str(&format!(
                        "[{}] Refreshing user list...\n",
                        chrono::Local::now().format("%H:%M:%S")
                    ));
                    s.updated.insert("chat_messages".to_string(), true);
                }
            }
        });

        notify_btn.set_callback(move |_| {
            send_event(UiEvent::ChatNotify);
            // Add notification indicator to UI state
            if let Some(state) = UiState::global() {
                if let Ok(mut s) = state.lock() {
                    s.chat_messages.push_str(&format!(
                        "[{}] Sending notification to all users...\n",
                        chrono::Local::now().format("%H:%M:%S")
                    ));
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

        // Periodic refresh for both displays
        let mut users_display_timer = users_display.clone();
        let mut messages_display_timer = messages_display.clone();
        fltk::app::add_idle3(move |_| {
            Self::refresh_users(&mut users_display_timer);
            Self::refresh_messages(&mut messages_display_timer);
        });

        grp
    }
}

impl ChatTab {
    fn refresh_users(display: &mut TextDisplay) {
        if let Some(state) = UiState::global() {
            if let Ok(mut s) = state.lock() {
                if s.is_updated("chat_users") {
                    if let Some(buf) = display.buffer() {
                        let current_text = buf.text();
                        if current_text != s.chat_users {
                            drop(buf);
                            if let Some(mut new_buf) = display.buffer() {
                                new_buf.set_text(&s.chat_users);
                                display.set_buffer(Some(new_buf));
                            }
                        }
                    }
                    s.clear_updated("chat_users");
                }
            }
        }
    }

    fn refresh_messages(display: &mut TextDisplay) {
        if let Some(state) = UiState::global() {
            if let Ok(mut s) = state.lock() {
                if s.is_updated("chat_messages") {
                    if let Some(buf) = display.buffer() {
                        let current_text = buf.text();
                        if current_text != s.chat_messages {
                            drop(buf);
                            if let Some(mut new_buf) = display.buffer() {
                                new_buf.set_text(&s.chat_messages);
                                let lines = new_buf.count_lines(0, new_buf.length());
                                display.set_buffer(Some(new_buf));
                                // Auto-scroll to bottom
                                display.scroll(lines, 0);
                            }
                        }
                    }
                    s.clear_updated("chat_messages");
                }
            }
        }
    }
}
