//! UI Events - Bridge between FLTK UI and async services
//!
//! FLTK runs in a synchronous event loop, while services are async.
//! This module provides a message passing system to bridge the two.

use parking_lot::Mutex;
use std::sync::mpsc::{channel, Sender, Receiver};

/// Global event sender (initialized in App::run)
static GLOBAL_EVENT_SENDER: Mutex<Option<Sender<UiEvent>>> = Mutex::new(None);

/// UI Events that can be triggered from UI callbacks
#[derive(Debug, Clone)]
pub enum UiEvent {
    // Module Toggle - 通用模块状态切换
    ModuleToggle { module: String },

    // TFTP Client
    TftpClientPut { server: String, local: String, remote: String, options: String },
    TftpClientGet { server: String, local: String, remote: String, options: String },

    // Plan
    PlanAdd { date: String, time: String, cycle: i32, unit: String, msg: String },
    PlanRemove { id: String },

    // Chat
    ChatSend { message: String },
    ChatRefresh,
    ChatNotify,

    // Settings
    SettingsSave,

    // Version Check
    VersionCheck,
}

/// Initialize the global event sender
pub fn init_event_system() -> Receiver<UiEvent> {
    let (sender, receiver) = channel::<UiEvent>();
    *GLOBAL_EVENT_SENDER.lock() = Some(sender);
    receiver
}

/// Send a UI event from anywhere (safe to call from FLTK callbacks)
pub fn send_event(event: UiEvent) {
    if let Some(ref sender) = *GLOBAL_EVENT_SENDER.lock() {
        let _ = sender.send(event);
    }
}

/// Check if event system is initialized
pub fn is_event_system_ready() -> bool {
    GLOBAL_EVENT_SENDER.lock().is_some()
}

/// Event handler trait for processing UI events
#[async_trait::async_trait]
pub trait EventHandler {
    async fn handle_event(&mut self, event: UiEvent) -> anyhow::Result<()>;
}