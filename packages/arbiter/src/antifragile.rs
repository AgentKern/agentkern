//! Antifragile Engine
//!
//! Per Roadmap: "Anti-Fragile Self-Healing Engine - make agents stronger through failure"
//! Per MANDATE.md Section 6: "Kill Switch: Hardware-level Red Button"
//!
//! This engine learns from failures and makes the system more resilient.
//!
//! OPEN SOURCE: Core antifragile patterns
//! - Failure memory and learning
//! - Circuit breaker with recovery
//! - Adaptive rate limiting
//!
//! ENTERPRISE (ee/resilience):
//! - Cross-fleet failure correlation
//! - Predictive failure detection (ML)
//! - Automated runbook execution

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// SEVERITY & CATEGORY (merged from synapse/antifragile.rs)
// ============================================================================

/// Failure severity levels - how serious is this failure?
/// Merged from synapse for "growing stronger through failure" pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FailureSeverity {
    /// Minor issue, auto-recoverable
    Minor,
    /// Moderate issue, needs intervention
    Moderate,
    /// Major issue, significant impact
    Major,
    /// Critical issue, system-wide impact
    Critical,
}

/// Failure category for pattern matching - more granular than FailureClass.
/// Merged from synapse for intelligent recovery strategy selection.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureCategory {
    /// Network-related failures
    Network,
    /// Timeout failures
    Timeout,
    /// Resource exhaustion (memory, CPU)
    ResourceExhaustion,
    /// External API failures
    ExternalApi,
    /// Data corruption
    DataCorruption,
    /// Policy violation
    PolicyViolation,
    /// Rate limiting
    RateLimit,
    /// Authentication failure
    AuthFailure,
    /// Unknown/other
    Other(String),
}

impl FailureCategory {
    /// Get recommended recovery strategy based on category and attempt count.
    /// This is the "growing stronger" logic - learns from repeated failures.
    pub fn recommend_strategy(&self, attempt: u32) -> RecoveryStrategyType {
        match (self, attempt) {
            (FailureCategory::Network, 0..=2) => RecoveryStrategyType::Retry,
            (FailureCategory::Network, 3..=5) => RecoveryStrategyType::ExponentialBackoff,
            (FailureCategory::Network, _) => RecoveryStrategyType::CircuitBreaker,

            (FailureCategory::Timeout, 0..=1) => RecoveryStrategyType::Retry,
            (FailureCategory::Timeout, _) => RecoveryStrategyType::CircuitBreaker,

            (FailureCategory::RateLimit, _) => RecoveryStrategyType::ExponentialBackoff,

            (FailureCategory::ExternalApi, 0..=2) => RecoveryStrategyType::Fallback,
            (FailureCategory::ExternalApi, _) => RecoveryStrategyType::GracefulDegradation,

            (FailureCategory::ResourceExhaustion, _) => RecoveryStrategyType::GracefulDegradation,

            (FailureCategory::DataCorruption, _) => RecoveryStrategyType::Rollback,

            (FailureCategory::PolicyViolation, _) => RecoveryStrategyType::Skip,

            (FailureCategory::AuthFailure, 0..=1) => RecoveryStrategyType::Retry,
            (FailureCategory::AuthFailure, _) => RecoveryStrategyType::HumanIntervention,

            (FailureCategory::Other(_), 0..=2) => RecoveryStrategyType::Retry,
            (FailureCategory::Other(_), _) => RecoveryStrategyType::HumanIntervention,
        }
    }
}

/// Recovery strategy types for automatic selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryStrategyType {
    Retry,
    ExponentialBackoff,
    Fallback,
    CircuitBreaker,
    GracefulDegradation,
    HumanIntervention,
    Rollback,
    Skip,
}

// ============================================================================
// ADAPTATION RATE (merged from synapse/antifragile.rs)
// ============================================================================

/// Learning rate for adaptation - implements "growing stronger through failure".
/// Systems boost their response rate after failures, decay after successes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRate {
    /// Base learning rate
    pub base_rate: f64,
    /// Current multiplier
    pub multiplier: f64,
    /// Maximum multiplier
    pub max_multiplier: f64,
    /// Decay factor per success
    pub decay: f64,
    /// Boost factor per failure
    pub boost_factor: f64,
}

impl Default for AdaptationRate {
    fn default() -> Self {
        Self {
            base_rate: 1.0,
            multiplier: 1.0,
            max_multiplier: 10.0,
            decay: 0.9,
            boost_factor: 1.5,
        }
    }
}

impl AdaptationRate {
    /// Get current effective rate.
    pub fn current(&self) -> f64 {
        self.base_rate * self.multiplier
    }

    /// Increase rate (on failure) - system becomes more sensitive.
    pub fn boost(&mut self) {
        self.multiplier = (self.multiplier * self.boost_factor).min(self.max_multiplier);
    }

    /// Decrease rate (on success) - system calms down.
    pub fn decay_rate(&mut self) {
        self.multiplier = (self.multiplier * self.decay).max(1.0);
    }
}

// ============================================================================
// FAILURE CLASSIFICATION (original Arbiter)
// ============================================================================

/// Failure classification (legacy, kept for compatibility).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureClass {
    /// Network-related failures
    Network,
    /// Timeout failures
    Timeout,
    /// Resource exhaustion (memory, CPU)
    ResourceExhaustion,
    /// External service unavailable
    ServiceUnavailable,
    /// Data validation errors
    ValidationError,
    /// Policy violations
    PolicyViolation,
    /// Unknown/unclassified
    Unknown,
}

impl FailureClass {
    /// Classify from error message.
    pub fn from_error(msg: &str) -> Self {
        let lower = msg.to_lowercase();

        if lower.contains("timeout") || lower.contains("timed out") {
            Self::Timeout
        } else if lower.contains("connection") || lower.contains("network") || lower.contains("dns")
        {
            Self::Network
        } else if lower.contains("memory") || lower.contains("resource") || lower.contains("limit")
        {
            Self::ResourceExhaustion
        } else if lower.contains("unavailable") || lower.contains("503") || lower.contains("502") {
            Self::ServiceUnavailable
        } else if lower.contains("validation")
            || lower.contains("invalid")
            || lower.contains("parse")
        {
            Self::ValidationError
        } else if lower.contains("policy")
            || lower.contains("denied")
            || lower.contains("forbidden")
        {
            Self::PolicyViolation
        } else {
            Self::Unknown
        }
    }

    /// Convert to FailureCategory for strategy recommendation.
    pub fn to_category(&self) -> FailureCategory {
        match self {
            Self::Network => FailureCategory::Network,
            Self::Timeout => FailureCategory::Timeout,
            Self::ResourceExhaustion => FailureCategory::ResourceExhaustion,
            Self::ServiceUnavailable => FailureCategory::ExternalApi,
            Self::ValidationError => FailureCategory::Other("validation".into()),
            Self::PolicyViolation => FailureCategory::PolicyViolation,
            Self::Unknown => FailureCategory::Other("unknown".into()),
        }
    }
}

/// A recorded failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    /// Failure ID
    pub id: String,
    /// Service/component that failed
    pub service: String,
    /// Failure class
    pub class: FailureClass,
    /// Error message
    pub message: String,
    /// When it occurred
    pub timestamp: DateTime<Utc>,
    /// Context data
    pub context: HashMap<String, String>,
    /// Was recovery successful?
    pub recovered: bool,
    /// Recovery strategy used
    pub recovery_strategy: Option<String>,
}

impl Failure {
    /// Create a new failure record.
    pub fn new(service: impl Into<String>, message: impl Into<String>) -> Self {
        let msg = message.into();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            service: service.into(),
            class: FailureClass::from_error(&msg),
            message: msg,
            timestamp: Utc::now(),
            context: HashMap::new(),
            recovered: false,
            recovery_strategy: None,
        }
    }

    /// Add context.
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Mark as recovered.
    pub fn mark_recovered(mut self, strategy: impl Into<String>) -> Self {
        self.recovered = true;
        self.recovery_strategy = Some(strategy.into());
        self
    }
}

/// Recovery strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// Strategy name
    pub name: String,
    /// Applicable failure classes
    pub applies_to: Vec<FailureClass>,
    /// Priority (higher = try first)
    pub priority: u8,
    /// Success rate (0-100)
    pub success_rate: u8,
    /// Average recovery time in ms
    pub avg_recovery_ms: u64,
}

impl RecoveryStrategy {
    /// Create a new strategy.
    pub fn new(name: impl Into<String>, applies_to: Vec<FailureClass>) -> Self {
        Self {
            name: name.into(),
            applies_to,
            priority: 50,
            success_rate: 50,
            avg_recovery_ms: 1000,
        }
    }

    /// Check if strategy applies to a failure class.
    pub fn applies(&self, class: &FailureClass) -> bool {
        self.applies_to.contains(class)
    }
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Too many failures, rejecting requests
    Open,
    /// Testing if service recovered
    HalfOpen,
}

/// Circuit breaker.
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    last_failure: Option<DateTime<Utc>>,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold: 5,
            success_threshold: 3,
            last_failure: None,
            reset_timeout: Duration::seconds(30),
        }
    }

    /// Check if requests should be allowed.
    pub fn is_allowed(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last) = self.last_failure {
                    if Utc::now() - last > self.reset_timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a success.
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    tracing::info!(circuit = %self.name, "Circuit closed - service recovered");
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failure.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Utc::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    tracing::warn!(circuit = %self.name, "Circuit opened - too many failures");
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                tracing::warn!(circuit = %self.name, "Circuit reopened - recovery failed");
            }
            CircuitState::Open => {}
        }
    }

    /// Get current state.
    pub fn state(&self) -> CircuitState {
        self.state
    }
}

/// Antifragile engine - learns from failures.
pub struct AntifragileEngine {
    /// Failure history
    failures: Arc<RwLock<Vec<Failure>>>,
    /// Circuit breakers by service
    circuits: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    /// Recovery strategies
    strategies: Vec<RecoveryStrategy>,
    /// Failure count by class (for learning)
    failure_stats: Arc<RwLock<HashMap<FailureClass, u32>>>,
}

impl AntifragileEngine {
    /// Create a new antifragile engine with default strategies.
    pub fn new() -> Self {
        let strategies = vec![
            RecoveryStrategy {
                name: "retry_with_backoff".into(),
                applies_to: vec![
                    FailureClass::Network,
                    FailureClass::Timeout,
                    FailureClass::ServiceUnavailable,
                ],
                priority: 80,
                success_rate: 70,
                avg_recovery_ms: 2000,
            },
            RecoveryStrategy {
                name: "failover_to_replica".into(),
                applies_to: vec![FailureClass::ServiceUnavailable],
                priority: 90,
                success_rate: 85,
                avg_recovery_ms: 500,
            },
            RecoveryStrategy {
                name: "reduce_load".into(),
                applies_to: vec![FailureClass::ResourceExhaustion],
                priority: 70,
                success_rate: 60,
                avg_recovery_ms: 5000,
            },
            RecoveryStrategy {
                name: "return_cached".into(),
                applies_to: vec![FailureClass::Network, FailureClass::Timeout],
                priority: 50,
                success_rate: 90,
                avg_recovery_ms: 10,
            },
        ];

        Self {
            failures: Arc::new(RwLock::new(Vec::new())),
            circuits: Arc::new(RwLock::new(HashMap::new())),
            strategies,
            failure_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a failure and get recommended recovery.
    pub async fn handle_failure(&self, failure: Failure) -> Option<RecoveryStrategy> {
        // Record failure
        {
            let mut failures = self.failures.write().await;
            failures.push(failure.clone());

            // Keep only last 1000 failures
            if failures.len() > 1000 {
                failures.remove(0);
            }
        }

        // Update stats
        {
            let mut stats = self.failure_stats.write().await;
            *stats.entry(failure.class.clone()).or_insert(0) += 1;
        }

        // Update circuit breaker
        {
            let mut circuits = self.circuits.write().await;
            let circuit = circuits
                .entry(failure.service.clone())
                .or_insert_with(|| CircuitBreaker::new(&failure.service));
            circuit.record_failure();
        }

        // Find best recovery strategy
        self.find_strategy(&failure.class)
    }

    /// Find the best recovery strategy for a failure class.
    fn find_strategy(&self, class: &FailureClass) -> Option<RecoveryStrategy> {
        self.strategies
            .iter()
            .filter(|s| s.applies(class))
            .max_by_key(|s| (s.priority, s.success_rate))
            .cloned()
    }

    /// Record a successful recovery.
    pub async fn record_recovery(&self, service: &str) {
        let mut circuits = self.circuits.write().await;
        if let Some(circuit) = circuits.get_mut(service) {
            circuit.record_success();
        }
    }

    /// Check if a service is available (circuit not open).
    pub async fn is_service_available(&self, service: &str) -> bool {
        let mut circuits = self.circuits.write().await;
        let circuit = circuits
            .entry(service.to_string())
            .or_insert_with(|| CircuitBreaker::new(service));
        circuit.is_allowed()
    }

    /// Get failure statistics.
    pub async fn get_stats(&self) -> HashMap<FailureClass, u32> {
        self.failure_stats.read().await.clone()
    }

    /// Get recent failures for a service.
    pub async fn get_recent_failures(&self, service: &str, limit: usize) -> Vec<Failure> {
        let failures = self.failures.read().await;
        failures
            .iter()
            .rev()
            .filter(|f| f.service == service)
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for AntifragileEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failure_classification() {
        assert_eq!(
            FailureClass::from_error("Connection timeout"),
            FailureClass::Timeout
        );
        assert_eq!(
            FailureClass::from_error("Network unreachable"),
            FailureClass::Network
        );
        assert_eq!(
            FailureClass::from_error("Out of memory"),
            FailureClass::ResourceExhaustion
        );
        assert_eq!(
            FailureClass::from_error("Service unavailable 503"),
            FailureClass::ServiceUnavailable
        );
    }

    #[test]
    fn test_circuit_breaker_normal() {
        let mut cb = CircuitBreaker::new("test-service");

        assert!(cb.is_allowed());
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens() {
        let mut cb = CircuitBreaker::new("failing-service");

        // Record failures up to threshold
        for _ in 0..5 {
            cb.record_failure();
        }

        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.is_allowed());
    }

    #[tokio::test]
    async fn test_antifragile_handle_failure() {
        let engine = AntifragileEngine::new();

        let failure = Failure::new("api-service", "Connection timeout");
        let strategy = engine.handle_failure(failure).await;

        assert!(strategy.is_some());
        let strat = strategy.unwrap();
        assert!(strat.applies(&FailureClass::Timeout));
    }

    #[tokio::test]
    async fn test_antifragile_service_availability() {
        let engine = AntifragileEngine::new();

        // Service should be available initially
        assert!(engine.is_service_available("new-service").await);

        // Record many failures
        for _ in 0..5 {
            let failure = Failure::new("failing-service", "Service unavailable");
            engine.handle_failure(failure).await;
        }

        // Circuit should be open now
        assert!(!engine.is_service_available("failing-service").await);
    }

    #[tokio::test]
    async fn test_recovery_recording() {
        let engine = AntifragileEngine::new();

        // Create failure then recover
        let failure = Failure::new("recovering-service", "Timeout");
        engine.handle_failure(failure).await;

        engine.record_recovery("recovering-service").await;

        // Should still be available
        assert!(engine.is_service_available("recovering-service").await);
    }

    #[test]
    fn test_recovery_strategy_selection() {
        let engine = AntifragileEngine::new();

        // Network failures should get retry strategy
        let strat = engine.find_strategy(&FailureClass::Network);
        assert!(strat.is_some());

        // Resource exhaustion gets reduce_load
        let strat = engine.find_strategy(&FailureClass::ResourceExhaustion);
        assert!(strat.is_some());
        assert!(strat.unwrap().name.contains("reduce"));
    }
}
