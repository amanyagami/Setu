//! X11 Fallback Capture
//!
//! X11 screen capture using XCB/Xlib.
//! Used as fallback when DRM/KMS and PipeWire are unavailable.
//! Also used for NVIDIA pre-login greeter (forced to X11 via udev quirk).
//!
//! [STD]: Universal fallback, higher CPU usage

use super::{FrameSurface, CaptureFrame, LinuxCaptureError, PixelFormat};
use setu_core::capabilities::CaptureCaps;
use tracing::{info, warn, debug};

/// Check if X11 is available
pub async fn is_available() -> bool {
    // Check DISPLAY environment variable
    std::env::var("DISPLAY").is_ok()
}

/// X11 capturer instance
pub struct X11Capturer {
    /// X11 connection
    _connection: u64, // Placeholder for *xcb_connection_t
    /// Root window
    _root_window: u32,
    /// Captured window geometry
    _width: u32,
    _height: u32,
}

impl X11Capturer {
    /// Create new X11 capturer
    pub async fn new() -> Result<Self, String> {
        info!("Initializing X11 capturer...");
        
        // TODO: Real implementation
        // 1. xcb_connect() or XOpenDisplay()
        // 2. Get root window
        // 3. Query window geometry
        
        Ok(X11Capturer {
            _connection: 0,
            _root_window: 0,
            _width: 1920,
            _height: 1080,
        })
    }
    
    /// Get capabilities
    pub fn get_caps(&self) -> CaptureCaps {
        CaptureCaps {
            x11_supported: true,
            zero_copy: false, // X11 requires CPU copy
            dirty_regions: false,
            cursor_metadata: false,
            max_fps: 30, // Limited by CPU encoding
            max_res_width: 1920,
            max_res_height: 1080,
            ..Default::default()
        }
    }
}

#[async_trait::async_trait]
impl super::LinuxScreenCapture for X11Capturer {
    async fn start(&mut self) -> Result<(), LinuxCaptureError> {
        info!("Starting X11 capture stream");
        
        // TODO: Real implementation
        // 1. Create X11 pixmap
        // 2. Set up damage extension for dirty regions (optional)
        
        Ok(())
    }
    
    async fn stop(&mut self) {
        info!("Stopping X11 capture");
        // TODO: xcb_disconnect()
    }
    
    async fn next_frame(&mut self) -> Result<CaptureFrame, LinuxCaptureError> {
        // TODO: Real implementation
        // 1. XGetImage() or xcb_get_image()
        // 2. Copy pixel data to buffer
        // 3. Return FrameSurface::CpuBuffer
        
        // Stub response with synthetic data
        let mut data = vec![0u8; (self._width * self._height * 4) as usize];
        
        Ok(CaptureFrame {
            timestamp_us: 0,
            width: self._width,
            height: self._height,
            surface: FrameSurface::CpuBuffer {
                data,
                stride: self._width * 4,
                format: PixelFormat::BGRA,
            },
            dirty_regions: None,
            cursor_state: None,
        })
    }
    
    fn get_capabilities(&self) -> CaptureCaps {
        self.get_caps()
    }
}
