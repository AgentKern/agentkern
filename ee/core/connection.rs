//! Connection Mode - Graceful Degradation Pattern
//!
//! Production-ready pattern: works with credentials, demo mode without
//! NEVER crashes due to missing credentials

use serde::{Deserialize, Serialize};
use std::env;

/// Connection mode for enterprise features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionMode {
    /// Live mode - real API calls (requires credentials)
    Live,
    /// Demo mode - returns realistic mock data
    Demo,
    /// Offline mode - cached data only
    Offline,
    /// Disabled - feature not enabled
    Disabled,
}

impl ConnectionMode {
    /// Detect mode from environment.
    pub fn detect(feature: &str) -> Self {
        // Check if explicitly disabled
        let disabled_key = format!("AGENTKERN_{}_DISABLED", feature.to_uppercase());
        if env::var(&disabled_key).is_ok() {
            return Self::Disabled;
        }
        
        // Check if demo mode forced
        let demo_key = format!("AGENTKERN_{}_DEMO", feature.to_uppercase());
        if env::var(&demo_key).is_ok() {
            return Self::Demo;
        }
        
        // Check if offline mode
        if env::var("AGENTKERN_OFFLINE").is_ok() {
            return Self::Offline;
        }
        
        // Check for credentials
        let cred_key = format!("AGENTKERN_{}_API_KEY", feature.to_uppercase());
        if env::var(&cred_key).is_ok() {
            return Self::Live;
        }
        
        // Default to demo mode (graceful fallback)
        Self::Demo
    }
    
    /// Is this mode operational (can return data)?
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Live | Self::Demo | Self::Offline)
    }
    
    /// Is this live production mode?
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live)
    }
    
    /// Should we use mock data?
    pub fn use_mock(&self) -> bool {
        matches!(self, Self::Demo | Self::Offline)
    }
}

/// Connection status with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub feature: String,
    pub mode: ConnectionMode,
    pub message: String,
    pub last_checked: String,
}

impl ConnectionStatus {
    /// Create new status.
    pub fn new(feature: &str) -> Self {
        let mode = ConnectionMode::detect(feature);
        let message = match mode {
            ConnectionMode::Live => "Connected to live API".to_string(),
            ConnectionMode::Demo => format!(
                "Demo mode - set AGENTKERN_{}_API_KEY for live", 
                feature.to_uppercase()
            ),
            ConnectionMode::Offline => "Offline mode - using cached data".to_string(),
            ConnectionMode::Disabled => format!(
                "Disabled - unset AGENTKERN_{}_DISABLED to enable",
                feature.to_uppercase()
            ),
        };
        
        Self {
            feature: feature.to_string(),
            mode,
            message,
            last_checked: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Trait for graceful degradation.
pub trait GracefulService {
    /// Get current connection mode.
    fn mode(&self) -> ConnectionMode;
    
    /// Get connection status.
    fn status(&self) -> ConnectionStatus;
    
    /// Check if service is available.
    fn is_available(&self) -> bool {
        self.mode().is_operational()
    }
    
    /// Check if using real credentials.
    fn is_live(&self) -> bool {
        self.mode().is_live()
    }
}

/// Result type that includes mode information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracefulResult<T> {
    pub data: T,
    pub mode: ConnectionMode,
    pub is_mock: bool,
}

impl<T> GracefulResult<T> {
    pub fn live(data: T) -> Self {
        Self { data, mode: ConnectionMode::Live, is_mock: false }
    }
    
    pub fn demo(data: T) -> Self {
        Self { data, mode: ConnectionMode::Demo, is_mock: true }
    }
    
    pub fn offline(data: T) -> Self {
        Self { data, mode: ConnectionMode::Offline, is_mock: true }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_mode_demo_default() {
        // Without credentials, should default to demo
        let mode = ConnectionMode::detect("test_feature");
        assert!(mode.is_operational());
        assert!(mode.use_mock());
    }

    #[test]
    fn test_connection_status() {
        let status = ConnectionStatus::new("sap");
        assert_eq!(status.feature, "sap");
        assert!(!status.last_checked.is_empty());
    }

    #[test]
    fn test_graceful_result() {
        let result = GracefulResult::demo("test data");
        assert!(result.is_mock);
        assert_eq!(result.mode, ConnectionMode::Demo);
    }
}
