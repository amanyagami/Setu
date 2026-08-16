//! Capability Model per Setu v5 §4
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HostCapabilities {
    pub capture: CaptureCaps,
    pub input: InputCaps,
    pub persist: PersistCaps,
    pub trans: TransportCaps,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaptureCaps {
    pub pre_login: Vec<String>,
    pub locked: Vec<String>,
    pub secure_desktop: Vec<String>,
    pub zero_copy: bool,
    pub dirty_regions: bool,
    pub cursor_metadata: bool,
    pub max_fps: u32,
    pub max_res_width: u32,
    pub max_res_height: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputCaps {
    pub pointer: bool,
    pub keyboard: bool,
    pub secure_attention: bool,
    pub absolute_touch: bool,
    pub relative_pointer: bool,
    pub global_shortcuts: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistCaps {
    pub reboot: bool,
    pub oem_power_exempt: bool,
    pub unattended: bool,
    pub push_wake: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportCaps {
    pub direct: bool,
    pub udp_relay: bool,
    pub masque: bool,
    pub migration: bool,
    pub multipath: bool,
}
