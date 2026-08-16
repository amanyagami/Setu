//! Video Encoder Traits and Implementations
use crate::config::{CodecConfig, VideoCodec, ChromaSubsampling};

pub trait VideoEncoder: Send + Sync {
    fn init(config: &CodecConfig) -> Result<Self, EncoderError> where Self: Sized;
    fn encode(&mut self, surface: &[u8]) -> Result<Vec<u8>, EncoderError>;
    fn set_bitrate(&mut self, bps: u32) -> Result<(), EncoderError>;
    fn request_keyframe(&mut self) -> Result<(), EncoderError>;
}

#[derive(Debug)]
pub struct EncoderError(pub String);

impl std::fmt::Display for EncoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encoder error: {}", self.0)
    }
}
