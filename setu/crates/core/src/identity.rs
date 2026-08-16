//! Identity Model per Setu v5 §3
use blake3::Hash;

#[derive(Debug, Clone)]
pub struct HostId {
    pub hash: Hash,
    pub platform: String,
}

impl HostId {
    pub fn new(platform: &str, machine_id: &[u8]) -> Self {
        let hash = blake3::hash(machine_id);
        Self { hash, platform: platform.to_string() }
    }
}

#[derive(Debug, Clone)]
pub struct UserId(pub String);
