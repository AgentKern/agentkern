//! Chaos Proxy for LLM Provider Failure Simulation
//!
//! Per Antifragility Roadmap: "Third-Party API Mocking"
//! Simulates failures of external LLM providers (OpenAI, Anthropic, etc.)
//! for chaos testing and resilience validation.
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_nexus::chaos_proxy::{ChaosProxy, ChaosProxyConfig, LLMProvider};
//!
//! let config = ChaosProxyConfig::default()
//!     .with_provider(LLMProvider::OpenAI, 0.1)  // 10% failure rate
//!     .with_provider(LLMProvider::Anthropic, 0.05);
//!
//! let proxy = ChaosProxy::new(config);
//!
//! // Wrap LLM calls with chaos injection
//! match proxy.maybe_fail(LLMProvider::OpenAI).await {
//!     Ok(()) => { /* proceed with actual call */ }
//!     Err(e) => { /* handle simulated failure */ }
//! }
//! ```

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Supported LLM providers for chaos simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    Anthropic,
    Google,
    Cohere,
    Mistral,
    Local,
    Custom,
}

impl std::fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAI => write!(f, "OpenAI"),
            Self::Anthropic => write!(f, "Anthropic"),
            Self::Google => write!(f, "Google"),
            Self::Cohere => write!(f, "Cohere"),
            Self::Mistral => write!(f, "Mistral"),
            Self::Local => write!(f, "Local"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// Types of simulated LLM failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMFailure {
    /// API rate limit exceeded (429)
    RateLimited { retry_after_secs: u64 },
    /// Service temporarily unavailable (503)
    ServiceUnavailable,
    /// Request timeout
    Timeout { duration_ms: u64 },
    /// Internal server error (500)
    InternalError,
    /// Bad gateway (502)
    BadGateway,
    /// Model overloaded
    ModelOverloaded,
    /// Invalid API key (401)
    AuthenticationError,
    /// Quota exceeded (402)
    QuotaExceeded,
    /// Content policy violation
    ContentFiltered,
    /// Network error
    NetworkError,
}

impl LLMFailure {
    /// Get HTTP status code for this failure.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::RateLimited { .. } => 429,
            Self::ServiceUnavailable => 503,
            Self::Timeout { .. } => 504,
            Self::InternalError => 500,
            Self::BadGateway => 502,
            Self::ModelOverloaded => 503,
            Self::AuthenticationError => 401,
            Self::QuotaExceeded => 402,
            Self::ContentFiltered => 400,
            Self::NetworkError => 0,
        }
    }

    /// Get error message.
    pub fn message(&self) -> String {
        match self {
            Self::RateLimited { retry_after_secs } => {
                format!("Rate limit exceeded. Retry after {} seconds.", retry_after_secs)
            }
            Self::ServiceUnavailable => "Service temporarily unavailable.".to_string(),
            Self::Timeout { duration_ms } => {
                format!("Request timed out after {}ms.", duration_ms)
            }
            Self::InternalError => "Internal server error.".to_string(),
            Self::BadGateway => "Bad gateway.".to_string(),
            Self::ModelOverloaded => "Model is currently overloaded. Try again later.".to_string(),
            Self::AuthenticationError => "Invalid API key.".to_string(),
            Self::QuotaExceeded => "API quota exceeded.".to_string(),
            Self::ContentFiltered => "Content filtered due to policy violation.".to_string(),
            Self::NetworkError => "Network connection error.".to_string(),
        }
    }
}

/// Provider-specific chaos configuration.
#[derive(Debug, Clone)]
pub struct ProviderChaosConfig {
    /// Probability of failure (0.0 - 1.0)
    pub failure_rate: f64,
    /// Types of failures to simulate
    pub failure_types: Vec<LLMFailure>,
    /// Latency injection range (min_ms, max_ms)
    pub latency_range_ms: Option<(u64, u64)>,
    /// Whether chaos is enabled for this provider
    pub enabled: bool,
}

impl Default for ProviderChaosConfig {
    fn default() -> Self {
        Self {
            failure_rate: 0.1, // 10% default
            failure_types: vec![
                LLMFailure::RateLimited { retry_after_secs: 30 },
                LLMFailure::ServiceUnavailable,
                LLMFailure::Timeout { duration_ms: 30000 },
            ],
            latency_range_ms: Some((100, 500)),
            enabled: true,
        }
    }
}

/// Chaos proxy configuration.
#[derive(Debug, Clone, Default)]
pub struct ChaosProxyConfig {
    /// Per-provider configurations
    pub providers: HashMap<LLMProvider, ProviderChaosConfig>,
    /// Global enabled flag
    pub enabled: bool,
}

impl ChaosProxyConfig {
    /// Create a new config with defaults.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            enabled: true,
        }
    }

    /// Add a provider with a specific failure rate.
    pub fn with_provider(mut self, provider: LLMProvider, failure_rate: f64) -> Self {
        self.providers.insert(provider, ProviderChaosConfig {
            failure_rate,
            ..Default::default()
        });
        self
    }

    /// Add a provider with full configuration.
    pub fn with_provider_config(mut self, provider: LLMProvider, config: ProviderChaosConfig) -> Self {
        self.providers.insert(provider, config);
        self
    }

    /// Create a "mild" chaos config for testing.
    pub fn mild() -> Self {
        Self::new()
            .with_provider(LLMProvider::OpenAI, 0.05)
            .with_provider(LLMProvider::Anthropic, 0.05)
    }

    /// Create a "moderate" chaos config.
    pub fn moderate() -> Self {
        Self::new()
            .with_provider(LLMProvider::OpenAI, 0.15)
            .with_provider(LLMProvider::Anthropic, 0.15)
            .with_provider(LLMProvider::Google, 0.10)
    }

    /// Create an "extreme" chaos config for stress testing.
    pub fn extreme() -> Self {
        Self::new()
            .with_provider(LLMProvider::OpenAI, 0.30)
            .with_provider(LLMProvider::Anthropic, 0.30)
            .with_provider(LLMProvider::Google, 0.25)
            .with_provider(LLMProvider::Cohere, 0.20)
    }
}

/// Chaos proxy statistics.
#[derive(Debug, Clone, Default)]
pub struct ChaosProxyStats {
    pub total_calls: u64,
    pub failures_injected: u64,
    pub latency_injected: u64,
    pub by_provider: HashMap<String, (u64, u64)>, // (total, failures)
}

/// Chaos Proxy for LLM provider failure simulation.
///
/// Wraps LLM API calls and randomly injects failures based on
/// configured probabilities to test system resilience.
pub struct ChaosProxy {
    config: ChaosProxyConfig,
    total_calls: AtomicU64,
    failures_injected: AtomicU64,
    latency_injected: AtomicU64,
    provider_stats: parking_lot::Mutex<HashMap<LLMProvider, (u64, u64)>>,
}

impl ChaosProxy {
    /// Create a new chaos proxy.
    pub fn new(config: ChaosProxyConfig) -> Self {
        Self {
            config,
            total_calls: AtomicU64::new(0),
            failures_injected: AtomicU64::new(0),
            latency_injected: AtomicU64::new(0),
            provider_stats: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    /// Create a disabled chaos proxy (passthrough).
    pub fn disabled() -> Self {
        Self::new(ChaosProxyConfig {
            enabled: false,
            ..Default::default()
        })
    }

    /// Check if chaos should be injected for this provider.
    /// Returns Ok(()) if call should proceed, Err(LLMFailure) if failure injected.
    pub async fn maybe_fail(&self, provider: LLMProvider) -> Result<(), LLMFailure> {
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        // Update per-provider stats
        {
            let mut stats = self.provider_stats.lock();
            let entry = stats.entry(provider).or_insert((0, 0));
            entry.0 += 1;
        }

        // Check global enable
        if !self.config.enabled {
            return Ok(());
        }

        // Get provider config
        let Some(provider_config) = self.config.providers.get(&provider) else {
            return Ok(()); // No config for this provider
        };

        if !provider_config.enabled {
            return Ok(());
        }

        let mut rng = rand::rng();

        // Inject latency if configured
        if let Some((min_ms, max_ms)) = provider_config.latency_range_ms {
            let latency = rng.random_range(min_ms..=max_ms);
            tokio::time::sleep(Duration::from_millis(latency)).await;
            self.latency_injected.fetch_add(1, Ordering::Relaxed);
        }

        // Check failure probability
        let roll: f64 = rng.random();
        if roll < provider_config.failure_rate {
            self.failures_injected.fetch_add(1, Ordering::Relaxed);

            // Update per-provider failure count
            {
                let mut stats = self.provider_stats.lock();
                if let Some(entry) = stats.get_mut(&provider) {
                    entry.1 += 1;
                }
            }

            // Select random failure type
            let failure_idx = rng.random_range(0..provider_config.failure_types.len());
            let failure = provider_config.failure_types[failure_idx].clone();

            tracing::warn!(
                provider = %provider,
                failure = ?failure,
                "Chaos proxy injected LLM failure"
            );

            return Err(failure);
        }

        Ok(())
    }

    /// Get current statistics.
    pub fn stats(&self) -> ChaosProxyStats {
        let provider_stats = self.provider_stats.lock();
        ChaosProxyStats {
            total_calls: self.total_calls.load(Ordering::Relaxed),
            failures_injected: self.failures_injected.load(Ordering::Relaxed),
            latency_injected: self.latency_injected.load(Ordering::Relaxed),
            by_provider: provider_stats
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
        }
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        self.total_calls.store(0, Ordering::Relaxed);
        self.failures_injected.store(0, Ordering::Relaxed);
        self.latency_injected.store(0, Ordering::Relaxed);
        self.provider_stats.lock().clear();
    }

    /// Get failure rate for a specific provider.
    pub fn failure_rate(&self, provider: LLMProvider) -> f64 {
        let stats = self.provider_stats.lock();
        if let Some((total, failures)) = stats.get(&provider) {
            if *total == 0 {
                return 0.0;
            }
            *failures as f64 / *total as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_disabled_proxy() {
        let proxy = ChaosProxy::disabled();
        
        for _ in 0..100 {
            let result = proxy.maybe_fail(LLMProvider::OpenAI).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_chaos_injection() {
        let config = ChaosProxyConfig::new()
            .with_provider_config(LLMProvider::OpenAI, ProviderChaosConfig {
                failure_rate: 1.0, // 100% failure for testing
                failure_types: vec![LLMFailure::RateLimited { retry_after_secs: 30 }],
                latency_range_ms: None,
                enabled: true,
            });
        
        let proxy = ChaosProxy::new(config);
        let result = proxy.maybe_fail(LLMProvider::OpenAI).await;
        
        assert!(result.is_err());
    }

    #[test]
    fn test_llm_failure_status_codes() {
        assert_eq!(LLMFailure::RateLimited { retry_after_secs: 30 }.status_code(), 429);
        assert_eq!(LLMFailure::ServiceUnavailable.status_code(), 503);
        assert_eq!(LLMFailure::AuthenticationError.status_code(), 401);
    }

    #[test]
    fn test_config_presets() {
        let mild = ChaosProxyConfig::mild();
        assert!(mild.providers.contains_key(&LLMProvider::OpenAI));
        
        let extreme = ChaosProxyConfig::extreme();
        assert_eq!(extreme.providers.get(&LLMProvider::OpenAI).unwrap().failure_rate, 0.30);
    }
}
