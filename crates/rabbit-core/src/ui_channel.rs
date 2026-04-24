use tokio::sync::mpsc;

pub enum UiData {
    Log(Module, String),
    PingStats(String),
    PingState { address: String, progress: u32, total: u32, color: String },
    ScanProgress(String),
    PlanReminder(String),
    ChatMessage(String, String),
    ChatUserList(String),
    Error(Module, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Ping, Http, Tftpd, Tftpc, Scan, Chat, Plan,
}

pub struct UiChannels {
    pub ping_tx: mpsc::Sender<UiData>,
    pub http_tx: mpsc::Sender<UiData>,
    pub scan_tx: mpsc::Sender<UiData>,
    pub tftpd_tx: mpsc::Sender<UiData>,
    pub tftpc_tx: mpsc::Sender<UiData>,
    pub chat_tx: mpsc::Sender<UiData>,
    pub plan_tx: mpsc::Sender<UiData>,
}

impl UiChannels {
    pub fn new() -> Self {
        const BUFFER: usize = 100;
        Self {
            ping_tx: mpsc::channel(BUFFER).0,
            http_tx: mpsc::channel(BUFFER).0,
            scan_tx: mpsc::channel(BUFFER).0,
            tftpd_tx: mpsc::channel(BUFFER).0,
            tftpc_tx: mpsc::channel(BUFFER).0,
            chat_tx: mpsc::channel(BUFFER).0,
            plan_tx: mpsc::channel(BUFFER).0,
        }
    }
}

pub struct UiReceivers {
    pub ping: mpsc::Receiver<UiData>,
    pub http: mpsc::Receiver<UiData>,
    pub scan: mpsc::Receiver<UiData>,
    pub tftpd: mpsc::Receiver<UiData>,
    pub tftpc: mpsc::Receiver<UiData>,
    pub chat: mpsc::Receiver<UiData>,
    pub plan: mpsc::Receiver<UiData>,
}

pub fn create_channel() -> (mpsc::Sender<UiData>, mpsc::Receiver<UiData>) {
    mpsc::channel(100)
}