//! HTTP Server Service

use crate::{Result, ServiceError};
use axum::{
    body::{Body, HttpBody},
    extract::{ConnectInfo, Multipart, Request, State},
    middleware::{self, Next},
    response::{Html, Response},
    routing::post,
    Router,
};
use rabbit_models::http::{HttpAccessLog, HttpServerConfig, HttpServerState};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;
use tracing::{error, info};

/// HTTP server service
pub struct HttpService {
    config: Arc<RwLock<HttpServerConfig>>,
    state: Arc<RwLock<HttpServerState>>,
    logs: Arc<RwLock<Vec<HttpAccessLog>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    server_handle: Option<JoinHandle<()>>,
}

impl HttpService {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(HttpServerConfig::default())),
            state: Arc::new(RwLock::new(HttpServerState::Stopped)),
            logs: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx: None,
            server_handle: None,
        }
    }

    /// Initialize with configuration
    pub async fn init(&mut self, config: HttpServerConfig) -> Result<()> {
        *self.config.write().await = config;
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

        let config = self.config.read().await.clone();
        if !config.enabled {
            *self.state.write().await = HttpServerState::Stopped;
            return Ok(());
        }

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

        let state = Arc::clone(&self.state);
        let logs = Arc::clone(&self.logs);

        let handle = tokio::spawn(async move {
            // Build axum router with access logging and file upload
            let logs_clone = Arc::clone(&logs);
            let root_clone = root.clone();
            
            let app = Router::new()
                .route("/upload", post(upload_handler))
                .with_state(root_clone)
                .fallback_service(
                    ServeDir::new(&root)
                        .append_index_html_on_directories(true)
                )
                .layer(middleware::from_fn(move |request: Request, next: Next| {
                    let logs = Arc::clone(&logs_clone);
                    async move {
                        access_log_middleware(request, next, logs).await
                    }
                }));

            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to bind HTTP server: {}", e);
                    *state.write().await = HttpServerState::Error;
                    return;
                }
            };

            info!("HTTP server listening on {}", addr);
            *state.write().await = HttpServerState::Running;

            // Run server with shutdown signal
            let server = axum::serve(listener, app);

            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!("HTTP server error: {}", e);
                        *state.write().await = HttpServerState::Error;
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("HTTP server shutting down");
                }
            }

            *state.write().await = HttpServerState::Stopped;
        });

        self.server_handle = Some(handle);
        Ok(())
    }

    /// Stop the HTTP server
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

    /// Get configuration
    pub async fn get_config(&self) -> HttpServerConfig {
        self.config.read().await.clone()
    }

    /// Update configuration (requires restart)
    pub async fn update_config(&mut self, config: HttpServerConfig) -> Result<()> {
        let was_running = *self.state.read().await == HttpServerState::Running;

        if was_running {
            self.stop().await?;
        }

        *self.config.write().await = config;

        if was_running {
            self.start().await?;
        }

        Ok(())
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
    State(root): State<PathBuf>,
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
