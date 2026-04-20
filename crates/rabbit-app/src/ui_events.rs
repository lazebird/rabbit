//! UI Events - Bridge between FLTK UI and async services
//!
//! FLTK runs in a synchronous event loop, while services are async.
//! This module provides a message passing system to bridge the two.

use std::sync::mpsc::{channel, Sender, Receiver};

/// Global event sender (initialized in App::run)
static mut GLOBAL_EVENT_SENDER: Option<Sender<UiEvent>> = None;

/// UI Events that can be triggered from UI callbacks
#[derive(Debug, Clone)]
pub enum UiEvent {
    // Ping
    PingStart { target: String, options: String },
    PingStop,

    // Scan
    ScanStart { start_ip: String, end_ip: String, options: String },
    ScanStop,

    // HTTP Server
    HttpToggle { port: u16, options: String, shell: bool },

    // TFTP Server
    TftpServerToggle { options: String },
    TftpServerAddDir,
    TftpServerRemoveDir,

    // TFTP Client
    TftpClientPut { server: String, local: String, remote: String, options: String },
    TftpClientGet { server: String, local: String, remote: String, options: String },

    // Plan
    PlanAdd { date: String, time: String, cycle: i32, unit: String, msg: String },
    PlanRemove { id: String },

    // Chat
    ChatToggle { username: String, port: u16, broadcast: String },
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
    unsafe {
        GLOBAL_EVENT_SENDER = Some(sender);
    }
    receiver
}

/// Send a UI event from anywhere (safe to call from FLTK callbacks)
pub fn send_event(event: UiEvent) {
    unsafe {
        if let Some(ref sender) = GLOBAL_EVENT_SENDER {
            let _ = sender.send(event);
        }
    }
}

/// Check if event system is initialized
pub fn is_event_system_ready() -> bool {
    unsafe { GLOBAL_EVENT_SENDER.is_some() }
}

/// Event handler trait for processing UI events
#[async_trait::async_trait]
pub trait EventHandler {
    async fn handle_event(&mut self, event: UiEvent) -> anyhow::Result<()>;
}
