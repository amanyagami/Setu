//! Setu Control Plane Server
//! 
//! Provides:
//! - HTTPS/WSS Signaling Endpoint
//! - Device Identity Registry (PostgreSQL)
//! - Key Distribution (Public keys only, never private)
//! - Relay Token Issuance (HMAC-signed, time-limited)
//! - Presence Tracking
//!
//! SECURITY INVARIANT: Control plane NEVER sees media payloads or session keys.
//! It only facilitates introduction and key exchange.

pub mod config;
pub mod db;
pub mod handlers;
pub mod relay;
pub mod server;
pub mod state;

pub use server::run_server;
pub use config::ServerConfig;
