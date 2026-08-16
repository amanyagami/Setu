//! HTTP/WebSocket Handlers for Signaling

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::state::AppState;
use crate::db;
use crate::relay::RelayToken;
use setu_core::{
    identity::DeviceId,
    protocol::{SignalingMessage, SessionOffer, SessionAnswer},
};

/// Device registration endpoint
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub device_id: String,
    pub owner_token: String, // JWT from SSO provider
    pub public_key: Vec<u8>,
    pub capabilities_hash: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub registered: bool,
    pub device_uuid: String,
}

pub async fn register_device(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // TODO: Validate owner_token with SSO provider (OIDC/OAuth2)
    let owner_id = "temp-user-id"; // Placeholder until SSO integration
    
    let device_id = match DeviceId::from_base64(&req.device_id) {
        Ok(id) => id,
        Err(e) => {
            warn!("Invalid device ID format: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid device ID").into_response();
        }
    };
    
    match db::upsert_device(&state.db, device_id, owner_id, &req.public_key, &req.capabilities_hash).await {
        Ok(uuid) => {
            info!("Device registered: {:?}", device_id);
            (StatusCode::OK, Json(RegisterResponse {
                registered: true,
                device_uuid: uuid.to_string(),
            })).into_response()
        }
        Err(e) => {
            error!("Database error during registration: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Registration failed").into_response()
        }
    }
}

/// Get device public key and capabilities
#[derive(Debug, Serialize)]
pub struct DeviceInfoResponse {
    pub device_id: String,
    pub public_key: String,
    pub is_online: bool,
    pub capabilities_hash: String,
}

pub async fn get_device_info(
    State(state): State<AppState>,
    Path(device_id_str): Path<String>,
) -> impl IntoResponse {
    let device_id = match DeviceId::from_base64(&device_id_str) {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid device ID").into_response(),
    };
    
    match db::get_device(&state.db, &device_id).await {
        Ok(Some(device)) => {
            (StatusCode::OK, Json(DeviceInfoResponse {
                device_id: device.device_id.to_string(),
                public_key: base64::encode(&device.public_key),
                is_online: device.is_online,
                capabilities_hash: device.capabilities_hash,
            })).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Device not found").into_response(),
        Err(e) => {
            error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Lookup failed").into_response()
        }
    }
}

/// Request relay token for T2/T3 fallback
#[derive(Debug, Deserialize)]
pub struct RelayTokenRequest {
    pub session_id: String,
    pub client_device: String,
    pub host_device: String,
    pub preferred_tier: u8, // 2 or 3
}

#[derive(Debug, Serialize)]
pub struct RelayTokenResponse {
    pub token: String,
    pub relay_endpoint: String,
    pub tier: u8,
}

pub async fn request_relay_token(
    State(state): State<AppState>,
    Json(req): Json<RelayTokenRequest>,
) -> impl IntoResponse {
    // Select relay based on tier preference and load
    let relay_id = format!("relay-{}", req.preferred_tier);
    let relay_endpoint = match req.preferred_tier {
        2 => "udp://relay-t2.setu.io:3478".to_string(),
        3 => "https://relay-t3.setu.io:443".to_string(),
        _ => return (StatusCode::BAD_REQUEST, "Invalid tier").into_response(),
    };
    
    let token = match RelayToken::new(
        &req.session_id,
        &relay_id,
        &req.client_device,
        &req.host_device,
        state.config.token_ttl_secs,
    ).sign(&state.config.relay_hmac_secret) {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to sign relay token: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Token generation failed").into_response();
        }
    };
    
    info!("Issued relay token for session {} (tier {})", req.session_id, req.preferred_tier);
    
    (StatusCode::OK, Json(RelayTokenResponse {
        token,
        relay_endpoint,
        tier: req.preferred_tier,
    })).into_response()
}

/// WebSocket upgrade handler for signaling
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> Response {
    ws.on_upgrade(|_socket| async move {
        // TODO: Implement WebSocket message loop for real-time signaling
        // - Handle SessionOffer from initiator
        // - Forward to target device
        // - Handle SessionAnswer from target
        // - Forward back to initiator
        // - Connection established, server steps out of media path
        info!("WebSocket connection established (signaling placeholder)");
    })
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// Helper for base64 encoding
mod base64 {
    use base64::{Engine, engine::general_purpose::STANDARD};
    
    pub fn encode(data: &[u8]) -> String {
        STANDARD.encode(data)
    }
}
