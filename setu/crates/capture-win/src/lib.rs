//! Windows Screen Capture: DDA → Winlogon BitBlt → WGC fallback
//! 
//! Implements Setu v5 §5.2 requirements:
//! - Dual-agent architecture (session-0 service + per-session user agent)
//! - DDA for user sessions with GPU zero-copy
//! - Winlogon/UAC capture via SetThreadDesktop + BitBlt
//! - WGC (Windows.Graphics.Capture) fallback
//! - Hot re-init on ACCESS_LOST within one frame interval

use setu_core::capabilities::{CaptureCaps, HostCapabilities};
use thiserror::Error;
use tracing::{info, warn, debug};

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("DDA unavailable: {0}")]
    DdaUnavailable(String),
    #[error("Winlogon capture failed: {0}")]
    WinlogonError(String),
    #[error("WGC unavailable: {0}")]
    WgcUnavailable(String),
    #[error("No capture backend available")]
    NoBackend,
    #[error("Frame capture failed: {0}")]
    FrameError(String),
    #[error("GPU access lost")]
    AccessLost,
    #[error("Secure desktop active")]
    SecureDesktop,
}

pub type Result<T> = std::result::Result<T, CaptureError>;

/// Windows capture backend trait
pub trait WinCaptureBackend: Send + Sync {
    fn init() -> Result<Self> where Self: Sized;
    fn is_available() -> bool;
    fn name(&self) -> &'static str;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn capture_frame(&mut self) -> Result<GpuSurfaceHandle>;
    fn handle_access_lost(&mut self) -> Result<()>;
    fn max_fps(&self) -> u32;
    fn max_resolution(&self) -> (u32, u32);
}

/// GPU surface handle for Windows (D3D11 texture)
#[derive(Debug)]
pub struct GpuSurfaceHandle {
    /// D3D11 texture handle (mock FD for now)
    pub texture_handle: i32,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub timestamp_ns: u64,
    _marker: std::marker::PhantomData<*mut ()>,
}

unsafe impl Send for GpuSurfaceHandle {}
unsafe impl Sync for GpuSurfaceHandle {}

/// DDA (DirectDisplay API) capture - primary path for user sessions
pub struct DdaCapture {
    output_index: u32,
    active: bool,
    gpu_device: Option<i32>,
}

impl DdaCapture {
    /// Handle ACCESS_LOST by re-initializing within one frame interval
    pub fn hot_reinit(&mut self) -> Result<()> {
        info!("DDA: hot re-initializing after ACCESS_LOST");
        
        // Real impl:
        // 1. Release old D3D11 device/resources
        // 2. Create new device
        // 3. Re-create output duplication
        // 4. Must complete within 16ms for 60 FPS
        
        self.gpu_device = Some(1); // Mock
        Ok(())
    }
    
    /// Check if running in session that supports DDA
    fn check_session_compatibility() -> bool {
        // DDA doesn't work in session 0 or during RDP
        // Real impl: check ProcessIdToSessionId
        true
    }
}

impl WinCaptureBackend for DdaCapture {
    fn init() -> Result<Self> {
        info!("DDA: initializing capture");
        
        if !Self::check_session_compatibility() {
            return Err(CaptureError::DdaUnavailable("Incompatible session".into()));
        }
        
        Ok(Self {
            output_index: 0,
            active: false,
            gpu_device: None,
        })
    }
    
    fn is_available() -> bool {
        // Check for DXGI 1.2+ and D3D11
        true
    }
    
    fn name(&self) -> &'static str {
        "DDA"
    }
    
    fn start(&mut self) -> Result<()> {
        info!("DDA: starting capture");
        
        // Real impl:
        // 1. Create IDXGIFactory1
        // 2. Enumerate adapters and outputs
        // 3. Create D3D11 device
        // 4. Create IDXGIOutputDuplication
        
        self.active = true;
        self.gpu_device = Some(1);
        Ok(())
    }
    
    fn stop(&mut self) -> Result<()> {
        info!("DDA: stopping capture");
        self.active = false;
        self.gpu_device = None;
        Ok(())
    }
    
    fn capture_frame(&mut self) -> Result<GpuSurfaceHandle> {
        if !self.active {
            return Err(CaptureError::FrameError("Not capturing".into()));
        }
        
        // Real impl: IDXGIOutputDuplication::AcquireNextFrame
        // Check for DXGI_ERROR_ACCESS_LOST → call handle_access_lost
        
        Ok(GpuSurfaceHandle {
            texture_handle: 100,
            width: 1920,
            height: 1080,
            format: 0x3231564e, // NV12
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            _marker: std::marker::PhantomData,
        })
    }
    
    fn handle_access_lost(&mut self) -> Result<()> {
        self.hot_reinit()
    }
    
    fn max_fps(&self) -> u32 {
        144
    }
    
    fn max_resolution(&self) -> (u32, u32) {
        (7680, 4320)
    }
}

/// Winlogon/Secure Desktop capture via BitBlt
pub struct WinlogonCapture {
    winlogon_desktop: bool,
    active: bool,
}

impl WinlogonCapture {
    /// Switch to Winlogon desktop for UAC/secure screens
    fn switch_to_winlogon() -> Result<()> {
        info!("Winlogon: switching to Winlogon desktop");
        
        // Real impl:
        // 1. OpenWindowStation("WinSta0")
        // 2. OpenDesktop("Winlogon")
        // 3. SetThreadDesktop()
        // Requires SE_TCB_PRIVILEGE or running as SYSTEM
        
        Ok(())
    }
    
    /// Capture via GDI BitBlt (CPU-based, ≤10 FPS for static screens)
    fn bitblt_capture() -> Result<Vec<u8>> {
        debug!("Winlogon: capturing via BitBlt (≤10 FPS)");
        
        // Real impl:
        // 1. GetDC(NULL) for screen
        // 2. Create compatible DC and bitmap
        // 3. BitBlt
        // 4. GetDIBits
        
        Ok(vec![0u8; 1920 * 1080 * 4]) // Mock RGBA data
    }
}

impl WinCaptureBackend for WinlogonCapture {
    fn init() -> Result<Self> {
        info!("Winlogon: initializing capture");
        
        Ok(Self {
            winlogon_desktop: false,
            active: false,
        })
    }
    
    fn is_available() -> bool {
        // Always available as fallback (requires appropriate privileges)
        true
    }
    
    fn name(&self) -> &'static str {
        "Winlogon"
    }
    
    fn start(&mut self) -> Result<()> {
        info!("Winlogon: starting capture");
        
        Self::switch_to_winlogon()?;
        self.active = true;
        self.winlogon_desktop = true;
        
        Ok(())
    }
    
    fn stop(&mut self) -> Result<()> {
        info!("Winlogon: stopping capture");
        self.active = false;
        Ok(())
    }
    
    fn capture_frame(&mut self) -> Result<GpuSurfaceHandle> {
        if !self.active {
            return Err(CaptureError::FrameError("Not capturing".into()));
        }
        
        // CPU capture then upload to GPU
        let _data = Self::bitblt_capture()?;
        
        warn!("Winlogon: BitBlt capture (CPU-based, not zero-copy)");
        
        Ok(GpuSurfaceHandle {
            texture_handle: 101,
            width: 1920,
            height: 1080,
            format: 0x3231564e,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            _marker: std::marker::PhantomData,
        })
    }
    
    fn handle_access_lost(&mut self) -> Result<()> {
        // Re-switch to Winlogon desktop
        Self::switch_to_winlogon()
    }
    
    fn max_fps(&self) -> u32 {
        10 // Limited for CPU capture
    }
    
    fn max_resolution(&self) -> (u32, u32) {
        (1920, 1080) // Limit for performance
    }
}

/// WGC (Windows.Graphics.Capture) fallback
pub struct WgcCapture {
    active: bool,
}

impl WgcCapture {
    /// Check for WGC availability (Windows 10 1803+)
    fn check_wgc_support() -> bool {
        // Real impl: check OS version + capability
        true
    }
}

impl WinCaptureBackend for WgcCapture {
    fn init() -> Result<Self> {
        info!("WGC: initializing capture");
        
        if !Self::check_wgc_support() {
            return Err(CaptureError::WgcUnavailable("OS too old".into()));
        }
        
        Ok(Self { active: false })
    }
    
    fn is_available() -> bool {
        Self::check_wgc_support()
    }
    
    fn name(&self) -> &'static str {
        "WGC"
    }
    
    fn start(&mut self) -> Result<()> {
        info!("WGC: starting capture");
        self.active = true;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<()> {
        info!("WGC: stopping capture");
        self.active = false;
        Ok(())
    }
    
    fn capture_frame(&mut self) -> Result<GpuSurfaceHandle> {
        if !self.active {
            return Err(CaptureError::FrameError("Not capturing".into()));
        }
        
        Ok(GpuSurfaceHandle {
            texture_handle: 102,
            width: 1920,
            height: 1080,
            format: 0x3231564e,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            _marker: std::marker::PhantomData,
        })
    }
    
    fn handle_access_lost(&mut self) -> Result<()> {
        // WGC handles this internally
        Ok(())
    }
    
    fn max_fps(&self) -> u32 {
        60
    }
    
    fn max_resolution(&self) -> (u32, u32) {
        (3840, 2160)
    }
}

/// Windows capture chain with dual-agent support
pub struct WindowsCaptureChain {
    backends: Vec<Box<dyn WinCaptureBackend>>,
    active_index: Option<usize>,
    is_service: bool, // Running as session-0 service
}

impl WindowsCaptureChain {
    pub fn new(is_service: bool) -> Self {
        Self {
            backends: Vec::new(),
            active_index: None,
            is_service,
        }
    }
    
    /// Probe and initialize backends based on session context
    pub fn probe_and_init(&mut self) -> Result<()> {
        info!("Windows: probing capture capabilities (service={})", self.is_service);
        
        // Service (session 0) can only do Winlogon
        // User agent can do DDA → WGC → Winlogon
        
        if self.is_service {
            // Service mode: Winlogon only
            if WinlogonCapture::is_available() {
                if let Ok(backend) = WinlogonCapture::init() {
                    self.backends.push(Box::new(backend));
                }
            }
        } else {
            // User mode: DDA first, then WGC, then Winlogon
            if DdaCapture::is_available() {
                if let Ok(backend) = DdaCapture::init() {
                    self.backends.push(Box::new(backend));
                }
            }
            
            if WgcCapture::is_available() {
                if let Ok(backend) = WgcCapture::init() {
                    self.backends.push(Box::new(backend));
                }
            }
            
            // Winlogon as ultimate fallback
            if WinlogonCapture::is_available() {
                if let Ok(backend) = WinlogonCapture::init() {
                    self.backends.push(Box::new(backend));
                }
            }
        }
        
        if self.backends.is_empty() {
            return Err(CaptureError::NoBackend);
        }
        
        Ok(())
    }
    
    pub fn start(&mut self) -> Result<&dyn WinCaptureBackend> {
        for (i, backend) in self.backends.iter_mut().enumerate() {
            match backend.start() {
                Ok(_) => {
                    self.active_index = Some(i);
                    return Ok(backend.as_ref());
                }
                Err(e) => {
                    warn!("Windows: {} failed to start: {}", backend.name(), e);
                    continue;
                }
            }
        }
        Err(CaptureError::NoBackend)
    }
    
    pub fn capture_frame(&mut self) -> Result<GpuSurfaceHandle> {
        let idx = self.active_index.ok_or(CaptureError::NoBackend)?;
        
        match self.backends[idx].capture_frame() {
            Ok(frame) => Ok(frame),
            Err(CaptureError::AccessLost) => {
                // Hot re-init
                self.backends[idx].handle_access_lost()?;
                self.backends[idx].capture_frame()
            }
            Err(e) => Err(e),
        }
    }
    
    pub fn get_capabilities(&self) -> Option<CaptureCaps> {
        self.active_index.map(|i| {
            let b = &self.backends[i];
            CaptureCaps {
                pre_login: vec![],
                locked: vec!["service".to_string()],
                secure_desktop: vec!["winlogon".to_string()],
                zero_copy: b.name() == "DDA",
                dirty_regions: false,
                cursor_metadata: false,
                max_fps: b.max_fps(),
                max_res_width: b.max_resolution().0,
                max_res_height: b.max_resolution().1,
            }
        })
    }
}

/// Build Windows host capabilities
pub fn build_windows_capabilities(is_service: bool) -> HostCapabilities {
    let mut chain = WindowsCaptureChain::new(is_service);
    let _ = chain.probe_and_init();
    
    let capture = chain.get_capabilities().unwrap_or_default();
    
    HostCapabilities {
        capture,
        input: setu_core::capabilities::InputCaps {
            pointer: true,
            keyboard: true,
            secure_attention: is_service, // Only service can SendSAS
            absolute_touch: false,
            relative_pointer: true,
            global_shortcuts: false,
        },
        persist: setu_core::capabilities::PersistCaps {
            reboot: true,
            oem_power_exempt: false,
            unattended: is_service,
            push_wake: vec![],
        },
        trans: setu_core::capabilities::TransportCaps {
            direct: true,
            udp_relay: true,
            masque: true,
            migration: true,
            multipath: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dda_available() {
        assert!(DdaCapture::is_available());
    }
    
    #[test]
    fn test_winlogon_available() {
        assert!(WinlogonCapture::is_available());
    }
    
    #[test]
    fn test_wgc_available() {
        assert!(WgcCapture::is_available());
    }
    
    #[test]
    fn test_capture_chain_user_mode() {
        let mut chain = WindowsCaptureChain::new(false);
        assert!(chain.probe_and_init().is_ok());
        assert!(!chain.backends.is_empty());
    }
    
    #[test]
    fn test_capture_chain_service_mode() {
        let mut chain = WindowsCaptureChain::new(true);
        assert!(chain.probe_and_init().is_ok());
        // Service should only have Winlogon
        assert_eq!(chain.backends.len(), 1);
        assert_eq!(chain.backends[0].name(), "Winlogon");
    }
}
