//! LAN Chat Service

use crate::{Result, ServiceError};
use rabbit_models::chat::{ChatConfig, ChatMessage, ChatRoom, ChatUser, MessageType};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tracing::{error, info};

/// LAN Chat service
pub struct ChatService {
    config: Arc<RwLock<ChatConfig>>,
    room: Arc<RwLock<ChatRoom>>,
    socket: Arc<RwLock<Option<Arc<UdpSocket>>>>,
    message_tx: Option<mpsc::Sender<ChatMessage>>,
    recv_handle: Option<JoinHandle<()>>,
    heartbeat_handle: Option<JoinHandle<()>>,
    user_activity: Arc<RwLock<HashMap<String, tokio::time::Instant>>>,
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
            heartbeat_handle: None,
            user_activity: Arc::new(RwLock::new(HashMap::new())),
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
        let user_activity = Arc::clone(&self.user_activity);

        // Spawn receive task
        let handle = tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];

            loop {
                tokio::select! {
                    result = socket.recv_from(&mut buf) => {
                        match result {
                            Ok((len, addr)) => {
                                if let Ok(msg) = Self::parse_message(&buf[..len], addr) {
                                    // Update user activity
                                    user_activity.write().await.insert(
                                        msg.sender.clone(),
                                        tokio::time::Instant::now()
                                    );
                                    
                                    // Add user if not exists
                                    let mut room_guard = room.write().await;
                                    if !room_guard.users.iter().any(|u| u.username == msg.sender) {
                                        room_guard.users.push(ChatUser {
                                            username: msg.sender.clone(),
                                            hostname: String::new(),
                                            online: true,
                                            last_seen: chrono::Local::now(),
                                        });
                                    }
                                    
                                    room_guard.messages.push(msg);
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

        // Spawn heartbeat task for online status detection
        let room = Arc::clone(&self.room);
        let user_activity = Arc::clone(&self.user_activity);
        let heartbeat_handle = tokio::spawn(async move {
            let mut heartbeat_interval = interval(Duration::from_secs(30));
            let timeout_duration = Duration::from_secs(120); // 2 minutes timeout

            loop {
                heartbeat_interval.tick().await;

                let now = tokio::time::Instant::now();
                let mut activity_guard = user_activity.write().await;
                let mut room_guard = room.write().await;

                // Check for inactive users
                let inactive_users: Vec<String> = activity_guard
                    .iter()
                    .filter(|(_, last_seen)| now.saturating_duration_since(**last_seen) > timeout_duration)
                    .map(|(name, _)| name.clone())
                    .collect();

                for user_name in inactive_users {
                    activity_guard.remove(&user_name);
                    if let Some(user) = room_guard.users.iter_mut().find(|u| u.username == user_name) {
                        user.online = false;
                        user.last_seen = chrono::Local::now();
                    }
                }

                // Update last_seen for online users
                for user in room_guard.users.iter_mut() {
                    if activity_guard.contains_key(&user.username) {
                        user.online = true;
                        user.last_seen = chrono::Local::now();
                    }
                }
            }
        });

        self.heartbeat_handle = Some(heartbeat_handle);

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

    /// Get online users (only those with online=true)
    pub async fn get_users(&self) -> Vec<ChatUser> {
        self.room.read().await.users
            .iter()
            .filter(|u| u.online)
            .cloned()
            .collect()
    }

    /// Get all users (including offline)
    pub async fn get_all_users(&self) -> Vec<ChatUser> {
        self.room.read().await.users.clone()
    }

    /// Refresh user list - clears offline users and broadcasts presence
    pub async fn refresh_users(&self) -> Result<()> {
        // Broadcast presence announcement
        self.send_message("Presence check", MessageType::Announcement).await?;
        
        // Clear offline users
        let mut room = self.room.write().await;
        room.users.retain(|u| u.online);
        
        info!("User list refreshed, {} users online", room.users.len());
        Ok(())
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
