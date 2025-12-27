//! VeriMantle Runtime: Graceful Fallback Infrastructure
//!
//! Production-ready pattern: works with credentials, graceful fallback without.
//! NEVER crashes due to missing credentials or external services.
//!
//! # Usage
//!
//! ```rust,ignore
//! use verimantle_runtime::fallback::{ServiceMode, GracefulFallback};
//!
//! struct MyEmbedder {
//!     mode: ServiceMode,
//! }
//!
//! impl GracefulFallback for MyEmbedder {
//!     fn mode(&self) -> ServiceMode { self.mode }
//!     fn feature_name() -> &'static str { "EMBEDDINGS" }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::env;

/// Service operation mode with graceful degradation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceMode {
    /// Live mode - real API calls (credentials available)
    Live,
    /// Demo mode - returns realistic mock data (no credentials)
    Demo,
    /// Offline mode - cached/local data only
    Offline,
    /// Disabled - feature explicitly disabled
    Disabled,
}

impl Default for ServiceMode {
    fn default() -> Self {
        Self::Demo
    }
}

impl ServiceMode {
    /// Detect mode from environment variables.
    ///
    /// Order of precedence:
    /// 1. `VERIMANTLE_{FEATURE}_DISABLED=1` -> Disabled
    /// 2. `VERIMANTLE_{FEATURE}_DEMO=1` -> Demo
    /// 3. `VERIMANTLE_OFFLINE=1` -> Offline
    /// 4. `VERIMANTLE_{FEATURE}_API_KEY` or `{FEATURE}_API_KEY` set -> Live
    /// 5. Default -> Demo (graceful fallback)
    pub fn detect(feature: &str) -> Self {
        let feature_upper = feature.to_uppercase();
        
        // Check if explicitly disabled
        if env::var(format!("VERIMANTLE_{}_DISABLED", feature_upper)).is_ok() {
            tracing::info!(feature = %feature, "Feature explicitly disabled");
            return Self::Disabled;
        }
        
        // Check if demo mode forced
        if env::var(format!("VERIMANTLE_{}_DEMO", feature_upper)).is_ok() {
            tracing::debug!(feature = %feature, "Demo mode forced via env");
            return Self::Demo;
        }
        
        // Check if global offline mode
        if env::var("VERIMANTLE_OFFLINE").is_ok() {
            tracing::debug!(feature = %feature, "Offline mode active");
            return Self::Offline;
        }
        
        // Check for API credentials (multiple patterns)
        let api_key_patterns = [
            format!("VERIMANTLE_{}_API_KEY", feature_upper),
            format!("{}_API_KEY", feature_upper),
            format!("VERIMANTLE_{}_KEY", feature_upper),
        ];
        
        for pattern in &api_key_patterns {
            if let Ok(key) = env::var(pattern) {
                if !key.is_empty() {
                    tracing::info!(feature = %feature, "Live mode - credentials found");
                    return Self::Live;
                }
            }
        }
        
        // Default to demo mode (graceful fallback)
        tracing::debug!(
            feature = %feature, 
            "No credentials found, using demo mode. Set {}_API_KEY for live.",
            feature_upper
        );
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
    
    /// Should we use mock/fallback data?
    pub fn use_fallback(&self) -> bool {
        matches!(self, Self::Demo | Self::Offline)
    }
    
    /// Get human-readable status message.
    pub fn status_message(&self, feature: &str) -> String {
        match self {
            Self::Live => "✓ Connected to live API".to_string(),
            Self::Demo => format!(
                "⚠ Demo mode - set VERIMANTLE_{}_API_KEY for live",
                feature.to_uppercase()
            ),
            Self::Offline => "⚠ Offline mode - using cached data".to_string(),
            Self::Disabled => format!(
                "✗ Disabled - unset VERIMANTLE_{}_DISABLED to enable",
                feature.to_uppercase()
            ),
        }
    }
}

/// Trait for services with graceful fallback capability.
pub trait GracefulFallback {
    /// Get current service mode.
    fn mode(&self) -> ServiceMode;
    
    /// Get feature name for environment variable lookups.
    fn feature_name() -> &'static str;
    
    /// Check if service is available (not disabled).
    fn is_available(&self) -> bool {
        self.mode().is_operational()
    }
    
    /// Check if using real credentials.
    fn is_live(&self) -> bool {
        self.mode().is_live()
    }
    
    /// Get status message.
    fn status(&self) -> String {
        self.mode().status_message(Self::feature_name())
    }
}

/// Result wrapper that indicates data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackResult<T> {
    pub data: T,
    pub mode: ServiceMode,
    pub is_fallback: bool,
}

impl<T> FallbackResult<T> {
    /// Create live result (real data).
    pub fn live(data: T) -> Self {
        Self { data, mode: ServiceMode::Live, is_fallback: false }
    }
    
    /// Create demo/fallback result (mock data).
    pub fn fallback(data: T) -> Self {
        Self { data, mode: ServiceMode::Demo, is_fallback: true }
    }
    
    /// Map the data.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> FallbackResult<U> {
        FallbackResult {
            data: f(self.data),
            mode: self.mode,
            is_fallback: self.is_fallback,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_mode_default_demo() {
        // Without any env vars, should default to demo
        std::env::remove_var("VERIMANTLE_TEST_API_KEY");
        std::env::remove_var("VERIMANTLE_TEST_DISABLED");
        std::env::remove_var("VERIMANTLE_OFFLINE");
        
        let mode = ServiceMode::detect("test");
        assert_eq!(mode, ServiceMode::Demo);
        assert!(mode.is_operational());
        assert!(mode.use_fallback());
    }

    #[test]
    fn test_service_mode_live_with_key() {
        std::env::set_var("VERIMANTLE_LIVETEST_API_KEY", "sk-test-123");
        let mode = ServiceMode::detect("livetest");
        assert_eq!(mode, ServiceMode::Live);
        assert!(!mode.use_fallback());
        std::env::remove_var("VERIMANTLE_LIVETEST_API_KEY");
    }

    #[test]
    fn test_service_mode_disabled() {
        std::env::set_var("VERIMANTLE_DISTEST_DISABLED", "1");
        let mode = ServiceMode::detect("distest");
        assert_eq!(mode, ServiceMode::Disabled);
        assert!(!mode.is_operational());
        std::env::remove_var("VERIMANTLE_DISTEST_DISABLED");
    }

    #[test]
    fn test_fallback_result() {
        let result = FallbackResult::fallback(vec![0.0f32; 384]);
        assert!(result.is_fallback);
        assert_eq!(result.mode, ServiceMode::Demo);
    }
}
