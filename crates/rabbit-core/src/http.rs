//! HTTP Server Service

use crate::{Result, ServiceError, ServiceUpdateResult, ui_channel::{UiData, Module}};
use axum::{
    body::HttpBody,
    extract::{ConnectInfo, Multipart, Request},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::post,
    Router,
};
use rabbit_models::http::{HttpAccessLog, HttpServerState};
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

/// HTTP server internal configuration
#[derive(Debug, Clone, Default)]
struct ServerConfig {
    pub port: u16,
    pub root_path: String,
    pub auto_index: bool,
    pub video_play: bool,
}

impl ServerConfig {
    fn from_platform() -> Self {
        Self {
            port: get_integer("http", "port").unwrap_or(8000) as u16,
            root_path: get_array_first("http", "dirs").unwrap_or_else(|| ".".to_string()),
            auto_index: get_bool("http", "autoindex").unwrap_or(true),
            video_play: get_bool("http", "videoplay").unwrap_or(true),
        }
    }
}

fn get_array_first(module: &str, key: &str) -> Option<String> {
    rabbit_platform::config::get_array(module, key).and_then(|arr| arr.first().cloned())
}

/// HTTP server service
pub struct HttpService {
    state: Arc<RwLock<HttpServerState>>,
    logs: Arc<RwLock<Vec<HttpAccessLog>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    server_handle: Option<JoinHandle<()>>,
    tx: Option<mpsc::Sender<UiData>>,
}

impl HttpService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HttpServerState::Stopped)),
            logs: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx: None,
            server_handle: None,
            tx: None,
        }
    }

    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self {
        Self {
            state: Arc::new(RwLock::new(HttpServerState::Stopped)),
            logs: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx: None,
            server_handle: None,
            tx: Some(tx),
        }
    }

    pub async fn send(&self, data: UiData) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(data).await;
        }
    }

    /// Initialize - now a no-op as config is pulled on start
    pub async fn init(&mut self) -> Result<()> {
        info!("HTTP service initialized");
        Ok(())
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

        // Oneshot channel to report startup result
        let (startup_tx, startup_rx) = oneshot::channel::<Result<()>>();

        let state = Arc::clone(&self.state);
        let logs = Arc::clone(&self.logs);
        let tx = self.tx.clone();

        let handle = tokio::spawn(async move {
            // Build axum router with access logging and file upload
            let logs_clone = Arc::clone(&logs);
            let auto_index = config.auto_index;
            let video_play = config.video_play;

            let app = Router::new()
                .route("/upload", post(upload_handler))
                .fallback_service(
                    ServeDir::new(&root)
                        .append_index_html_on_directories(auto_index)
                )
                .layer(axum::extract::Extension(root.clone()))
                .layer(axum::middleware::from_fn(
                    move |req: Request, next: Next| {
                        let logs = Arc::clone(&logs_clone);
                        let video_play = video_play;
                        async move {
                            // Check if this is a video file request and video_play is enabled
                            let uri = req.uri().path().to_string();
                            let mime = path2mime(&uri);
                            
                            if video_play && is_video_mime(mime) {
                                let query = req.uri().query().unwrap_or("");
                                // Show player page if videoplay=true or if no videoplay parameter
                                if query.contains("videoplay=true") || !query.contains("videoplay=false") {
                                    let player_html = generate_video_player(&uri, mime);
                                    return Html(player_html).into_response();
                                }
                            }
                            
                            access_log_middleware(req, next, logs).await
                        }
                    },
                ));

            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind HTTP server: {}", e);
                    *state.write().await = HttpServerState::Stopped;
                    let _ = startup_tx.send(Err(ServiceError::Other(format!(
                        "Failed to bind to port {}: {}",
                        config.port, e
                    ))));
                    return;
                }
            };

            info!("HTTP server listening on {}", addr);
            *state.write().await = HttpServerState::Running;
            let _ = startup_tx.send(Ok(()));
            if let Some(ref tx) = tx {
                let _ = tx.send(UiData::Log(Module::Http, "HTTP server started".to_string())).await;
            }

            // Run server with shutdown signal
            let server = axum::serve(listener, app);

            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!("HTTP server error: {}", e);
                        *state.write().await = HttpServerState::Stopped;
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("HTTP server shutting down");
                }
            }

            *state.write().await = HttpServerState::Stopped;
            if let Some(ref tx) = tx {
                let _ = tx.send(UiData::Log(Module::Http, "HTTP server stopped".to_string())).await;
            }
        });

        self.server_handle = Some(handle);

        // Wait for startup result
        match startup_rx.await {
            Ok(result) => result,
            Err(_) => Err(ServiceError::Other("HTTP server startup timed out".into())),
        }
    }


    /// Stop the HTTP server
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

    pub fn is_running(&self) -> bool {
        if let Ok(s) = self.state.try_read() {
            *s == HttpServerState::Running
        } else {
            false
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }

        *self.state.write().await = HttpServerState::Stopped;
        info!("HTTP service stopped");
        Ok(())
    }

    /// Get current state
    pub async fn get_state(&self) -> HttpServerState {
        *self.state.read().await
    }

    /// Get access logs
    pub async fn get_logs(&self) -> Vec<HttpAccessLog> {
        self.logs.read().await.clone()
    }


    /// Get recent logs (limited count)
    pub async fn get_recent_logs(&self, count: usize) -> Vec<HttpAccessLog> {
        let logs = self.logs.read().await;
        let start = logs.len().saturating_sub(count);
        logs[start..].to_vec()
    }

    /// Clear access logs
    pub async fn clear_logs(&self) {
        self.logs.write().await.clear();
    }
}

impl Default for HttpService {
    fn default() -> Self {
        Self::new()
    }
}

/// MIME types mapping for video files
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
    mimes.insert(".*", "application/octet-stream");
    mimes
}

/// Get MIME type from file path
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

/// Check if MIME type is video
fn is_video_mime(mime: &str) -> bool {
    mime.starts_with("video/")
}

/// Generate video player HTML page using Video.js (compatible with old implementation)
fn generate_video_player(uri: &str, mime: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Video Player</title>
    <link href="https://vjs.zencdn.net/7.1.0/video-js.css" rel="stylesheet">
    <script src="https://vjs.zencdn.net/7.1.0/video.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/videojs-flash@2/dist/videojs-flash.min.js"></script>
</head>
<body>
    <video id="my-video" class="video-js" controls preload="auto" width="640" height="264" 
           poster="MY_VIDEO_POSTER.jpg" data-setup='{{"techOrder": ["flash","html5"]}}'>
        <source src="{}?videoplay=false" type='{}'>
        <p class="vjs-no-js">
            To view this video please enable JavaScript, and consider upgrading to a web browser that
            <a href="https://videojs.com/html5-video-support/" target="_blank">supports HTML5 video</a>
        </p>
    </video>
</body>
</html>"#,
        uri, mime
    )
}

/// Access logging middleware
async fn access_log_middleware(
    request: Request,
    next: Next,
    logs: Arc<RwLock<Vec<HttpAccessLog>>>,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let client_addr = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|addr| addr.0.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status();
    let bytes_sent = response.body().size_hint().lower();

    // Log the request
    let log_entry = HttpAccessLog {
        timestamp: chrono::Local::now(),
        method: method.to_string(),
        path: uri.path().to_string(),
        status_code: status.as_u16(),
        bytes_sent,
        client_addr: client_addr.clone(),
    };

    // Add to logs (keep last 1000 entries)
    let mut logs = logs.write().await;
    logs.push(log_entry);
    if logs.len() > 1000 {
        logs.remove(0);
    }

    info!(
        "[{}] {} {} - {} ({:.2}ms)",
        client_addr,
        method,
        uri.path(),
        status.as_u16(),
        duration.as_secs_f64() * 1000.0
    );

    response
}

/// File upload handler
async fn upload_handler(
    axum::extract::Extension(root): axum::extract::Extension<PathBuf>,
    mut multipart: Multipart,
) -> Html<String> {
    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("unknown").to_string();
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        
        if name == "file" {
            let file_path = root.join(&file_name);
            
            // Create file
            match File::create(&file_path).await {
                Ok(mut file) => {
                    // Write chunks
                    let mut success = true;
                    while let Ok(Some(chunk)) = field.chunk().await {
                        if let Err(e) = file.write_all(&chunk).await {
                            error!("Failed to write chunk: {}", e);
                            success = false;
                            break;
                        }
                    }
                    
                    if success {
                        info!("File uploaded: {}", file_path.display());
                        return Html(format!(
                            "<html><body><h1>Upload Successful</h1><p>File '{}' saved.</p><a href=\"/\">Back</a></body></html>",
                            file_name
                        ));
                    }
                }
                Err(e) => {
                    error!("Failed to create file: {}", e);
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
