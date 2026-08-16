//! Relay Token Management
//! 
//! Issues HMAC-signed, time-limited tokens for relay access.
//! Tokens are single-use and bound to specific session/relay pairs.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayToken {
    pub session_id: String,
    pub relay_id: String,
    pub client_device: String,
    pub host_device: String,
    pub expires_at: i64, // Unix timestamp
    pub nonce: String,   // Prevents replay
}

impl RelayToken {
    /// Generate a new relay token
    pub fn new(
        session_id: &str,
        relay_id: &str,
        client_device: &str,
        host_device: &str,
        ttl_secs: u64,
    ) -> Self {
        use uuid::Uuid;
        
        Self {
            session_id: session_id.to_string(),
            relay_id: relay_id.to_string(),
            client_device: client_device.to_string(),
            host_device: host_device.to_string(),
            expires_at: (Utc::now() + Duration::seconds(ttl_secs as i64)).timestamp(),
            nonce: Uuid::new_v4().to_string(),
        }
    }
    
    /// Serialize and sign the token
    pub fn sign(&self, secret: &[u8]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(self)?;
        
        let mut mac = HmacSha256::new_from_slice(secret)
            .map_err(|_| "Invalid HMAC key length")?;
        mac.update(json.as_bytes());
        let signature = mac.finalize().into_bytes();
        
        // Format: base64(token_json).base64(signature)
        let token_b64 = base64_url::encode(json.as_bytes());
        let sig_b64 = base64_url::encode(&signature);
        
        Ok(format!("{}.{}", token_b64, sig_b64))
    }
    
    /// Verify and deserialize a token
    pub fn verify(token_str: &str, secret: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let parts: Vec<&str> = token_str.split('.').collect();
        if parts.len() != 2 {
            return Err("Invalid token format".into());
        }
        
        let token_bytes = base64_url::decode(parts[0])?;
        let sig_bytes = base64_url::decode(parts[1])?;
        
        // Verify signature
        let mut mac = HmacSha256::new_from_slice(secret)
            .map_err(|_| "Invalid HMAC key length")?;
        mac.update(&token_bytes);
        mac.verify_slice(&sig_bytes)
            .map_err(|_| "Invalid token signature")?;
        
        // Parse token
        let token: Self = serde_json::from_slice(&token_bytes)?;
        
        // Check expiration
        if token.expires_at < Utc::now().timestamp() {
            return Err("Token expired".into());
        }
        
        Ok(token)
    }
}

// Use URL-safe base64 without padding
mod base64_url {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    
    pub fn encode(data: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(data)
    }
    
    pub fn decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
        URL_SAFE_NO_PAD.decode(s)
    }
}
