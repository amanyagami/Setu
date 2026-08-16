//! Codec Configuration per Setu v5 §6
#[derive(Debug, Clone)]
pub struct CodecConfig {
    pub codec: VideoCodec,
    pub profile: u32,
    pub level: u32,
    pub bit_depth: u8,
    pub chroma: ChromaSubsampling,
    pub screen_content_tools: bool,
    pub max_width: u32,
    pub max_height: u32,
    pub max_fps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec { H264, Hevc, Av1 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaSubsampling { YUV420, YUV422, YUV444 }
