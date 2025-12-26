//! VeriMantle-Synapse: Anti-Fragile Self-Healing Engine
//!
//! Per FUTURE_INNOVATION_ROADMAP.md Innovation #4:
//! - Systems that grow STRONGER through failure
//! - Dynamic learning rate adaptation
//! - Intelligent circuit breakers
//! - Failure memory graph
//! - Recovery strategy selection
//!
//! Unlike resilient systems (resist failure) or robust systems (survive failure),
//! anti-fragile systems IMPROVE from volatility, stress, and disruption.

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

// ============================================================================
// CORE TYPES
// ============================================================================

/// Failure severity levels.
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

/// Failure category for pattern matching.
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

/// A recorded failure event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureEvent {
    /// Unique ID
    pub id: String,
    /// Agent that experienced the failure
    pub agent_id: String,
    /// Failure category
    pub category: FailureCategory,
    /// Severity level
    pub severity: FailureSeverity,
    /// Error message
    pub message: String,
    /// Context (action being performed, etc.)
    pub context: HashMap<String, String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Was recovery successful?
    pub recovered: bool,
    /// Recovery strategy used
    pub recovery_strategy: Option<RecoveryStrategy>,
    /// Time to recover (if recovered)
    pub recovery_time_ms: Option<u64>,
}

/// Recovery strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Simple retry
    Retry,
    /// Retry with exponential backoff
    ExponentialBackoff,
    /// Fallback to alternative
    Fallback,
    /// Circuit breaker (stop trying)
    CircuitBreaker,
    /// Graceful degradation
    GracefulDegradation,
    /// Request human intervention
    HumanIntervention,
    /// Rollback to previous state
    Rollback,
    /// Skip and continue
    Skip,
}

impl RecoveryStrategy {
    /// Get recommended strategy for a failure category.
    pub fn recommend_for(category: &FailureCategory, attempt: u32) -> Self {
        match (category, attempt) {
            (FailureCategory::Network, 0..=2) => RecoveryStrategy::Retry,
            (FailureCategory::Network, 3..=5) => RecoveryStrategy::ExponentialBackoff,
            (FailureCategory::Network, _) => RecoveryStrategy::CircuitBreaker,
            
            (FailureCategory::Timeout, 0..=1) => RecoveryStrategy::Retry,
            (FailureCategory::Timeout, _) => RecoveryStrategy::CircuitBreaker,
            
            (FailureCategory::RateLimit, _) => RecoveryStrategy::ExponentialBackoff,
            
            (FailureCategory::ExternalApi, 0..=2) => RecoveryStrategy::Fallback,
            (FailureCategory::ExternalApi, _) => RecoveryStrategy::GracefulDegradation,
            
            (FailureCategory::ResourceExhaustion, _) => RecoveryStrategy::GracefulDegradation,
            
            (FailureCategory::DataCorruption, _) => RecoveryStrategy::Rollback,
            
            (FailureCategory::PolicyViolation, _) => RecoveryStrategy::Skip,
            
            (FailureCategory::AuthFailure, 0..=1) => RecoveryStrategy::Retry,
            (FailureCategory::AuthFailure, _) => RecoveryStrategy::HumanIntervention,
            
            (FailureCategory::Other(_), 0..=2) => RecoveryStrategy::Retry,
            (FailureCategory::Other(_), _) => RecoveryStrategy::HumanIntervention,
        }
    }
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Failing, limited retries
    HalfOpen,
    /// No retries, fail fast
    Open,
}

/// Circuit breaker for a specific operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// Circuit name/ID
    pub name: String,
    /// Current state
    pub state: CircuitState,
    /// Failure count
    pub failure_count: u32,
    /// Success count (in half-open)
    pub success_count: u32,
    /// Threshold to open circuit
    pub failure_threshold: u32,
    /// Threshold to close circuit
    pub success_threshold: u32,
    /// When circuit opened
    pub opened_at: Option<DateTime<Utc>>,
    /// Cooldown period before half-open
    pub cooldown_seconds: u32,
}

impl CircuitBreaker {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold: 5,
            success_threshold: 3,
            opened_at: None,
            cooldown_seconds: 30,
        }
    }

    /// Record a failure.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.success_count = 0;
        
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            self.opened_at = Some(Utc::now());
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
                    self.success_count = 0;
                    self.opened_at = None;
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Check if request should be allowed.
    pub fn should_allow(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true, // Allow probe requests
            CircuitState::Open => {
                // Check if cooldown has passed
                if let Some(opened_at) = self.opened_at {
                    let elapsed = Utc::now() - opened_at;
                    if elapsed > Duration::seconds(self.cooldown_seconds as i64) {
                        self.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
        }
    }
}

/// Learning rate for adaptation.
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
    pub boost: f64,
}

impl Default for AdaptationRate {
    fn default() -> Self {
        Self {
            base_rate: 1.0,
            multiplier: 1.0,
            max_multiplier: 10.0,
            decay: 0.9,
            boost: 1.5,
        }
    }
}

impl AdaptationRate {
    /// Get current effective rate.
    pub fn current(&self) -> f64 {
        self.base_rate * self.multiplier
    }

    /// Increase rate (on failure).
    pub fn boost(&mut self) {
        self.multiplier = (self.multiplier * self.boost).min(self.max_multiplier);
    }

    /// Decrease rate (on success).
    pub fn decay(&mut self) {
        self.multiplier = (self.multiplier * self.decay).max(1.0);
    }
}

// ============================================================================
// ANTI-FRAGILE ENGINE
// ============================================================================

/// The Anti-Fragile Self-Healing Engine.
pub struct AntifragileEngine {
    /// Failure history
    failures: Arc<RwLock<Vec<FailureEvent>>>,
    /// Circuit breakers by operation
    circuits: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    /// Adaptation rates by category
    adaptation: Arc<RwLock<HashMap<FailureCategory, AdaptationRate>>>,
    /// Recovery success rates by strategy
    strategy_stats: Arc<RwLock<HashMap<RecoveryStrategy, (u32, u32)>>>, // (success, total)
    /// Maximum failure history
    max_history: usize,
}

impl Default for AntifragileEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AntifragileEngine {
    /// Create a new anti-fragile engine.
    pub fn new() -> Self {
        Self {
            failures: Arc::new(RwLock::new(Vec::new())),
            circuits: Arc::new(RwLock::new(HashMap::new())),
            adaptation: Arc::new(RwLock::new(HashMap::new())),
            strategy_stats: Arc::new(RwLock::new(HashMap::new())),
            max_history: 10_000,
        }
    }

    /// Handle a failure event.
    pub fn handle_failure(
        &self,
        agent_id: &str,
        category: FailureCategory,
        severity: FailureSeverity,
        message: &str,
        context: HashMap<String, String>,
    ) -> RecoveryStrategy {
        // Find similar past failures
        let attempt = self.count_recent_failures(agent_id, &category);
        
        // Select recovery strategy
        let strategy = self.select_strategy(&category, attempt);
        
        // Record failure
        let failure = FailureEvent {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            category: category.clone(),
            severity,
            message: message.to_string(),
            context,
            timestamp: Utc::now(),
            recovered: false,
            recovery_strategy: Some(strategy),
            recovery_time_ms: None,
        };
        
        {
            let mut failures = self.failures.write();
            failures.push(failure);
            if failures.len() > self.max_history {
                failures.remove(0);
            }
        }
        
        // Update circuit breaker
        self.update_circuit(&format!("{}:{:?}", agent_id, category), false);
        
        // Boost adaptation rate
        self.boost_adaptation(&category);
        
        strategy
    }

    /// Record successful recovery.
    pub fn record_recovery(&self, failure_id: &str, recovery_time_ms: u64) {
        let mut failures = self.failures.write();
        if let Some(failure) = failures.iter_mut().find(|f| f.id == failure_id) {
            failure.recovered = true;
            failure.recovery_time_ms = Some(recovery_time_ms);
            
            // Update strategy stats
            if let Some(strategy) = failure.recovery_strategy {
                let mut stats = self.strategy_stats.write();
                let entry = stats.entry(strategy).or_insert((0, 0));
                entry.0 += 1; // success
                entry.1 += 1; // total
            }
            
            // Update circuit
            self.update_circuit(
                &format!("{}:{:?}", failure.agent_id, failure.category),
                true
            );
            
            // Decay adaptation rate
            self.decay_adaptation(&failure.category);
        }
    }

    /// Select best recovery strategy based on history.
    fn select_strategy(&self, category: &FailureCategory, attempt: u32) -> RecoveryStrategy {
        // Get strategy success rates
        let stats = self.strategy_stats.read();
        
        // Start with recommended strategy
        let recommended = RecoveryStrategy::recommend_for(category, attempt);
        
        // Check if we have enough data to override
        if let Some((success, total)) = stats.get(&recommended) {
            if *total >= 10 {
                let success_rate = *success as f64 / *total as f64;
                if success_rate < 0.3 {
                    // Try alternative strategies
                    return self.find_best_alternative(category, recommended);
                }
            }
        }
        
        recommended
    }

    /// Find best alternative strategy.
    fn find_best_alternative(&self, _category: &FailureCategory, current: RecoveryStrategy) -> RecoveryStrategy {
        let stats = self.strategy_stats.read();
        
        let alternatives = [
            RecoveryStrategy::Retry,
            RecoveryStrategy::ExponentialBackoff,
            RecoveryStrategy::Fallback,
            RecoveryStrategy::GracefulDegradation,
        ];
        
        let mut best = current;
        let mut best_rate = 0.0;
        
        for strategy in alternatives {
            if strategy == current {
                continue;
            }
            if let Some((success, total)) = stats.get(&strategy) {
                if *total >= 5 {
                    let rate = *success as f64 / *total as f64;
                    if rate > best_rate {
                        best = strategy;
                        best_rate = rate;
                    }
                }
            }
        }
        
        best
    }

    /// Count recent failures for an agent/category.
    fn count_recent_failures(&self, agent_id: &str, category: &FailureCategory) -> u32 {
        let failures = self.failures.read();
        let cutoff = Utc::now() - Duration::minutes(5);
        
        failures.iter()
            .filter(|f| {
                f.agent_id == agent_id &&
                &f.category == category &&
                f.timestamp > cutoff
            })
            .count() as u32
    }

    /// Update circuit breaker.
    fn update_circuit(&self, circuit_name: &str, success: bool) {
        let mut circuits = self.circuits.write();
        let circuit = circuits.entry(circuit_name.to_string())
            .or_insert_with(|| CircuitBreaker::new(circuit_name));
        
        if success {
            circuit.record_success();
        } else {
            circuit.record_failure();
        }
    }

    /// Boost adaptation rate for category.
    fn boost_adaptation(&self, category: &FailureCategory) {
        let mut adaptation = self.adaptation.write();
        let rate = adaptation.entry(category.clone())
            .or_insert_with(AdaptationRate::default);
        rate.boost();
    }

    /// Decay adaptation rate for category.
    fn decay_adaptation(&self, category: &FailureCategory) {
        let mut adaptation = self.adaptation.write();
        if let Some(rate) = adaptation.get_mut(category) {
            rate.decay();
        }
    }

    /// Check if operation should be allowed (circuit breaker).
    pub fn should_allow(&self, operation: &str) -> bool {
        let mut circuits = self.circuits.write();
        if let Some(circuit) = circuits.get_mut(operation) {
            circuit.should_allow()
        } else {
            true // No circuit means allow
        }
    }

    /// Get current adaptation rate for category.
    pub fn get_adaptation_rate(&self, category: &FailureCategory) -> f64 {
        let adaptation = self.adaptation.read();
        adaptation.get(category)
            .map(|r| r.current())
            .unwrap_or(1.0)
    }

    /// Get failure statistics.
    pub fn get_stats(&self) -> AntifragileStats {
        let failures = self.failures.read();
        let circuits = self.circuits.read();
        let strategy_stats = self.strategy_stats.read();
        
        let total_failures = failures.len() as u32;
        let recovered = failures.iter().filter(|f| f.recovered).count() as u32;
        let open_circuits = circuits.values().filter(|c| c.state == CircuitState::Open).count() as u32;
        
        let mut best_strategy = None;
        let mut best_rate = 0.0;
        for (strategy, (success, total)) in strategy_stats.iter() {
            if *total >= 5 {
                let rate = *success as f64 / *total as f64;
                if rate > best_rate {
                    best_strategy = Some(*strategy);
                    best_rate = rate;
                }
            }
        }
        
        AntifragileStats {
            total_failures,
            recovered_failures: recovered,
            recovery_rate: if total_failures > 0 { recovered as f64 / total_failures as f64 } else { 1.0 },
            open_circuits,
            best_strategy,
        }
    }

    /// Get recent failures.
    pub fn get_recent_failures(&self, limit: usize) -> Vec<FailureEvent> {
        let failures = self.failures.read();
        failures.iter().rev().take(limit).cloned().collect()
    }
}

/// Statistics from the anti-fragile engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntifragileStats {
    pub total_failures: u32,
    pub recovered_failures: u32,
    pub recovery_rate: f64,
    pub open_circuits: u32,
    pub best_strategy: Option<RecoveryStrategy>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker() {
        let mut circuit = CircuitBreaker::new("test");
        
        assert!(circuit.should_allow());
        
        // Trigger failures
        for _ in 0..5 {
            circuit.record_failure();
        }
        
        assert_eq!(circuit.state, CircuitState::Open);
        assert!(!circuit.should_allow());
    }

    #[test]
    fn test_circuit_recovery() {
        let mut circuit = CircuitBreaker::new("test");
        circuit.failure_threshold = 2;
        circuit.success_threshold = 2;
        
        circuit.record_failure();
        circuit.record_failure();
        assert_eq!(circuit.state, CircuitState::Open);
        
        // Manually transition to half-open for test
        circuit.state = CircuitState::HalfOpen;
        
        circuit.record_success();
        circuit.record_success();
        assert_eq!(circuit.state, CircuitState::Closed);
    }

    #[test]
    fn test_adaptation_rate() {
        let mut rate = AdaptationRate::default();
        
        assert_eq!(rate.current(), 1.0);
        
        rate.boost();
        assert!(rate.current() > 1.0);
        
        rate.decay();
        assert!(rate.current() < rate.max_multiplier);
    }

    #[test]
    fn test_recovery_strategy_recommendation() {
        assert_eq!(
            RecoveryStrategy::recommend_for(&FailureCategory::Network, 0),
            RecoveryStrategy::Retry
        );
        
        assert_eq!(
            RecoveryStrategy::recommend_for(&FailureCategory::Network, 10),
            RecoveryStrategy::CircuitBreaker
        );
        
        assert_eq!(
            RecoveryStrategy::recommend_for(&FailureCategory::RateLimit, 5),
            RecoveryStrategy::ExponentialBackoff
        );
    }

    #[test]
    fn test_engine_failure_handling() {
        let engine = AntifragileEngine::new();
        
        let strategy = engine.handle_failure(
            "agent-1",
            FailureCategory::Network,
            FailureSeverity::Minor,
            "Connection refused",
            HashMap::new(),
        );
        
        assert_eq!(strategy, RecoveryStrategy::Retry);
        
        let stats = engine.get_stats();
        assert_eq!(stats.total_failures, 1);
    }

    #[test]
    fn test_recovery_tracking() {
        let engine = AntifragileEngine::new();
        
        let strategy = engine.handle_failure(
            "agent-1",
            FailureCategory::Timeout,
            FailureSeverity::Minor,
            "Request timed out",
            HashMap::new(),
        );
        
        let failures = engine.get_recent_failures(1);
        assert_eq!(failures.len(), 1);
        
        engine.record_recovery(&failures[0].id, 100);
        
        let stats = engine.get_stats();
        assert_eq!(stats.recovered_failures, 1);
    }

    #[test]
    fn test_circuit_integration() {
        let engine = AntifragileEngine::new();
        
        // Trigger multiple failures
        for _ in 0..10 {
            engine.handle_failure(
                "agent-1",
                FailureCategory::ExternalApi,
                FailureSeverity::Major,
                "API unavailable",
                HashMap::new(),
            );
        }
        
        // Circuit should be open
        let should_allow = engine.should_allow("agent-1:ExternalApi");
        assert!(!should_allow);
    }
}
