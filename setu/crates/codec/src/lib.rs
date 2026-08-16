//! Video Codec Pipeline: H.264/HEVC/AV1 with damage-aware tiling
pub mod encoder;
pub mod decoder;
pub mod config;

// Re-export main traits
pub use encoder::{VideoEncoder, EncoderError};
pub use decoder::{VideoDecoder, DecoderError, DecodedFrame, FrameSurface};
pub use config::{CodecConfig, VideoCodec, ChromaSubsampling};
