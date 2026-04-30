//! HTTP Server Service

use crate::{Result, ServiceError, ServiceUpdateResult, ui_channel::{UiData, Module}};
use axum::{
    body::HttpBody,
    extract::{Multipart, Request},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::post,
    Router,
};
use rabbit_platform::config::{get_bool, get_integer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;
use tracing::{error, info};

/// Internal HTTP server state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HttpServerState {
    Stopped,
    Starting,
    Running,
}

/// HTTP server internal configuration
#[derive(Debug, Clone, Default)]
struct ServerConfig {
    pub port: u16,
    pub root_path: String,
    pub auto_index: bool,
    pub video_play: bool,
    pub log_path: Option<String>,
}

impl ServerConfig {
    fn from_platform() -> Self {
        Self {
            port: get_integer("http", "port").unwrap_or(8000) as u16,
            root_path: get_array_first("http", "dirs").unwrap_or_else(|| ".".to_string()),
            auto_index: get_bool("http", "autoindex").unwrap_or(true),
            video_play: get_bool("http", "videoplay").unwrap_or(true),
            log_path: rabbit_platform::config::get_string("http", "log"),
        }
    }
}

fn get_array_first(module: &str, key: &str) -> Option<String> {
    rabbit_platform::config::get_array(module, key).and_then(|arr| arr.first().cloned())
}

/// HTTP server service
pub struct HttpService {
    state: Arc<RwLock<HttpServerState>>,
    log_path: Option<String>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    server_handle: Option<JoinHandle<()>>,
    tx: Option<mpsc::Sender<UiData>>,
}

impl HttpService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HttpServerState::Stopped)),
            log_path: None,
            shutdown_tx: None,
            server_handle: None,
            tx: None,
        }
    }

    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self {
        Self {
            state: Arc::new(RwLock::new(HttpServerState::Stopped)),
            log_path: None,
            shutdown_tx: None,
            server_handle: None,
            tx: Some(tx),
        }
    }

    pub async fn send(&self, data: UiData) {
        crate::send_ui(&self.tx, data).await;
    }

    /// Start the HTTP server
    pub async fn start(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        if *state != HttpServerState::Stopped {
            return Err(ServiceError::AlreadyRunning);
        }
        *state = HttpServerState::Starting;
        drop(state);

        // Pull configuration directly from platform cache
        let config = ServerConfig::from_platform();

        let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse()
            .map_err(|e| ServiceError::Config(format!("Invalid address: {}", e)))?;

        let root = PathBuf::from(&config.root_path);
        if !root.exists() {
            return Err(ServiceError::Config(format!(
                "Root path does not exist: {}",
                config.root_path
            )));
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        self.log_path = config.log_path.clone();

        // Oneshot channel to report startup result
        let (startup_tx, startup_rx) = oneshot::channel::<Result<()>>();

        let state_arc = Arc::clone(&self.state);
        let tx_clone = self.tx.clone();

        let handle = tokio::spawn(async move {
            let auto_index = config.auto_index;
            let video_play = config.video_play;
            let tx_middleware = tx_clone.clone();

            let app = Router::new()
                .route("/upload", post(upload_handler))
                .fallback_service(
                    ServeDir::new(&root)
                        .append_index_html_on_directories(auto_index)
                )
                .layer(axum::extract::Extension(root.clone()))
                .layer(axum::middleware::from_fn(
                    move |req: Request, next: Next| {
                        let ui_tx = tx_middleware.clone();
                        async move {
                            let uri = req.uri().path().to_string();
                            let mime = path2mime(&uri);
                            
                            if video_play && is_video_mime(mime) {
                                let query = req.uri().query().unwrap_or("");
                                if query.contains("videoplay=true") || !query.contains("videoplay=false") {
                                    let player_html = generate_video_player(&uri, mime);
                                    return Html(player_html).into_response();
                                }
                            }
                            
                            access_log_middleware(req, next, ui_tx).await
                        }
                    },
                ));

            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind HTTP server: {}", e);
                    *state_arc.write().await = HttpServerState::Stopped;
                    let _ = startup_tx.send(Err(ServiceError::Other(format!(
                        "Failed to bind to port {}: {}",
                        config.port, e
                    ))));
                    return;
                }
            };

            info!("HTTP server listening on {}", addr);
            *state_arc.write().await = HttpServerState::Running;
            let _ = startup_tx.send(Ok(()));
            if let Some(ref ui_tx) = tx_clone {
                let _ = ui_tx.send(UiData::Log(Module::Http, "HTTP server started".to_string())).await;
            }

            let server = axum::serve(listener, app);

            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!("HTTP server error: {}", e);
                        *state_arc.write().await = HttpServerState::Stopped;
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("HTTP server shutting down");
                }
            }

            *state_arc.write().await = HttpServerState::Stopped;
            if let Some(ref ui_tx) = tx_clone {
                let _ = ui_tx.send(UiData::Log(Module::Http, "HTTP server stopped".to_string())).await;
            }
        });

        self.server_handle = Some(handle);

        match startup_rx.await {
            Ok(result) => result,
            Err(_) => Err(ServiceError::Other("HTTP server startup timed out".into())),
        }
    }

    pub async fn update(&mut self) -> ServiceUpdateResult {
        let state = *self.state.read().await;
        match state {
            HttpServerState::Running => {
                match self.stop().await {
                    Ok(()) => ServiceUpdateResult::Stopped("HTTP server stopped".to_string()),
                    Err(e) => ServiceUpdateResult::Error(format!("Failed to stop: {}", e)),
                }
            }
            HttpServerState::Stopped => {
                match self.start().await {
                    Ok(()) => ServiceUpdateResult::Started("HTTP server started".to_string()),
                    Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
                }
            }
            _ => ServiceUpdateResult::NoChange,
        }
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }
        *self.state.write().await = HttpServerState::Stopped;
        info!("HTTP server stopped");
        Ok(())
    }
}

impl Default for HttpService {
    fn default() -> Self {
        Self::new()
    }
}

fn get_mime_types() -> HashMap<&'static str, &'static str> {
    let mut mimes = HashMap::new();
    mimes.insert(".mp4", "video/mp4");
    mimes.insert(".webm", "video/webm");
    mimes.insert(".ogg", "video/ogg");
    mimes.insert(".ogv", "video/ogg");
    mimes.insert(".avi", "video/x-msvideo");
    mimes.insert(".mov", "video/quicktime");
    mimes.insert(".wmv", "video/x-ms-wmv");
    mimes.insert(".flv", "video/x-flv");
    mimes.insert(".mkv", "video/x-matroska");
    mimes.insert(".m4v", "video/x-m4v");
    mimes
}

fn path2mime(path: &str) -> &'static str {
    let mimes = get_mime_types();
    if let Some(filename) = path.rsplit('/').next() {
        if let Some(dot_pos) = filename.rfind('.') {
            let ext = &filename[dot_pos..].to_lowercase();
            if let Some(&mime) = mimes.get(ext.as_str()) {
                return mime;
            }
        }
    }
    "application/octet-stream"
}

fn is_video_mime(mime: &str) -> bool {
    mime.starts_with("video/")
}

fn generate_video_player(uri: &str, mime: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Video Player</title>
    <link href="https://vjs.zencdn.net/7.1.0/video-js.css" rel="stylesheet">
    <script src="https://vjs.zencdn.net/7.1.0/video.js"></script>
</head>
<body>
    <video id="my-video" class="video-js" controls preload="auto" width="640" height="264">
        <source src="{}?videoplay=false" type='{}'>
    </video>
</body>
</html>"#,
        uri, mime
    )
}

async fn access_log_middleware(
    request: Request,
    next: Next,
    ui_tx: Option<mpsc::Sender<UiData>>,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status();
    let bytes_sent = response.body().size_hint().lower();

    let log_msg = format!("[{}] {} {} {} - {} bytes ({}ms)",
        chrono::Local::now().format("%H:%M:%S"),
        method, uri.path(), status.as_u16(), bytes_sent, duration.as_millis()
    );

    if let Some(ref ui_tx) = ui_tx {
        let _ = ui_tx.send(UiData::Log(Module::Http, log_msg.clone())).await;
    }

    response
}

async fn upload_handler(
    axum::extract::Extension(root): axum::extract::Extension<PathBuf>,
    mut multipart: Multipart,
) -> Html<String> {
    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("unknown").to_string();
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        if name == "file" {
            let file_path = root.join(&file_name);
            match File::create(&file_path).await {
                Ok(mut file) => {
                    let mut success = true;
                    while let Ok(Some(chunk)) = field.chunk().await {
                        if let Err(e) = file.write_all(&chunk).await {
                            error!("Failed to write chunk: {}", e);
                            success = false;
                            break;
                        }
                    }
                    if success {
                        return Html(format!(
                            "<html><body><h1>Upload Successful</h1><p>File '{}' saved.</p><a href=\"/\">Back</a></body></html>",
                            file_name
                        ));
                    }
                }
                Err(e) => {
                    return Html(format!(
                        "<html><body><h1>Upload Failed</h1><p>Failed to create file: {}</p><a href=\"/\">Back</a></body></html>",
                        e
                    ));
                }
            }
        }
    }
    Html("<html><body><h1>No file received</h1><a href=\"/\">Back</a></body></html>".to_string())
}
