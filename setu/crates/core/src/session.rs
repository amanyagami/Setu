//! Session State Machine per Setu v5 §7
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Invalid transition from {from:?} to {to:?}")]
    InvalidTransition { from: SessionState, to: &'static str },
    #[error("SLO violation: {0}")]
    SloViolation(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Connecting,
    Authenticating,
    CapabilitiesExchange,
    Streaming,
    Migrating,
    Disconnecting,
    Disconnected,
    Error,
}

pub struct Session {
    state: SessionState,
    slo_deadline_ns: u64,
}

impl Session {
    pub fn new() -> Self {
        Self {
            state: SessionState::Idle,
            slo_deadline_ns: 0,
        }
    }
    
    pub fn state(&self) -> SessionState { self.state }
    
    pub fn connect(&mut self) -> Result<(), SessionError> {
        if self.state != SessionState::Idle {
            return Err(SessionError::InvalidTransition { from: self.state, to: "Connecting" });
        }
        self.state = SessionState::Connecting;
        Ok(())
    }
    
    pub fn authenticate(&mut self) -> Result<(), SessionError> {
        if self.state != SessionState::Connecting {
            return Err(SessionError::InvalidTransition { from: self.state, to: "Authenticating" });
        }
        self.state = SessionState::Authenticating;
        Ok(())
    }
    
    pub fn exchange_capabilities(&mut self) -> Result<(), SessionError> {
        if self.state != SessionState::Authenticating {
            return Err(SessionError::InvalidTransition { from: self.state, to: "CapabilitiesExchange" });
        }
        self.state = SessionState::CapabilitiesExchange;
        Ok(())
    }
    
    pub fn start_streaming(&mut self, ttff_deadline_ns: u64) -> Result<(), SessionError> {
        if self.state != SessionState::CapabilitiesExchange {
            return Err(SessionError::InvalidTransition { from: self.state, to: "Streaming" });
        }
        self.state = SessionState::Streaming;
        self.slo_deadline_ns = ttff_deadline_ns;
        Ok(())
    }
    
    pub fn migrate(&mut self) -> Result<(), SessionError> {
        if self.state != SessionState::Streaming {
            return Err(SessionError::InvalidTransition { from: self.state, to: "Migrating" });
        }
        self.state = SessionState::Migrating;
        Ok(())
    }
    
    pub fn complete_migration(&mut self) -> Result<(), SessionError> {
        if self.state != SessionState::Migrating {
            return Err(SessionError::InvalidTransition { from: self.state, to: "Streaming" });
        }
        self.state = SessionState::Streaming;
        Ok(())
    }
    
    pub fn disconnect(&mut self) -> Result<(), SessionError> {
        self.state = SessionState::Disconnecting;
        Ok(())
    }
    
    pub fn finish_disconnect(&mut self) -> Result<(), SessionError> {
        if self.state != SessionState::Disconnecting {
            return Err(SessionError::InvalidTransition { from: self.state, to: "Disconnected" });
        }
        self.state = SessionState::Disconnected;
        Ok(())
    }
    
    pub fn check_slo(&self, elapsed_ns: u64) -> bool {
        elapsed_ns <= self.slo_deadline_ns
    }
}

impl Default for Session {
    fn default() -> Self { Self::new() }
}
