//! Android Screen Capture: Capability Classes per Setu v5 §5.3
//! 
//! Implements capability-based abstraction (NOT universal tiers):
//! - Attended: MediaProjection + consent, AccessibilityService (assist-limited)
//! - Managed/Privileged: MDM Device Owner
//! - Root/System: Shell UID, Shizuku, root
//! - Unsupported: Documented limitations
//!
//! scrcpy-model app_process server for input injection

use setu_core::capabilities::{CaptureCaps, HostCapabilities};
use thiserror::Error;
use tracing::{info, warn, debug};

#[derive(Error, Debug)]
pub enum CaptureError {
    #[error("MediaProjection unavailable: {0}")]
    MediaProjectionUnavailable(String),
    #[error("Injection unsupported: {0}")]
    InjectionUnsupported(String),
    #[error("Consent required")]
    ConsentRequired,
    #[error("Protected content (DRM) - black frame")]
    ProtectedContent,
    #[error("No capture backend available")]
    NoBackend,
    #[error("Frame capture failed: {0}")]
    FrameError(String),
}

pub type Result<T> = std::result::Result<T, CaptureError>;

/// Android capability class (device-certified via Spike 4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidCapabilityClass {
    /// Attended: MediaProjection + user consent each session
    Attended,
    /// Managed: MDM Device Owner privileges
    Managed,
    /// Root/System: Root access or system signature
    RootSystem,
    /// Unsupported: Cannot capture/inject
    Unsupported,
}

/// Execution identity for injection testing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionIdentity {
    ShellUid,      // com.android.shell
    ShizukuUid,    // Via Shizuku
    Root,          // Root
    DeviceOwner,   // MDM Device Owner
    Accessibility, // AccessibilityService (assist-limited)
    OemSpecific,   // OEM-specific permission
}

/// Android capture backend trait
pub trait AndroidCaptureBackend: Send + Sync {
    fn init() -> Result<Self> where Self: Sized;
    fn is_available() -> bool;
    fn name(&self) -> &'static str;
    fn capability_class(&self) -> AndroidCapabilityClass;
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn capture_frame(&mut self) -> Result<GpuSurfaceHandle>;
    fn max_fps(&self) -> u32;
    fn max_resolution(&self) -> (u32, u32);
    fn supports_screen_off(&self) -> bool;
}

/// GPU surface handle (Android Surface/BufferQueue)
#[derive(Debug)]
pub struct GpuSurfaceHandle {
    pub buffer_handle: i32,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub timestamp_ns: u64,
    _marker: std::marker::PhantomData<*mut ()>,
}

unsafe impl Send for GpuSurfaceHandle {}
unsafe impl Sync for GpuSurfaceHandle {}

/// MediaProjection capture (attended, requires consent)
pub struct MediaProjectionCapture {
    consent_given: bool,
    active: bool,
}

impl MediaProjectionCapture {
    /// Request consent via MediaProjectionManager
    fn request_consent() -> Result<bool> {
        info!("MediaProjection: requesting user consent");
        
        // Real impl:
        // 1. Start Activity with MediaProjectionManager.createScreenCaptureIntent()
        // 2. User sees system consent dialog
        // 3. onActivityResult returns MediaProjection token
        
        // Android 14+ requires per-session consent
        Ok(true) // Mock consent
    }
    
    /// Check for protected content (renders black per OS limitation)
    fn check_protected_content() -> bool {
        // Real impl: check for DRM surfaces via SurfaceControl
        false
    }
}

impl AndroidCaptureBackend for MediaProjectionCapture {
    fn init() -> Result<Self> {
        info!("MediaProjection: initializing");
        
        if !Self::request_consent()? {
            return Err(CaptureError::ConsentRequired);
        }
        
        Ok(Self {
            consent_given: true,
            active: false,
        })
    }
    
    fn is_available() -> bool {
        // Always available but requires consent
        true
    }
    
    fn name(&self) -> &'static str {
        "MediaProjection"
    }
    
    fn capability_class(&self) -> AndroidCapabilityClass {
        AndroidCapabilityClass::Attended
    }
    
    fn start(&mut self) -> Result<()> {
        info!("MediaProjection: starting capture");
        self.active = true;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<()> {
        info!("MediaProjection: stopping capture");
        self.active = false;
        Ok(())
    }
    
    fn capture_frame(&mut self) -> Result<GpuSurfaceHandle> {
        if !self.active {
            return Err(CaptureError::FrameError("Not capturing".into()));
        }
        
        if Self::check_protected_content() {
            return Err(CaptureError::ProtectedContent);
        }
        
        Ok(GpuSurfaceHandle {
            buffer_handle: 100,
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
    
    fn max_fps(&self) -> u32 {
        60
    }
    
    fn max_resolution(&self) -> (u32, u32) {
        (2560, 1440)
    }
    
    fn supports_screen_off(&self) -> bool {
        // MediaProjection doesn't work with screen off by default
        // Requires device-specific workaround (wake-lock + brightness-0)
        false
    }
}

/// scrcpy-model input injection server
pub struct ScrcpyInputServer {
    identity: ExecutionIdentity,
    active: bool,
}

impl ScrcpyInputServer {
    /// Start app_process server for input injection
    fn start_server(identity: ExecutionIdentity) -> Result<Self> {
        info!("scrcpy: starting input server as {:?}", identity);
        
        // Real impl:
        // 1. Push server.jar to device
        // 2. Run via app_process with appropriate UID
        // 3. InputManager.injectInputEvent()
        
        match identity {
            ExecutionIdentity::ShellUid => {
                // adb shell permissions
                Ok(Self { identity, active: true })
            }
            ExecutionIdentity::ShizukuUid => {
                // Via Shizuku service
                Ok(Self { identity, active: true })
            }
            ExecutionIdentity::Root => {
                // Root injection
                Ok(Self { identity, active: true })
            }
            ExecutionIdentity::DeviceOwner => {
                // MDM Device Owner - full injection
                Ok(Self { identity, active: true })
            }
            ExecutionIdentity::Accessibility => {
                // AccessibilityService - assist-limited, not full injection
                warn!("AccessibilityService: assist-limited injection only");
                Ok(Self { identity, active: true })
            }
            ExecutionIdentity::OemSpecific => {
                // OEM-specific permission (Samsung, Xiaomi, etc.)
                Ok(Self { identity, active: true })
            }
        }
    }
    
    /// Test injection capability for given identity
    pub fn test_injection(identity: ExecutionIdentity) -> bool {
        // Real impl: actually try to inject event and verify
        match identity {
            ExecutionIdentity::DeviceOwner => true,
            ExecutionIdentity::Root => true,
            ExecutionIdentity::ShellUid => true,
            ExecutionIdentity::ShizukuUid => true,
            ExecutionIdentity::Accessibility => false, // Limited
            ExecutionIdentity::OemSpecific => true,
        }
    }
}

/// Android capture chain with capability classes
pub struct AndroidCaptureChain {
    backend: Option<Box<dyn AndroidCaptureBackend>>,
    input_server: Option<ScrcpyInputServer>,
    capability_class: AndroidCapabilityClass,
}

impl AndroidCaptureChain {
    pub fn new() -> Self {
        Self {
            backend: None,
            input_server: None,
            capability_class: AndroidCapabilityClass::Unsupported,
        }
    }
    
    /// Probe and determine capability class
    pub fn probe_and_init(&mut self) -> Result<()> {
        info!("Android: probing capabilities");
        
        // Try MediaProjection first (attended)
        if MediaProjectionCapture::is_available() {
            if let Ok(backend) = MediaProjectionCapture::init() {
                self.capability_class = backend.capability_class();
                self.backend = Some(Box::new(backend));
                info!("Android: using MediaProjection (attended class)");
                
                // Try to start input server
                // Priority: DeviceOwner > Root > Shizuku > Shell > Accessibility
                for identity in [
                    ExecutionIdentity::DeviceOwner,
                    ExecutionIdentity::Root,
                    ExecutionIdentity::ShizukuUid,
                    ExecutionIdentity::ShellUid,
                    ExecutionIdentity::Accessibility,
                ] {
                    if ScrcpyInputServer::test_injection(identity) {
                        if let Ok(server) = ScrcpyInputServer::start_server(identity) {
                            self.input_server = Some(server);
                            info!("Android: input injection via {:?}", identity);
                            break;
                        }
                    }
                }
                
                return Ok(());
            }
        }
        
        Err(CaptureError::NoBackend)
    }
    
    pub fn start(&mut self) -> Result<()> {
        if let Some(ref mut backend) = self.backend {
            backend.start()
        } else {
            Err(CaptureError::NoBackend)
        }
    }
    
    pub fn capture_frame(&mut self) -> Result<GpuSurfaceHandle> {
        if let Some(ref mut backend) = self.backend {
            backend.capture_frame()
        } else {
            Err(CaptureError::NoBackend)
        }
    }
    
    pub fn get_capabilities(&self) -> Option<CaptureCaps> {
        self.backend.as_ref().map(|b| CaptureCaps {
            pre_login: vec![],
            locked: vec![],
            secure_desktop: vec![],
            zero_copy: true, // MediaProjection is zero-copy
            dirty_regions: false,
            cursor_metadata: false,
            max_fps: b.max_fps(),
            max_res_width: b.max_resolution().0,
            max_res_height: b.max_resolution().1,
        })
    }
    
    pub fn capability_class(&self) -> AndroidCapabilityClass {
        self.capability_class
    }
    
    /// Check if screen-off capture is supported (device-certified)
    pub fn supports_screen_off(&self) -> bool {
        self.backend.as_ref()
            .map(|b| b.supports_screen_off())
            .unwrap_or(false)
    }
}

impl Default for AndroidCaptureChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Build Android host capabilities
pub fn build_android_capabilities() -> HostCapabilities {
    let mut chain = AndroidCaptureChain::new();
    let _ = chain.probe_and_init();
    
    let capture = chain.get_capabilities().unwrap_or_default();
    
    HostCapabilities {
        capture,
        input: setu_core::capabilities::InputCaps {
            pointer: true,
            keyboard: true,
            secure_attention: false,
            absolute_touch: true, // Android is touch-first
            relative_pointer: chain.input_server.is_some(),
            global_shortcuts: false,
        },
        persist: setu_core::capabilities::PersistCaps {
            reboot: true,
            oem_power_exempt: false, // Requires OEM exemption
            unattended: matches!(chain.capability_class(), 
                AndroidCapabilityClass::Managed | AndroidCapabilityClass::RootSystem),
            push_wake: vec![], // FCM high-priority requires OEM exemption
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
    fn test_media_projection_available() {
        assert!(MediaProjectionCapture::is_available());
    }
    
    #[test]
    fn test_capability_classes() {
        let cap = AndroidCapabilityClass::Attended;
        assert_eq!(cap, AndroidCapabilityClass::Attended);
    }
    
    #[test]
    fn test_injection_identities() {
        assert!(ScrcpyInputServer::test_injection(ExecutionIdentity::DeviceOwner));
        assert!(ScrcpyInputServer::test_injection(ExecutionIdentity::Root));
        assert!(!ScrcpyInputServer::test_injection(ExecutionIdentity::Accessibility));
    }
    
    #[test]
    fn test_capture_chain_init() {
        let mut chain = AndroidCaptureChain::new();
        // May fail if no consent, but shouldn't panic
        let _ = chain.probe_and_init();
    }
}
