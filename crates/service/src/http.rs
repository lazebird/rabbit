//! HTTP Server Service

use crate::{
    ui_channel::{Module, UiData},
    Result, ServiceError, ServiceUpdateResult,
};
use axum::{
    body::HttpBody,
    extract::{Multipart, Request},
    middleware::Next,
    response::{Html, IntoResponse, Response},
    routing::post,
    Router,
};
use rabbit_config::{get_bool, get_integer};
use schema::config::keys;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info};

/// Static MIME types for video files
const VIDEO_MIME_TYPES: &[(&str, &str)] = &[
    (".mp4", "video/mp4"),
    (".webm", "video/webm"),
    (".ogg", "video/ogg"),
    (".ogv", "video/ogg"),
    (".avi", "video/x-msvideo"),
    (".mov", "video/quicktime"),
    (".wmv", "video/x-ms-wmv"),
    (".flv", "video/x-flv"),
    (".mkv", "video/x-matroska"),
    (".m4v", "video/x-m4v"),
];

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
    pub dirs: Vec<String>,
    pub auto_index: bool,
    pub video_play: bool,
    pub log_path: Option<String>,
}

impl ServerConfig {
    fn from_platform() -> Self {
        Self {
            port: get_integer("http", keys::http::PORT).unwrap_or(8000) as u16,
            dirs: rabbit_config::get_array("http", keys::http::DIRS).unwrap_or_default(),
            auto_index: get_bool("http", keys::http::AUTO_INDEX).unwrap_or(true),
            video_play: get_bool("http", keys::http::VIDEO_PLAY).unwrap_or(true),
            log_path: rabbit_config::get_string("http", keys::http::LOG),
        }
    }
}

#[derive(Clone)]
struct HttpFallbackConfig {
    dirs: Vec<PathBuf>,
    auto_index: bool,
}

const PATH_ENCODE_SET: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'/')
    .remove(b'.')
    .remove(b'-')
    .remove(b'_')
    .remove(b'~');

async fn file_fallback(
    uri: axum::http::Uri,
    axum::extract::Extension(config): axum::extract::Extension<HttpFallbackConfig>,
) -> axum::response::Response {
    use percent_encoding::percent_decode_str;

    let relative = uri.path().trim_start_matches('/');
    let decoded = percent_decode_str(relative).decode_utf8_lossy();

    // Root path: show listing of all configured dirs
    if relative.is_empty() {
        if !config.auto_index {
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .body(axum::body::Body::empty())
                .unwrap();
        }

        let mut items = Vec::new();
        for dir_path in &config.dirs {
            let name = dir_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| dir_path.to_string_lossy().to_string());

            if dir_path.is_dir() {
                let encoded = percent_encoding::utf8_percent_encode(&name, PATH_ENCODE_SET);
                items.push(format!(
                    "<tr><td><a href=\"{}/\">{}/</a></td><td></td></tr>",
                    encoded, name
                ));
            } else if dir_path.is_file() {
                let size = std::fs::metadata(dir_path)
                    .map(|m| format_size(m.len()))
                    .unwrap_or_default();
                let encoded = percent_encoding::utf8_percent_encode(&name, PATH_ENCODE_SET);
                items.push(format!(
                    "<tr><td><a href=\"{}\">{}</a></td><td>{}</td></tr>",
                    encoded, name, size
                ));
            }
        }

        let html = format!(
            "<!DOCTYPE html><html><head><title>Index of /</title></head><body><h1>Index of /</h1><table><tr><th>Name</th><th>Size</th></tr>{}</table></body></html>",
            items.join("")
        );
        return axum::response::Response::builder()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(axum::body::Body::from(html))
            .unwrap();
    }

    // Sub-path: find which configured dir matches
    let first_segment = decoded.split('/').next().unwrap_or("");
    let matched_dir = config.dirs.iter().find(|d| {
        d.file_name()
            .map(|n| n.to_string_lossy() == first_segment)
            .unwrap_or(false)
    });

    let Some(base_dir) = matched_dir else {
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .body(axum::body::Body::empty())
            .unwrap();
    };

    // If matched item is a file and path is exactly that file, serve it
    if base_dir.is_file() && decoded == first_segment {
        match tokio::fs::read(base_dir).await {
            Ok(content) => {
                let mime = path2mime(first_segment);
                return axum::response::Response::builder()
                    .header("Content-Type", mime)
                    .body(axum::body::Body::from(content))
                    .unwrap();
            }
            Err(_) => {
                return axum::response::Response::builder()
                    .status(axum::http::StatusCode::NOT_FOUND)
                    .body(axum::body::Body::empty())
                    .unwrap();
            }
        }
    }

    // If matched item is a directory, serve from it
    if base_dir.is_dir() {
        let sub_path = decoded.strip_prefix(first_segment).unwrap_or(&decoded);
        let sub_path = sub_path.trim_start_matches('/');
        let full_path = base_dir.join(sub_path);

        // Try index.html first
        let index_path = if full_path.is_dir() {
            full_path.join("index.html")
        } else {
            full_path.with_file_name("index.html")
        };

        if full_path.is_dir() {
            if index_path.exists() {
                if let Ok(content) = tokio::fs::read(&index_path).await {
                    return axum::response::Response::builder()
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(axum::body::Body::from(content))
                        .unwrap();
                }
            }

            // Directory listing
            if config.auto_index {
                if let Ok(mut entries) = tokio::fs::read_dir(&full_path).await {
                    let display_path = format!("/{}/", decoded.trim_end_matches('/'));
                    let mut items = Vec::new();

                    if !sub_path.is_empty() {
                        items.push("<tr><td><a href=\"..\">..</a></td><td></td></tr>".to_string());
                    }

                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let metadata = entry.metadata().await;
                        let (size, is_dir) = match metadata {
                            Ok(m) => {
                                if m.is_dir() { (String::new(), true) }
                                else { (format_size(m.len()), false) }
                            }
                            Err(_) => (String::new(), false),
                        };
                        let trailing = if is_dir { "/" } else { "" };
                        let encoded = percent_encoding::utf8_percent_encode(&name, PATH_ENCODE_SET);
                        items.push(format!(
                            "<tr><td><a href=\"{}{}{}\">{}{}</a></td><td>{}</td></tr>",
                            display_path, encoded, trailing, name, trailing, size
                        ));
                    }

                    let html = format!(
                        "<!DOCTYPE html><html><head><title>Index of {}</title></head><body><h1>Index of {}</h1><table><tr><th>Name</th><th>Size</th></tr>{}</table></body></html>",
                        display_path, display_path, items.join("")
                    );
                    return axum::response::Response::builder()
                        .header("Content-Type", "text/html; charset=utf-8")
                        .body(axum::body::Body::from(html))
                        .unwrap();
                }
            }

            return axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .body(axum::body::Body::empty())
                .unwrap();
        }

        // Serve file
        if full_path.is_file() {
            if let Ok(content) = tokio::fs::read(&full_path).await {
                let mime = path2mime(&decoded);
                return axum::response::Response::builder()
                    .header("Content-Type", mime)
                    .body(axum::body::Body::from(content))
                    .unwrap();
            }
        }
    }

    axum::response::Response::builder()
        .status(axum::http::StatusCode::NOT_FOUND)
        .body(axum::body::Body::empty())
        .unwrap()
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
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

    /// 内部启动 HTTP 服务器
    async fn start(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        if *state != HttpServerState::Stopped {
            return Err(ServiceError::AlreadyRunning);
        }
        *state = HttpServerState::Starting;
        drop(state);

        // Pull configuration directly from platform cache
        let config = ServerConfig::from_platform();

        let addr: SocketAddr = format!("0.0.0.0:{}", config.port).parse().map_err(|e| ServiceError::Config(format!("Invalid address: {}", e)))?;

        // Validate all dirs exist
        let dirs: Vec<PathBuf> = config.dirs.iter().map(PathBuf::from).collect();
        for d in &dirs {
            if !d.exists() {
                return Err(ServiceError::Config(format!("Path does not exist: {}", d.display())));
            }
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
                .fallback(file_fallback)
                .layer(axum::extract::Extension(HttpFallbackConfig { dirs, auto_index }))
                .layer(axum::middleware::from_fn(move |req: Request, next: Next| {
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
                }));

            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind HTTP server: {}", e);
                    *state_arc.write().await = HttpServerState::Stopped;
                    let _ = startup_tx.send(Err(ServiceError::Other(format!("Failed to bind to port {}: {}", config.port, e))));
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

    /// 公开接口：启停切换，会发状态通告
    pub async fn update(&mut self) -> ServiceUpdateResult {
        let state = *self.state.read().await;
        match state {
            HttpServerState::Running => match self.stop().await {
                Ok(()) => ServiceUpdateResult::Stopped("HTTP server stopped".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to stop: {}", e)),
            },
            HttpServerState::Stopped => match self.start().await {
                Ok(()) => ServiceUpdateResult::Started("HTTP server started".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
            },
            _ => ServiceUpdateResult::NoChange,
        }
    }

    /// 程序退出时调用，销毁资源，不发状态通告
    pub async fn destroy(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        if let Some(handle) = self.server_handle.take() {
            let _ = handle.await;
        }
        *self.state.write().await = HttpServerState::Stopped;
        info!("HTTP server destroyed");
        Ok(())
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

#[async_trait::async_trait]
impl crate::Service for HttpService {
    async fn update(&mut self) -> crate::ServiceUpdateResult {
        self.update().await
    }

    async fn destroy(&mut self) -> crate::Result<()> {
        self.destroy().await
    }

    async fn send(&self, data: crate::UiData) {
        self.send(data).await;
    }
}

impl Default for HttpService {
    fn default() -> Self {
        Self::new()
    }
}

fn path2mime(path: &str) -> &'static str {
    if let Some(filename) = path.rsplit('/').next() {
        if let Some(dot_pos) = filename.rfind('.') {
            let ext = &filename[dot_pos..].to_lowercase();
            for &(ext_key, mime) in VIDEO_MIME_TYPES {
                if ext == ext_key {
                    return mime;
                }
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

async fn access_log_middleware(request: Request, next: Next, ui_tx: Option<mpsc::Sender<UiData>>) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let status = response.status();
    let bytes_sent = response.body().size_hint().lower();

    let log_msg = format!(
        "{} {} {} - {} bytes ({}ms)",
        method,
        uri.path(),
        status.as_u16(),
        bytes_sent,
        duration.as_millis()
    );

    if let Some(ref ui_tx) = ui_tx {
        let _ = ui_tx.send(UiData::Log(Module::Http, log_msg.clone())).await;
    }

    response
}

async fn upload_handler(axum::extract::Extension(config): axum::extract::Extension<HttpFallbackConfig>, mut multipart: Multipart) -> Html<String> {
    let upload_dir = config.dirs.first().cloned().unwrap_or_else(|| PathBuf::from("."));
    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("unknown").to_string();
        let file_name = field.file_name().unwrap_or("unnamed").to_string();
        if name == "file" {
            let file_path = upload_dir.join(&file_name);
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
                        return Html(format!("<html><body><h1>Upload Successful</h1><p>File '{}' saved.</p><a href=\"/\">Back</a></body></html>", file_name));
                    }
                }
                Err(e) => {
                    return Html(format!("<html><body><h1>Upload Failed</h1><p>Failed to create file: {}</p><a href=\"/\">Back</a></body></html>", e));
                }
            }
        }
    }
    Html("<html><body><h1>No file received</h1><a href=\"/\">Back</a></body></html>".to_string())
}
