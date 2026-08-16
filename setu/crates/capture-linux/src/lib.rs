//! Linux Screen Capture Implementation
//!
//! Probe-ordered fallback chain per Setu v5 §5.1:
//! 1. DRM/KMS Scanout (Zero-Copy via dma-buf + EGL/Vulkan import) [HYP→S1]
//! 2. PipeWire/XDG-Portal (with restore_token persistence) [STD]
//! 3. X11 Fallback (including NVIDIA pre-login greeter) [STD]
//!
//! Re-probed on compositor/driver events.

use setu_core::capabilities::{CaptureCaps, HostCapabilities};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{info, warn, debug};

pub mod drm_kms;
pub mod pipewire;
pub mod x11;

/// Linux-specific capture error
#[derive(Debug, Error)]
pub enum LinuxCaptureError {
    #[error("DRM/KMS capture failed: {0}")]
    DrmKmsFailed(String),
    
    #[error("PipeWire capture failed: {0}")]
    PipewireFailed(String),
    
    #[error("X11 capture failed: {0}")]
    X11Failed(String),
    
    #[error("No available capture method")]
    NoCaptureMethod,
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Unified Linux screen capturer trait
#[async_trait::async_trait]
pub trait LinuxScreenCapture: Send + Sync {
    /// Start capture stream
    async fn start(&mut self) -> Result<(), LinuxCaptureError>;
    
    /// Stop capture stream
    async fn stop(&mut self);
    
    /// Get next frame (DMA-BUF fd or CPU buffer)
    async fn next_frame(&mut self) -> Result<CaptureFrame, LinuxCaptureError>;
    
    /// Get current capture capabilities
    fn get_capabilities(&self) -> CaptureCaps;
}

/// Captured frame representation
#[derive(Debug)]
pub struct CaptureFrame {
    /// Timestamp (monotonic, host timeline)
    pub timestamp_us: u64,
    /// Frame dimensions
    pub width: u32,
    pub height: u32,
    /// Zero-copy DMA-BUF fd (preferred) or CPU buffer
    pub surface: FrameSurface,
    /// Dirty regions (tile-based encoding hint)
    pub dirty_regions: Option<Vec<DirtyRegion>>,
    /// Cursor metadata (position, shape change)
    pub cursor_state: Option<CursorState>,
}

/// Frame surface: zero-copy handle or fallback buffer
#[derive(Debug)]
pub enum FrameSurface {
    /// DMA-BUF file descriptor (zero-copy path)
    DmaBuf {
        fd: i32,
        stride: u32,
        format: u32, // DRM_FORMAT_*
        modifier: u64, // DRM_FORMAT_MOD_*
    },
    /// CPU buffer (fallback, should be avoided)
    CpuBuffer {
        data: Vec<u8>,
        stride: u32,
        format: PixelFormat,
    },
}

/// Pixel format for CPU buffers
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    NV12,
    BGRA,
    RGBA,
}

/// Dirty region for tile-based encoding
#[derive(Debug, Clone)]
pub struct DirtyRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Cursor state metadata
#[derive(Debug, Clone)]
pub struct CursorState {
    pub x: i32,
    pub y: i32,
    pub shape_changed: bool,
    pub visible: bool,
}

// ============================================================================
// Capability Probing
// ============================================================================

/// Probe Linux capture capabilities at startup
pub async fn probe_capabilities() -> HostCapabilities {
    info!("Probing Linux capture capabilities...");
    
    let mut capture_caps = CaptureCaps::default();
    
    // Check DRM/KMS availability
    if drm_kms::is_available().await {
        info!("DRM/KMS scanout available");
        capture_caps.drm_kms_supported = true;
        capture_caps.zero_copy = true;
        capture_caps.dirty_regions = true;
        // TODO: Query max resolution/FPS from GPU
        capture_caps.max_fps = 144;
        capture_caps.max_res_width = 3840;
        capture_caps.max_res_height = 2160;
    } else {
        warn!("DRM/KMS not available (NVIDIA proprietary?)");
    }
    
    // Check PipeWire availability
    if pipewire::is_available().await {
        info!("PipeWire portal available");
        capture_caps.pipewire_supported = true;
        // PipeWire may support restore_token for reconnection without prompt
        capture_caps.persisted_permission = pipewire::has_restore_token().await;
    } else {
        warn!("PipeWire not available");
    }
    
    // X11 always available as fallback
    if x11::is_available().await {
        info!("X11 fallback available");
        capture_caps.x11_supported = true;
    }
    
    HostCapabilities {
        capture: capture_caps,
        input: probe_input_capabilities(),
        persist: probe_persist_capabilities(),
        transport: probe_transport_capabilities(),
    }
}

/// Probe input capabilities (uinput)
fn probe_input_capabilities() -> setu_core::capabilities::InputCaps {
    use setu_core::capabilities::InputCaps;
    
    // Check /dev/uinput availability
    let has_uinput = std::path::Path::new("/dev/uinput").exists();
    
    InputCaps {
        pointer: has_uinput,
        keyboard: has_uinput,
        secure_attention: false, // Requires root/privileged helper
        absolute_touch: has_uinput,
        relative_pointer: has_uinput,
        global_shortcuts: false, // Requires Wayland compositor integration
    }
}

/// Probe persistence capabilities
fn probe_persist_capabilities() -> setu_core::capabilities::PersistCaps {
    use setu_core::capabilities::PersistCaps;
    
    PersistCaps {
        reboot: false, // Requires systemd service setup
        oem_power_exempt: false, // Requires OEM certification
        unattended: std::env::var("SETU_UNATTENDED").is_ok(),
        push_wake: vec![], // Requires MDM/root
    }
}

/// Probe transport capabilities
fn probe_transport_capabilities() -> setu_core::capabilities::TransportCaps {
    use setu_core::capabilities::TransportCaps;
    
    TransportCaps {
        direct: true,
        udp_relay: true,
        masque: true,
        migration: true,
        multipath: false, // draft-ietf-quic-multipath not yet stable
    }
}

// ============================================================================
// Capture Method Selection
// ============================================================================

/// Create optimal capturer based on probed capabilities
pub async fn create_capturer() -> Result<Box<dyn LinuxScreenCapture>, LinuxCaptureError> {
    let caps = probe_capabilities().await;
    
    // Priority 1: DRM/KMS (zero-copy, lowest latency)
    if caps.capture.drm_kms_supported {
        info!("Selecting DRM/KMS capture path");
        return drm_kms::DrmKmsCapturer::new()
            .await
            .map(|c| Box::new(c) as Box<dyn LinuxScreenCapture>)
            .map_err(LinuxCaptureError::DrmKmsFailed);
    }
    
    // Priority 2: PipeWire (good latency, user consent)
    if caps.capture.pipewire_supported {
        info!("Selecting PipeWire capture path");
        return pipewire::PipewireCapturer::new()
            .await
            .map(|c| Box::new(c) as Box<dyn LinuxScreenCapture>)
            .map_err(LinuxCaptureError::PipewireFailed);
    }
    
    // Priority 3: X11 (fallback, higher CPU usage)
    if caps.capture.x11_supported {
        info!("Selecting X11 fallback capture path");
        return x11::X11Capturer::new()
            .await
            .map(|c| Box::new(c) as Box<dyn LinuxScreenCapture>)
            .map_err(LinuxCaptureError::X11Failed);
    }
    
    Err(LinuxCaptureError::NoCaptureMethod)
}
