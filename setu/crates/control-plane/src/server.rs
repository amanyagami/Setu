//! Control Plane Server Implementation

use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tracing::{info, error};
use std::sync::Arc;

use crate::config::ServerConfig;
use crate::state::AppState;
use crate::handlers;
use crate::db;

/// Run the control plane server
pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize database connection
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await?;
    
    // Run migrations
    db::init_db(&pool).await?;
    info!("Database initialized");
    
    // Create shared state
    let state = AppState {
        db: pool,
        config: Arc::new(config.clone()),
    };
    
    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/v1/devices/register", post(handlers::register_device))
        .route("/api/v1/devices/:device_id", get(handlers::get_device_info))
        .route("/api/v1/relay/token", post(handlers::request_relay_token))
        .route("/ws/signaling", get(handlers::ws_handler))
        .with_state(state);
    
    // Setup TLS
    let cert_pem = std::fs::read(&config.tls_cert_path)?;
    let key_pem = std::fs::read(&config.tls_key_path)?;
    
    let cert = rustls_pemfile::certs(&mut std::io::BufReader::new(std::io::Cursor::new(cert_pem)))?;
    let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(std::io::Cursor::new(key_pem)))?
        .ok_or("No private key found")?;
    
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            cert.into_iter().map(rustls::Certificate).collect(),
            rustls::PrivateKey(key.secret_contents().to_vec()),
        )?;
    
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    
    // Bind and listen
    let listener = TcpListener::bind(&config.listen_addr).await?;
    info!("Control plane listening on {}", config.listen_addr);
    
    // Accept loop
    loop {
        let (stream, addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let app = app.clone();
        
        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    let hyper_service = hyper_util::service::TowerToHyperService::new(app);
                    if let Err(e) = hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                        .serve_connection(hyper_util::server::conn::http1::Connection::new(tls_stream), hyper_service)
                        .await
                    {
                        error!("Error serving connection from {}: {}", addr, e);
                    }
                }
                Err(e) => {
                    error!("TLS handshake failed for {}: {}", addr, e);
                }
            }
        });
    }
}
