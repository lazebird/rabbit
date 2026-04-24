//! LAN Chat Module Models

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::config::ModuleConfigs;

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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatConfig {
    pub enabled: bool,
    pub username: String,
    pub port: u16,
    pub multicast_addr: String,
}

impl From<&ModuleConfigs> for ChatConfig {
    fn from(modules: &ModuleConfigs) -> Self {
        Self {
            enabled: false,
            username: modules.get_string("chat", "username").unwrap_or_else(|| "User@PC".into()),
            port: modules.get_integer("chat", "port").unwrap_or(1314) as u16,
            multicast_addr: modules.get_string("chat", "broadcast_addr").unwrap_or_else(|| "255.255.255.255".into()),
        }
    }
}

/// Chat room state
#[derive(Debug, Clone)]
pub struct ChatRoom {
    pub messages: Vec<ChatMessage>,
    pub users: Vec<ChatUser>,
}
