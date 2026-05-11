//! LAN Chat Service

use crate::{ui_channel::UiData, Result, ServiceError, ServiceUpdateResult};
use rabbit_config::{get_integer, get_string};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info};

/// Internal Message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum MessageType {
    Text,
    Announcement,
}

/// Internal Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub message_type: MessageType,
}

/// Internal Chat user
#[derive(Debug, Clone)]
struct ChatUser {
    pub username: String,
    pub online: bool,
}

/// LAN Chat internal configuration
#[derive(Debug, Clone, Default)]
struct ChatConfig {
    pub port: u16,
}

impl ChatConfig {
    fn from_platform() -> Self {
        Self {
            port: get_integer("chat", "port").unwrap_or(1314) as u16,
        }
    }
}

/// LAN Chat service
pub struct ChatService {
    messages: Arc<RwLock<Vec<ChatMessage>>>,
    users: Arc<RwLock<HashMap<String, ChatUser>>>,
    socket: Arc<RwLock<Option<Arc<UdpSocket>>>>,
    message_tx: Option<mpsc::Sender<ChatMessage>>,
    recv_handle: Option<JoinHandle<()>>,
    heartbeat_handle: Option<JoinHandle<()>>,
    tx: Option<mpsc::Sender<UiData>>,
}

impl ChatService {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            socket: Arc::new(RwLock::new(None)),
            message_tx: None,
            recv_handle: None,
            heartbeat_handle: None,
            tx: None,
        }
    }

    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            socket: Arc::new(RwLock::new(None)),
            message_tx: None,
            recv_handle: None,
            heartbeat_handle: None,
            tx: Some(tx),
        }
    }

    pub async fn send(&self, data: UiData) {
        crate::send_ui(&self.tx, data).await;
    }

    /// 内部启动 Chat 服务
    async fn start(&mut self) -> Result<()> {
        if self.recv_handle.is_some() {
            return Err(ServiceError::AlreadyRunning);
        }

        // Pull configuration directly from platform cache
        let config = ChatConfig::from_platform();

        // Create UDP socket
        let bind_addr = format!("0.0.0.0:{}", config.port);
        let socket = UdpSocket::bind(&bind_addr).await.map_err(ServiceError::Io)?;

        socket.set_broadcast(true)?;

        let socket_arc = Arc::new(socket);
        *self.socket.write().await = Some(Arc::clone(&socket_arc));

        let (tx, mut rx) = mpsc::channel(100);
        self.message_tx = Some(tx);

        let messages_arc = Arc::clone(&self.messages);
        let users_arc = Arc::clone(&self.users);
        let tx_ui = self.tx.clone();
        let port = config.port;

        let handle = tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];

            loop {
                tokio::select! {
                    result = socket_arc.recv_from(&mut buf) => {
                        match result {
                            Ok((len, _addr)) => {
                                if let Ok(msg) = serde_json::from_slice::<ChatMessage>(&buf[..len]) {
                                    // Add user if not exists, track if list changed
                                    let mut users_guard = users_arc.write().await;
                                    let is_new_user = !users_guard.values()
                                        .any(|u| u.username == msg.sender);

                                    users_guard.insert(msg.sender.clone(), ChatUser {
                                        username: msg.sender.clone(),
                                        online: true,
                                    });

                                    messages_arc.write().await.push(msg.clone());

                                    if let Some(ref ui_tx) = tx_ui {
                                        let _ = ui_tx.send(UiData::ChatMessage(msg.sender.clone(), msg.content.clone())).await;

                                        // Only send user list if it changed
                                        if is_new_user {
                                            let user_list: Vec<String> = users_guard.values()
                                                    .filter(|u| u.online)
                                                    .map(|u| u.username.clone())
                                                    .collect();
                                            let _ = ui_tx.send(UiData::ChatUserList(user_list)).await;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("UDP receive error: {}", e);
                            }
                        }
                    },
                    Some(msg) = rx.recv() => {
                        match serde_json::to_vec(&msg) {
                            Ok(data) => {
                                let broadcast_addr = format!("255.255.255.255:{}", port);
                                let _ = socket_arc.send_to(&data, broadcast_addr).await;
                            }
                            Err(e) => {
                                error!("Failed to serialize chat message: {}", e);
                            }
                        }
                    },
                }
            }
        });

        self.recv_handle = Some(handle);
        Ok(())
    }

    pub async fn update(&mut self) -> ServiceUpdateResult {
        if self.recv_handle.is_some() {
            match self.stop().await {
                Ok(()) => ServiceUpdateResult::Stopped("Chat stopped".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to stop: {}", e)),
            }
        } else {
            match self.start().await {
                Ok(()) => ServiceUpdateResult::Started("Chat started".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
            }
        }
    }

    /// 程序退出时调用，销毁资源，不发状态通告
    pub async fn destroy(&mut self) -> Result<()> {
        self.send_message("Left the chat", MessageType::Announcement).await.ok();

        if let Some(tx) = self.message_tx.take() {
            drop(tx);
        }

        if let Some(handle) = self.recv_handle.take() {
            handle.abort();
        }

        if let Some(handle) = self.heartbeat_handle.take() {
            handle.abort();
        }

        *self.socket.write().await = None;
        info!("Chat service destroyed");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.send_message("Left the chat", MessageType::Announcement).await.ok();

        if let Some(tx) = self.message_tx.take() {
            drop(tx);
        }

        if let Some(handle) = self.recv_handle.take() {
            handle.abort();
        }

        if let Some(handle) = self.heartbeat_handle.take() {
            handle.abort();
        }

        *self.socket.write().await = None;
        info!("Chat service stopped");
        Ok(())
    }

    /// Send a text message
    pub async fn send_text(&self, content: &str) -> Result<()> {
        self.send_message(content, MessageType::Text).await
    }

    /// Send a message
    async fn send_message(&self, content: &str, msg_type: MessageType) -> Result<()> {
        let username = get_string("chat", "username").unwrap_or_else(|| "User@PC".to_string());

        let message = ChatMessage {
            id: format!("msg_{}", chrono::Local::now().timestamp_millis()),
            sender: username,
            content: content.to_string(),
            timestamp: chrono::Local::now(),
            message_type: msg_type,
        };

        if let Some(tx) = &self.message_tx {
            tx.send(message).await.map_err(|_| ServiceError::Other("Message channel closed".into()))?;
        }

        Ok(())
    }

    /// Refresh user list - clears offline users and broadcasts presence
    pub async fn refresh_users(&self) -> Result<()> {
        self.send_message("Presence check", MessageType::Announcement).await?;
        let mut users = self.users.write().await;
        users.clear();
        info!("User list cleared for refresh");
        Ok(())
    }
}

impl Default for ChatService {
    fn default() -> Self {
        Self::new()
    }
}
