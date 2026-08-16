//! Common Error Types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SetuError {
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Capture error: {0}")]
    Capture(String),
    #[error("Codec error: {0}")]
    Codec(String),
    #[error("Session error: {0}")]
    Session(String),
}
