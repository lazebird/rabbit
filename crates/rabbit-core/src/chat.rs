//! LAN Chat Service

use crate::{Result, ServiceError};
use rabbit_models::chat::{ChatConfig, ChatMessage, ChatRoom, ChatUser, MessageType};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info};

/// LAN Chat service
pub struct ChatService {
    config: Arc<RwLock<ChatConfig>>,
    room: Arc<RwLock<ChatRoom>>,
    socket: Arc<RwLock<Option<Arc<UdpSocket>>>>,
    message_tx: Option<mpsc::Sender<ChatMessage>>,
    recv_handle: Option<JoinHandle<()>>,
}

impl ChatService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(ChatConfig::default())),
            room: Arc::new(RwLock::new(ChatRoom {
                messages: Vec::new(),
                users: Vec::new(),
            })),
            socket: Arc::new(RwLock::new(None)),
            message_tx: None,
            recv_handle: None,
        }
    }

    /// Initialize with configuration
    pub async fn init(&mut self, config: ChatConfig) -> Result<()> {
        *self.config.write().await = config;
        info!("Chat service initialized");
        Ok(())
    }

    /// Start the chat service
    pub async fn start(&mut self) -> Result<()> {
        if self.recv_handle.is_some() {
            return Err(ServiceError::AlreadyRunning);
        }

        let _config = self.config.read().await.clone();
        let config_clone = self.config.read().await.clone();
        if !config_clone.enabled {
            return Ok(());
        }

        // Create UDP socket
        let bind_addr = format!("0.0.0.0:{}", config_clone.port);
        let socket = UdpSocket::bind(&bind_addr).await
            .map_err(|e| ServiceError::Io(e))?;

        // Enable broadcast
        socket.set_broadcast(true)?;

        let socket = Arc::new(socket);
        *self.socket.write().await = Some(Arc::clone(&socket));

        let (tx, mut rx) = mpsc::channel(100);
        self.message_tx = Some(tx);

        let room = Arc::clone(&self.room);
        let config = Arc::clone(&self.config);

        // Spawn receive task
        let handle = tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];

            loop {
                tokio::select! {
                    result = socket.recv_from(&mut buf) => {
                        match result {
                            Ok((len, addr)) => {
                                if let Ok(msg) = Self::parse_message(&buf[..len], addr) {
                                    room.write().await.messages.push(msg);
                                }
                            }
                            Err(e) => {
                                error!("UDP receive error: {}", e);
                            }
                        }
                    }
                    Some(msg) = rx.recv() => {
                        // Send message
                        let data = serde_json::to_vec(&msg).unwrap_or_default();
                        let broadcast_addr = format!("255.255.255.255:{}",
                            config.read().await.port);
                        let _ = socket.send_to(&data, broadcast_addr).await;
                    }
                }
            }
        });

        self.recv_handle = Some(handle);

        // Send announcement
        self.send_message("Joined the chat", MessageType::Announcement).await?;

        info!("Chat service started on port {}", config_clone.port);
        Ok(())
    }

    /// Stop the chat service
    pub async fn stop(&mut self) -> Result<()> {
        // Send leave announcement
        self.send_message("Left the chat", MessageType::Announcement).await.ok();

        if let Some(tx) = self.message_tx.take() {
            drop(tx);
        }

        if let Some(handle) = self.recv_handle.take() {
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
        let config = self.config.read().await;

        let message = ChatMessage {
            id: format!("msg_{}", chrono::Local::now().timestamp_millis()),
            sender: config.username.clone(),
            content: content.to_string(),
            timestamp: chrono::Local::now(),
            message_type: msg_type,
        };

        if let Some(tx) = &self.message_tx {
            tx.send(message).await
                .map_err(|_| ServiceError::Other("Message channel closed".into()))?;
        }

        Ok(())
    }

    /// Get all messages
    pub async fn get_messages(&self) -> Vec<ChatMessage> {
        self.room.read().await.messages.clone()
    }

    /// Get online users
    pub async fn get_users(&self) -> Vec<ChatUser> {
        self.room.read().await.users.clone()
    }

    /// Update username
    pub async fn set_username(&self, username: String) -> Result<()> {
        self.config.write().await.username = username;
        Ok(())
    }

    /// Parse received message
    fn parse_message(data: &[u8], _addr: SocketAddr) -> Result<ChatMessage> {
        serde_json::from_slice(data)
            .map_err(|e| ServiceError::Other(format!("Parse error: {}", e)))
    }
}

impl Default for ChatService {
    fn default() -> Self {
        Self::new()
    }
}
