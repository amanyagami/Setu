//! Video Decoder Trait and Implementations

use crate::config::{CodecConfig, VideoCodec};
use crate::encoder::EncoderError;

/// Hardware-accelerated video decoder trait
pub trait VideoDecoder: Send + Sync {
    /// Initialize decoder with configuration
    fn init(config: &CodecConfig) -> Result<Self, DecoderError>
    where
        Self: Sized;

    /// Decode a frame from encoded data
    /// Returns decoded surface (GPU handle or raw buffer)
    fn decode(&mut self, data: &[u8]) -> Result<DecodedFrame, DecoderError>;

    /// Flush decoder (end of stream)
    fn flush(&mut self) -> Result<Option<DecodedFrame>, DecoderError>;

    /// Get decoder latency (encode-to-decode time)
    fn get_latency_ms(&self) -> u32;
}

/// Decoded frame representation
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// Timestamp (media timeline)
    pub pts_us: u64,
    /// Frame dimensions
    pub width: u32,
    pub height: u32,
    /// GPU surface handle (zero-copy) or CPU buffer
    pub surface: FrameSurface,
    /// Keyframe indicator
    pub is_keyframe: bool,
}

/// Frame surface: GPU handle for zero-copy compositing
#[derive(Debug, Clone)]
pub enum FrameSurface {
    /// DMA-BUF file descriptor (Linux)
    DmaBuf { fd: i32, stride: u32 },
    /// IOSurface ID (macOS)
    IOSurface { surface_id: u32 },
    /// ID3D11Texture2D handle (Windows)
    DxgiHandle { handle: u64 },
    /// Android SurfaceTexture
    SurfaceTexture { texture_id: u32 },
    /// Fallback CPU buffer (should be avoided)
    CpuBuffer(Vec<u8>),
}

#[derive(Debug)]
pub struct DecoderError(pub String);

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Decoder error: {}", self.0)
    }
}

impl std::error::Error for DecoderError {}

// ============================================================================
// Hardware Decoder Implementations (Stubs - Phase 2 Integration)
// ============================================================================

/// VAAPI Decoder (Intel/AMD on Linux)
pub struct VaapiDecoder {
    _config: CodecConfig,
    // Real impl: VADisplay, VAContext, VASurfaceID
}

impl VideoDecoder for VaapiDecoder {
    fn init(config: &CodecConfig) -> Result<Self, DecoderError> {
        // TODO: Initialize VAAPI context
        // - vaGetDisplayDRM()
        // - vaCreateConfig()
        // - vaCreateSurfaces()
        Ok(VaapiDecoder { _config: config.clone() })
    }

    fn decode(&mut self, _data: &[u8]) -> Result<DecodedFrame, DecoderError> {
        Err(DecoderError("VAAPI not initialized".into()))
    }

    fn flush(&mut self) -> Result<Option<DecodedFrame>, DecoderError> {
        Ok(None)
    }

    fn get_latency_ms(&self) -> u32 {
        5 // Target: ≤8ms per SLO
    }
}

/// NVDEC Decoder (NVIDIA)
pub struct NvdecDecoder {
    _config: CodecConfig,
    // Real impl: CUcontext, CUvideoparser, CUvideoctxmapper
}

impl VideoDecoder for NvdecDecoder {
    fn init(config: &CodecConfig) -> Result<Self, DecoderError> {
        // TODO: Initialize NVDEC via CUDA Video SDK
        Ok(NvdecDecoder { _config: config.clone() })
    }

    fn decode(&mut self, _data: &[u8]) -> Result<DecodedFrame, DecoderError> {
        Err(DecoderError("NVDEC not initialized".into()))
    }

    fn flush(&mut self) -> Result<Option<DecodedFrame>, DecoderError> {
        Ok(None)
    }

    fn get_latency_ms(&self) -> u32 {
        4 // NVIDIA typically faster
    }
}

/// VideoToolbox Decoder (macOS/iOS)
pub struct VideotoolboxDecoder {
    _config: CodecConfig,
    // Real impl: VTDecompressionSessionRef, CVPixelBufferRef
}

impl VideoDecoder for VideotoolboxDecoder {
    fn init(config: &CodecConfig) -> Result<Self, DecoderError> {
        // TODO: Initialize VideoToolbox session
        // - VTDecompressionSessionCreate()
        // - Configure output callback with CVPixelBuffer
        Ok(VideotoolboxDecoder { _config: config.clone() })
    }

    fn decode(&mut self, _data: &[u8]) -> Result<DecodedFrame, DecoderError> {
        Err(DecoderError("VideoToolbox not initialized".into()))
    }

    fn flush(&mut self) -> Result<Option<DecodedFrame>, DecoderError> {
        Ok(None)
    }

    fn get_latency_ms(&self) -> u32 {
        6 // Apple Silicon very fast
    }
}

/// MediaCodec Decoder (Android)
pub struct MediacodecDecoder {
    _config: CodecConfig,
    // Real impl: AMediaCodec, ANativeWindow, AImageReader
}

impl VideoDecoder for MediacodecDecoder {
    fn init(config: &CodecConfig) -> Result<Self, DecoderError> {
        // TODO: Initialize MediaCodec via NDK
        // - AMediaCodec_createDecoderByType()
        // - Configure format with AFormat
        // - Create input/output buffers
        Ok(MediacodecDecoder { _config: config.clone() })
    }

    fn decode(&mut self, _data: &[u8]) -> Result<DecodedFrame, DecoderError> {
        Err(DecoderError("MediaCodec not initialized".into()))
    }

    fn flush(&mut self) -> Result<Option<DecodedFrame>, DecoderError> {
        Ok(None)
    }

    fn get_latency_ms(&self) -> u32 {
        8 // Target for mobile
    }
}

/// Software fallback decoder (FFmpeg) - Only if HW unavailable
pub struct SwDecoder {
    _config: CodecConfig,
    // Real impl: AVCodecContext, AVFrame, SwsContext
}

impl VideoDecoder for SwDecoder {
    fn init(config: &CodecConfig) -> Result<Self, DecoderError> {
        // WARNING: Software decoding violates latency SLOs for high resolutions
        // Use only as last resort
        Err(DecoderError("Software decoding not supported in Setu v5".into()))
    }

    fn decode(&mut self, _data: &[u8]) -> Result<DecodedFrame, DecoderError> {
        Err(DecoderError("Software decoding disabled".into()))
    }

    fn flush(&mut self) -> Result<Option<DecodedFrame>, DecoderError> {
        Ok(None)
    }

    fn get_latency_ms(&self) -> u32 {
        50 // Too slow for realtime
    }
}
