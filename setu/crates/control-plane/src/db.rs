//! Database Layer: Device Registry & Session Metadata

use sqlx::{PgPool, Error};
use uuid::Uuid;
use setu_core::identity::DeviceId;

/// Device record in the registry
#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub id: Uuid,
    pub device_id: DeviceId,
    pub owner_id: String, // User identity from SSO
    pub public_key: Vec<u8>, // Ed25519 + ML-DSA-65 public key
    pub capabilities_hash: String, // Hash of advertised capabilities
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub is_online: bool,
}

/// Register a new device or update existing
pub async fn upsert_device(
    pool: &PgPool,
    device_id: DeviceId,
    owner_id: &str,
    public_key: &[u8],
    capabilities_hash: &str,
) -> Result<Uuid, Error> {
    let row = sqlx::query_as::<_, DeviceRecord>(
        r#"
        INSERT INTO devices (device_id, owner_id, public_key, capabilities_hash, last_seen, is_online)
        VALUES ($1, $2, $3, $4, NOW(), TRUE)
        ON CONFLICT (device_id) 
        DO UPDATE SET 
            owner_id = EXCLUDED.owner_id,
            public_key = EXCLUDED.public_key,
            capabilities_hash = EXCLUDED.capabilities_hash,
            last_seen = NOW(),
            is_online = TRUE
        RETURNING *
        "#
    )
    .bind(device_id.to_string())
    .bind(owner_id)
    .bind(public_key)
    .bind(capabilities_hash)
    .fetch_one(pool)
    .await?;
    
    Ok(row.id)
}

/// Get device by ID
pub async fn get_device(pool: &PgPool, device_id: &DeviceId) -> Result<Option<DeviceRecord>, Error> {
    sqlx::query_as::<_, DeviceRecord>(
        "SELECT * FROM devices WHERE device_id = $1"
    )
    .bind(device_id.to_string())
    .fetch_optional(pool)
    .await
}

/// Mark device offline
pub async fn set_device_offline(pool: &PgPool, device_id: &DeviceId) -> Result<(), Error> {
    sqlx::query(
        "UPDATE devices SET is_online = FALSE, last_seen = NOW() WHERE device_id = $1"
    )
    .bind(device_id.to_string())
    .execute(pool)
    .await?;
    Ok(())
}

/// Initialize database schema
pub async fn init_db(pool: &PgPool) -> Result<(), Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS devices (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            device_id TEXT UNIQUE NOT NULL,
            owner_id TEXT NOT NULL,
            public_key BYTEA NOT NULL,
            capabilities_hash TEXT NOT NULL,
            last_seen TIMESTAMPTZ NOT NULL,
            is_online BOOLEAN NOT NULL DEFAULT FALSE
        );
        
        CREATE INDEX IF NOT EXISTS idx_devices_owner ON devices(owner_id);
        CREATE INDEX IF NOT EXISTS idx_devices_online ON devices(is_online) WHERE is_online = TRUE;
        "#
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
