//! LAN Chat Module Models

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Announcement,
}

/// Chat user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUser {
    pub username: String,
    pub hostname: String,
    pub last_seen: DateTime<Local>,
    pub online: bool,
}

/// Chat configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub enabled: bool,
    pub username: String,
    pub port: u16,
    pub multicast_addr: String,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            username: String::from("User"),
            port: 5000,
            multicast_addr: String::from("239.255.255.250"),
        }
    }
}

/// Chat room state
#[derive(Debug, Clone)]
pub struct ChatRoom {
    pub messages: Vec<ChatMessage>,
    pub users: Vec<ChatUser>,
}
