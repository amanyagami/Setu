//! Server Configuration

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// HTTPS listening address
    pub listen_addr: String,
    
    /// PostgreSQL connection string
    pub database_url: String,
    
    /// Path to TLS certificate (PEM)
    pub tls_cert_path: PathBuf,
    
    /// Path to TLS private key (PEM)
    pub tls_key_path: PathBuf,
    
    /// HMAC secret for relay token signing
    pub relay_hmac_secret: Vec<u8>,
    
    /// Token validity duration (seconds)
    pub token_ttl_secs: u64,
    
    /// OpenTelemetry endpoint (optional)
    pub otel_endpoint: Option<String>,
    
    /// Environment (dev, staging, prod)
    pub environment: String,
}

impl ServerConfig {
    pub fn load_from_env() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::Environment::with_prefix("SETU_CP").separator("__"))
            .build()?;
        
        cfg.try_deserialize()
    }
}
