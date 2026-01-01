//! Minimal Runtime for Edge Devices
//!
//! Stripped-down runtime optimized for constrained environments

use serde::{Deserialize, Serialize};

#[cfg(feature = "embedded")]
use alloc::{string::String, vec::Vec};

/// Edge runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConfig {
    /// Device ID
    pub device_id: String,
    /// Max memory usage (bytes)
    pub max_memory: usize,
    /// Enable offline mode
    pub offline_enabled: bool,
    /// Sync interval (seconds, 0 = manual)
    pub sync_interval_secs: u32,
    /// Task queue size
    pub queue_size: usize,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            max_memory: super::MAX_MEMORY,
            offline_enabled: true,
            sync_interval_secs: 60,
            queue_size: 100,
        }
    }
}

/// Minimal edge runtime.
pub struct EdgeRuntime {
    config: EdgeConfig,
    state: RuntimeState,
    policies: Vec<super::policy::PolicyRule>,
}

/// Runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Initializing
    Starting,
    /// Running normally
    Running,
    /// Offline mode
    Offline,
    /// Low memory
    Constrained,
    /// Shutting down
    Stopping,
}

impl EdgeRuntime {
    /// Create new edge runtime.
    pub fn new(config: EdgeConfig) -> Result<Self, EdgeError> {
        if config.max_memory < 64 * 1024 {
            return Err(EdgeError::InsufficientMemory);
        }

        Ok(Self {
            config,
            state: RuntimeState::Starting,
            policies: Vec::new(),
        })
    }

    /// Start the runtime.
    pub fn start(&mut self) -> Result<(), EdgeError> {
        self.state = RuntimeState::Running;
        Ok(())
    }

    /// Stop the runtime.
    pub fn stop(&mut self) -> Result<(), EdgeError> {
        self.state = RuntimeState::Stopping;
        Ok(())
    }

    /// Get current state.
    pub fn state(&self) -> RuntimeState {
        self.state
    }

    /// Enter offline mode.
    pub fn go_offline(&mut self) {
        self.state = RuntimeState::Offline;
    }

    /// Check if online.
    pub fn is_online(&self) -> bool {
        self.state == RuntimeState::Running
    }

    /// Add policy rule.
    pub fn add_policy(&mut self, rule: super::policy::PolicyRule) {
        self.policies.push(rule);
    }

    /// Evaluate action against policies.
    pub fn evaluate(&self, action: &str) -> super::policy::PolicyAction {
        for rule in &self.policies {
            if rule.matches(action) {
                return rule.action;
            }
        }
        super::policy::PolicyAction::Allow
    }

    /// Get memory usage estimate.
    pub fn memory_usage(&self) -> usize {
        // Simplified estimate
        core::mem::size_of::<Self>() + self.policies.len() * 64
    }

    /// Check memory constraints.
    pub fn is_constrained(&self) -> bool {
        self.memory_usage() > self.config.max_memory / 2
    }
}

/// Edge runtime error.
#[derive(Debug)]
pub enum EdgeError {
    /// Not enough memory
    InsufficientMemory,
    /// Already running
    AlreadyRunning,
    /// Not running
    NotRunning,
    /// Policy violation
    PolicyViolation,
    /// Offline
    Offline,
}

impl core::fmt::Display for EdgeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InsufficientMemory => write!(f, "Insufficient memory"),
            Self::AlreadyRunning => write!(f, "Already running"),
            Self::NotRunning => write!(f, "Not running"),
            Self::PolicyViolation => write!(f, "Policy violation"),
            Self::Offline => write!(f, "Offline"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_config_default() {
        let config = EdgeConfig::default();
        assert!(config.offline_enabled);
        assert_eq!(config.max_memory, super::super::MAX_MEMORY);
    }

    #[test]
    fn test_edge_runtime_create() {
        let config = EdgeConfig::default();
        let runtime = EdgeRuntime::new(config).unwrap();
        assert_eq!(runtime.state(), RuntimeState::Starting);
    }

    #[test]
    fn test_edge_runtime_lifecycle() {
        let config = EdgeConfig::default();
        let mut runtime = EdgeRuntime::new(config).unwrap();

        runtime.start().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Running);

        runtime.go_offline();
        assert_eq!(runtime.state(), RuntimeState::Offline);

        runtime.stop().unwrap();
        assert_eq!(runtime.state(), RuntimeState::Stopping);
    }
}
