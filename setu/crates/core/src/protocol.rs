//! Protocol Messages per Setu v5 §8
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    CapabilitiesExchange(CapabilitiesMsg),
    SessionControl(SessionControlMsg),
    InputEvent(InputEventMsg),
    Clipboard(ClipboardMsg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesMsg {
    pub host_caps: crate::capabilities::HostCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionControlMsg {
    pub action: SessionAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionAction {
    Start,
    Pause,
    Resume,
    Stop,
    Migrate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEventMsg {
    pub event_type: InputEventType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEventType {
    Pointer,
    Keyboard,
    Touch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardMsg {
    pub data: Vec<u8>,
    pub mime_type: String,
}
