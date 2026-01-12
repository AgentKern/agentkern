//! Chaos Proxy for Inter-Agent & LLM Failure Simulation
//!
//! Per Antifragility Roadmap: "Third-Party API Mocking" AND "Inter-Agent Chaos"
//! Simulates failures of external LLM providers AND internal agent communication failures.
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_nexus::chaos_proxy::{ChaosProxy, ChaosConfig, ChaosTarget, LLMProvider};
//! use agentkern_nexus::types::Protocol;
//!
//! let config = ChaosConfig::default()
//!     .with_target(ChaosTarget::LLM(LLMProvider::OpenAI), 0.1)
//!     .with_target(ChaosTarget::Protocol(Protocol::GoogleA2A), 0.05);
//!
//! let proxy = ChaosProxy::new(config);
//!
//! // Check chaos for a target
//! match proxy.maybe_fail(ChaosTarget::Protocol(Protocol::GoogleA2A)).await {
//!     Ok(()) => { /* proceed */ }
//!     Err(e) => { /* handle simulated failure */ }
//! }
//! ```

use crate::types::Protocol;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Target for chaos injection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChaosTarget {
    /// External LLM Provider
    LLM(LLMProvider),
    /// Agent Protocol (e.g. A2A, MCP)
    Protocol(Protocol),
    /// Specific Agent ID
    Agent(String),
}

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

impl std::fmt::Display for ChaosTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LLM(p) => write!(f, "LLM::{}", p),
            Self::Protocol(p) => write!(f, "Protocol::{:?}", p),
            Self::Agent(id) => write!(f, "Agent::{}", id),
        }
    }
}

/// Types of simulated failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChaosFailure {
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
    /// Network connection error
    NetworkError,
    // --- LLM Specific ---
    /// Model overloaded
    ModelOverloaded,
    /// Invalid API key (401)
    AuthenticationError,
    /// Quota exceeded (402)
    QuotaExceeded,
    /// Content policy violation
    ContentFiltered,
}

impl ChaosFailure {
    /// Get HTTP status code for this failure.
    pub fn status_code(&self) -> u16 {
        match self {
            Self::RateLimited { .. } => 429,
            Self::ServiceUnavailable => 503,
            Self::Timeout { .. } => 504,
            Self::InternalError => 500,
            Self::BadGateway => 502,
            Self::NetworkError => 0,
            Self::ModelOverloaded => 503,
            Self::AuthenticationError => 401,
            Self::QuotaExceeded => 402,
            Self::ContentFiltered => 400,
        }
    }
}

/// Chaos configuration for a specific target.
#[derive(Debug, Clone)]
pub struct TargetChaosConfig {
    /// Probability of failure (0.0 - 1.0)
    pub failure_rate: f64,
    /// Types of failures to simulate
    pub failure_types: Vec<ChaosFailure>,
    /// Latency injection range (min_ms, max_ms)
    pub latency_range_ms: Option<(u64, u64)>,
    /// Whether chaos is enabled for this target
    pub enabled: bool,
}

impl Default for TargetChaosConfig {
    fn default() -> Self {
        Self {
            failure_rate: 0.1, // 10% default
            failure_types: vec![
                ChaosFailure::RateLimited {
                    retry_after_secs: 5,
                },
                ChaosFailure::ServiceUnavailable,
                ChaosFailure::Timeout { duration_ms: 1000 },
            ],
            latency_range_ms: Some((50, 200)),
            enabled: true,
        }
    }
}

/// Global chaos proxy configuration.
#[derive(Debug, Clone, Default)]
pub struct ChaosConfig {
    /// Per-target configurations
    pub targets: HashMap<ChaosTarget, TargetChaosConfig>,
    /// Global enabled flag
    pub enabled: bool,
}

impl ChaosConfig {
    /// Create a new config.
    pub fn new() -> Self {
        Self {
            targets: HashMap::new(),
            enabled: true,
        }
    }

    /// Add a target with a specific failure rate.
    pub fn with_target(mut self, target: ChaosTarget, failure_rate: f64) -> Self {
        self.targets.insert(
            target,
            TargetChaosConfig {
                failure_rate,
                ..Default::default()
            },
        );
        self
    }

    /// Add a generic protocol target.
    pub fn with_protocol(self, protocol: Protocol, failure_rate: f64) -> Self {
        self.with_target(ChaosTarget::Protocol(protocol), failure_rate)
    }

    /// Add an LLM provider target.
    pub fn with_provider(self, provider: LLMProvider, failure_rate: f64) -> Self {
        self.with_target(ChaosTarget::LLM(provider), failure_rate)
    }
}

/// Chaos proxy statistics.
#[derive(Debug, Clone, Default)]
pub struct ChaosStats {
    pub total_calls: u64,
    pub failures_injected: u64,
    pub latency_injected: u64,
    pub by_target: HashMap<String, (u64, u64)>, // (total, failures)
}

/// Chaos Proxy for failure simulation.
pub struct ChaosProxy {
    config: ChaosConfig,
    total_calls: AtomicU64,
    failures_injected: AtomicU64,
    latency_injected: AtomicU64,
    target_stats: parking_lot::Mutex<HashMap<ChaosTarget, (u64, u64)>>,
}

impl ChaosProxy {
    /// Create a new chaos proxy.
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config,
            total_calls: AtomicU64::new(0),
            failures_injected: AtomicU64::new(0),
            latency_injected: AtomicU64::new(0),
            target_stats: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    /// Create a disabled chaos proxy.
    pub fn disabled() -> Self {
        Self::new(ChaosConfig {
            enabled: false,
            ..Default::default()
        })
    }

    /// Check if chaos should be injected.
    pub async fn maybe_fail(&self, target: ChaosTarget) -> Result<(), ChaosFailure> {
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        // Update stats
        {
            let mut stats = self.target_stats.lock();
            let entry = stats.entry(target.clone()).or_insert((0, 0));
            entry.0 += 1;
        }

        if !self.config.enabled {
            return Ok(());
        }

        let Some(target_config) = self.config.targets.get(&target) else {
            return Ok(());
        };

        if !target_config.enabled {
            return Ok(());
        }

        let mut rng = rand::rng();

        // Inject latency
        if let Some((min_ms, max_ms)) = target_config.latency_range_ms {
            let latency = rng.random_range(min_ms..=max_ms);
            tokio::time::sleep(Duration::from_millis(latency)).await;
            self.latency_injected.fetch_add(1, Ordering::Relaxed);
        }

        // Inject failure
        let roll: f64 = rng.random();
        if roll < target_config.failure_rate {
            self.failures_injected.fetch_add(1, Ordering::Relaxed);

            // Update stats
            {
                let mut stats = self.target_stats.lock();
                if let Some(entry) = stats.get_mut(&target) {
                    entry.1 += 1;
                }
            }

            let failure_idx = rng.random_range(0..target_config.failure_types.len());
            let failure = target_config.failure_types[failure_idx].clone();

            tracing::warn!(target = %target, failure = ?failure, "Chaos Injection Triggered");
            return Err(failure);
        }

        Ok(())
    }

    /// Get current chaos statistics.
    pub fn stats(&self) -> ChaosStats {
        let stats_map = self.target_stats.lock().clone();
        let mut by_target_str = HashMap::new();
        
        for (k, v) in stats_map {
            by_target_str.insert(k.to_string(), v);
        }

        ChaosStats {
            total_calls: self.total_calls.load(Ordering::Relaxed),
            failures_injected: self.failures_injected.load(Ordering::Relaxed),
            latency_injected: self.latency_injected.load(Ordering::Relaxed),
            by_target: by_target_str,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_protocol_chaos() {
        let config = ChaosConfig::new().with_target(ChaosTarget::Protocol(Protocol::GoogleA2A), 1.0);
        let proxy = ChaosProxy::new(config);

        let result = proxy.maybe_fail(ChaosTarget::Protocol(Protocol::GoogleA2A)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_selective_chaos() {
        let config = ChaosConfig::new().with_target(ChaosTarget::Protocol(Protocol::GoogleA2A), 1.0);
        let proxy = ChaosProxy::new(config);

        // Should fail
        let res1 = proxy.maybe_fail(ChaosTarget::Protocol(Protocol::GoogleA2A)).await;
        assert!(res1.is_err());

        // Should pass (no config)
        let res2 = proxy.maybe_fail(ChaosTarget::Protocol(Protocol::AnthropicMCP)).await;
        assert!(res2.is_ok());
    }
}
