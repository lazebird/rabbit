pub use schema::{Module, UiData};
use tokio::sync::mpsc;

/// Helper to send data through UI channel (eliminates duplicate send() implementations)
pub async fn send_ui(tx: &Option<mpsc::Sender<UiData>>, data: UiData) {
    if let Some(sender) = tx {
        let _ = sender.send(data).await;
    }
}
