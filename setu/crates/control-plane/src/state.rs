//! Application State (Shared across handlers)

use axum::extract::FromRef;
use sqlx::PgPool;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<crate::config::ServerConfig>,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

impl FromRef<AppState> for Arc<crate::config::ServerConfig> {
    fn from_ref(input: &AppState) -> Self {
        input.config.clone()
    }
}
