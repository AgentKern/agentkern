//! Chaos Testing Module
//!
//! Per MANDATE.md: "Antifragile by default - every failure makes us stronger"
//!
//! Provides fault injection and chaos engineering capabilities for testing
//! system resilience under adverse conditions.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use rand::Rng;

/// Chaos configuration.
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Probability of injecting latency (0-100%)
    pub latency_probability: u8,
    /// Latency range in milliseconds
    pub latency_range_ms: (u64, u64),
    /// Probability of injecting errors (0-100%)
    pub error_probability: u8,
    /// Error types to inject
    pub error_types: Vec<ChaosError>,
    /// Enable/disable chaos
    pub enabled: bool,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            latency_probability: 0,
            latency_range_ms: (100, 500),
            error_probability: 0,
            error_types: vec![ChaosError::Timeout, ChaosError::NetworkError],
            enabled: false,
        }
    }
}

impl ChaosConfig {
    /// Create a mild chaos config (low probability).
    pub fn mild() -> Self {
        Self {
            latency_probability: 10,
            error_probability: 5,
            enabled: true,
            ..Default::default()
        }
    }

    /// Create a moderate chaos config.
    pub fn moderate() -> Self {
        Self {
            latency_probability: 25,
            latency_range_ms: (200, 1000),
            error_probability: 15,
            error_types: vec![
                ChaosError::Timeout,
                ChaosError::NetworkError,
                ChaosError::ServiceUnavailable,
            ],
            enabled: true,
        }
    }

    /// Create an extreme chaos config (stress testing).
    pub fn extreme() -> Self {
        Self {
            latency_probability: 50,
            latency_range_ms: (500, 5000),
            error_probability: 30,
            error_types: vec![
                ChaosError::Timeout,
                ChaosError::NetworkError,
                ChaosError::ServiceUnavailable,
                ChaosError::InternalError,
                ChaosError::DataCorruption,
            ],
            enabled: true,
        }
    }
}

/// Types of chaos errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaosError {
    /// Request timeout
    Timeout,
    /// Network connectivity error
    NetworkError,
    /// Service temporarily unavailable
    ServiceUnavailable,
    /// Internal server error
    InternalError,
    /// Data corruption (bad response)
    DataCorruption,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// Authentication failure
    AuthFailure,
}

impl ChaosError {
    /// Get HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Timeout => 504,
            Self::NetworkError => 503,
            Self::ServiceUnavailable => 503,
            Self::InternalError => 500,
            Self::DataCorruption => 500,
            Self::RateLimitExceeded => 429,
            Self::AuthFailure => 401,
        }
    }

    /// Get error message.
    pub fn message(&self) -> &'static str {
        match self {
            Self::Timeout => "Request timed out",
            Self::NetworkError => "Network error occurred",
            Self::ServiceUnavailable => "Service temporarily unavailable",
            Self::InternalError => "Internal server error",
            Self::DataCorruption => "Data corruption detected",
            Self::RateLimitExceeded => "Rate limit exceeded",
            Self::AuthFailure => "Authentication failed",
        }
    }
}

/// Chaos injection result.
#[derive(Debug, Clone)]
pub enum ChaosResult<T> {
    /// Normal execution
    Ok(T),
    /// Latency injected
    Delayed { result: T, delay_ms: u64 },
    /// Error injected
    Error(ChaosError),
}

impl<T> ChaosResult<T> {
    /// Check if chaos was injected.
    pub fn had_chaos(&self) -> bool {
        !matches!(self, ChaosResult::Ok(_))
    }

    /// Get result if available.
    pub fn into_result(self) -> Result<T, ChaosError> {
        match self {
            ChaosResult::Ok(v) => Ok(v),
            ChaosResult::Delayed { result, .. } => Ok(result),
            ChaosResult::Error(e) => Err(e),
        }
    }
}

/// Chaos monkey for fault injection.
pub struct ChaosMonkey {
    config: ChaosConfig,
    /// Total operations processed
    total_ops: AtomicU32,
    /// Latency injections
    latency_injections: AtomicU32,
    /// Error injections
    error_injections: AtomicU32,
    /// Is paused
    paused: AtomicBool,
}

impl ChaosMonkey {
    /// Create a new chaos monkey.
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config,
            total_ops: AtomicU32::new(0),
            latency_injections: AtomicU32::new(0),
            error_injections: AtomicU32::new(0),
            paused: AtomicBool::new(false),
        }
    }

    /// Create disabled chaos monkey.
    pub fn disabled() -> Self {
        Self::new(ChaosConfig::default())
    }

    /// Enable chaos.
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable chaos.
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// Pause chaos injection.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    /// Resume chaos injection.
    pub fn resume(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    /// Maybe inject chaos into an operation.
    pub fn maybe_inject<T, F>(&self, operation: F) -> ChaosResult<T>
    where
        F: FnOnce() -> T,
    {
        self.total_ops.fetch_add(1, Ordering::Relaxed);

        // Check if chaos is enabled and not paused
        if !self.config.enabled || self.paused.load(Ordering::Relaxed) {
            return ChaosResult::Ok(operation());
        }

        let mut rng = rand::thread_rng();

        // Check for error injection first
        if rng.gen_range(0..100) < self.config.error_probability {
            self.error_injections.fetch_add(1, Ordering::Relaxed);
            let error_idx = rng.gen_range(0..self.config.error_types.len());
            return ChaosResult::Error(self.config.error_types[error_idx]);
        }

        // Check for latency injection
        if rng.gen_range(0..100) < self.config.latency_probability {
            self.latency_injections.fetch_add(1, Ordering::Relaxed);
            let (min, max) = self.config.latency_range_ms;
            let delay = rng.gen_range(min..=max);
            
            // In a real implementation, we'd actually sleep here
            // For now, we just record the intended delay
            let result = operation();
            return ChaosResult::Delayed { result, delay_ms: delay };
        }

        ChaosResult::Ok(operation())
    }

    /// Async version with actual delay.
    pub async fn maybe_inject_async<T, F, Fut>(&self, operation: F) -> ChaosResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        self.total_ops.fetch_add(1, Ordering::Relaxed);

        if !self.config.enabled || self.paused.load(Ordering::Relaxed) {
            return ChaosResult::Ok(operation().await);
        }

        let mut rng = rand::thread_rng();

        // Error injection
        if rng.gen_range(0..100) < self.config.error_probability {
            self.error_injections.fetch_add(1, Ordering::Relaxed);
            let error_idx = rng.gen_range(0..self.config.error_types.len());
            return ChaosResult::Error(self.config.error_types[error_idx]);
        }

        // Latency injection
        if rng.gen_range(0..100) < self.config.latency_probability {
            self.latency_injections.fetch_add(1, Ordering::Relaxed);
            let (min, max) = self.config.latency_range_ms;
            let delay = rng.gen_range(min..=max);
            
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            
            let result = operation().await;
            return ChaosResult::Delayed { result, delay_ms: delay };
        }

        ChaosResult::Ok(operation().await)
    }

    /// Get statistics.
    pub fn stats(&self) -> ChaosStats {
        ChaosStats {
            total_ops: self.total_ops.load(Ordering::Relaxed),
            latency_injections: self.latency_injections.load(Ordering::Relaxed),
            error_injections: self.error_injections.load(Ordering::Relaxed),
        }
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        self.total_ops.store(0, Ordering::Relaxed);
        self.latency_injections.store(0, Ordering::Relaxed);
        self.error_injections.store(0, Ordering::Relaxed);
    }
}

/// Chaos statistics.
#[derive(Debug, Clone, Default)]
pub struct ChaosStats {
    pub total_ops: u32,
    pub latency_injections: u32,
    pub error_injections: u32,
}

impl ChaosStats {
    /// Get chaos injection rate.
    pub fn chaos_rate(&self) -> f64 {
        if self.total_ops == 0 {
            return 0.0;
        }
        (self.latency_injections + self.error_injections) as f64 / self.total_ops as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_chaos() {
        let monkey = ChaosMonkey::disabled();
        
        let result = monkey.maybe_inject(|| 42);
        
        match result {
            ChaosResult::Ok(v) => assert_eq!(v, 42),
            _ => panic!("Expected Ok result"),
        }
    }

    #[test]
    fn test_chaos_config_presets() {
        let mild = ChaosConfig::mild();
        assert_eq!(mild.latency_probability, 10);
        assert_eq!(mild.error_probability, 5);

        let moderate = ChaosConfig::moderate();
        assert_eq!(moderate.latency_probability, 25);

        let extreme = ChaosConfig::extreme();
        assert_eq!(extreme.error_probability, 30);
    }

    #[test]
    fn test_chaos_error_codes() {
        assert_eq!(ChaosError::Timeout.status_code(), 504);
        assert_eq!(ChaosError::RateLimitExceeded.status_code(), 429);
        assert_eq!(ChaosError::AuthFailure.status_code(), 401);
    }

    #[test]
    fn test_chaos_stats() {
        let monkey = ChaosMonkey::new(ChaosConfig {
            latency_probability: 100,
            error_probability: 0,
            enabled: true,
            ..Default::default()
        });

        for _ in 0..10 {
            let _ = monkey.maybe_inject(|| 1);
        }

        let stats = monkey.stats();
        assert_eq!(stats.total_ops, 10);
        assert_eq!(stats.latency_injections, 10);
    }

    #[test]
    fn test_chaos_pause_resume() {
        let monkey = ChaosMonkey::new(ChaosConfig {
            error_probability: 100,
            enabled: true,
            ..Default::default()
        });

        // Should inject error
        let result1 = monkey.maybe_inject(|| 1);
        assert!(matches!(result1, ChaosResult::Error(_)));

        // Pause - should not inject
        monkey.pause();
        let result2 = monkey.maybe_inject(|| 2);
        assert!(matches!(result2, ChaosResult::Ok(2)));

        // Resume - should inject again
        monkey.resume();
        let result3 = monkey.maybe_inject(|| 3);
        assert!(matches!(result3, ChaosResult::Error(_)));
    }

    #[test]
    fn test_chaos_result_conversion() {
        let ok: ChaosResult<i32> = ChaosResult::Ok(42);
        assert!(!ok.had_chaos());
        assert_eq!(ok.into_result().unwrap(), 42);

        let delayed: ChaosResult<i32> = ChaosResult::Delayed { result: 42, delay_ms: 100 };
        assert!(delayed.had_chaos());
        assert_eq!(delayed.into_result().unwrap(), 42);

        let err: ChaosResult<i32> = ChaosResult::Error(ChaosError::Timeout);
        assert!(err.had_chaos());
        assert!(err.into_result().is_err());
    }
}
