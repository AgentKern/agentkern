//! Loop Prevention Module
//!
//! Prevents the $47,000 Runaway AI Loop incident by detecting and stopping
//! infinite agent-to-agent message loops.
//!
//! Per MANDATE.md: "Antifragile by default - every failure makes us stronger"
//!
//! # Features
//!
//! - **Message TTL**: Messages expire after N hops
//! - **Loop Detection**: Tracks agent-pair message patterns
//! - **Cost Ceiling**: Enforces budget limits per task
//! - **Circuit Breaker**: Trips after repeated failures
//!
//! # The $47k Problem
//!
//! In March 2024, a multi-agent system ran an infinite loop for 11 days:
//! - Analysis agent requested clarification
//! - Verification agent issued new instructions
//! - Loop repeated endlessly → $47,000 API bill
//!
//! AgentKern prevents this with multiple layers of protection.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Configuration for loop prevention.
#[derive(Debug, Clone)]
pub struct LoopPreventionConfig {
    /// Maximum hops before message is dropped
    pub max_hops: u8,
    /// Maximum messages between same agent pair per minute
    pub max_pair_rate: u32,
    /// Maximum total cost before circuit breaker trips
    pub cost_ceiling: f64,
    /// Window for rate limiting (seconds)
    pub rate_window_secs: u64,
    /// Enabled
    pub enabled: bool,
}

impl Default for LoopPreventionConfig {
    fn default() -> Self {
        Self {
            max_hops: 10,
            max_pair_rate: 100,
            cost_ceiling: 1000.0, // $1000 default ceiling
            rate_window_secs: 60,
            enabled: true,
        }
    }
}

impl LoopPreventionConfig {
    /// Strict configuration for production.
    pub fn strict() -> Self {
        Self {
            max_hops: 5,
            max_pair_rate: 20,
            cost_ceiling: 100.0,
            rate_window_secs: 60,
            enabled: true,
        }
    }

    /// Relaxed for testing.
    pub fn relaxed() -> Self {
        Self {
            max_hops: 50,
            max_pair_rate: 1000,
            cost_ceiling: 10000.0,
            rate_window_secs: 60,
            enabled: true,
        }
    }
}

/// Message with hop tracking.
#[derive(Debug, Clone)]
pub struct TrackedMessage {
    /// Original message ID
    pub message_id: String,
    /// Root correlation ID (tracks entire conversation)
    pub correlation_id: String,
    /// Current hop count
    pub hop_count: u8,
    /// Path of agents this message has traversed
    pub agent_path: Vec<String>,
    /// Estimated cost so far
    pub accumulated_cost: f64,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl TrackedMessage {
    /// Create a new tracked message.
    pub fn new(message_id: &str, source_agent: &str) -> Self {
        Self {
            message_id: message_id.to_string(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            hop_count: 0,
            agent_path: vec![source_agent.to_string()],
            accumulated_cost: 0.0,
            created_at: Utc::now(),
        }
    }

    /// Add a hop with cost.
    pub fn add_hop(&mut self, agent_id: &str, cost: f64) {
        self.hop_count += 1;
        self.agent_path.push(agent_id.to_string());
        self.accumulated_cost += cost;
    }

    /// Check if message is in a loop (visited same agent twice).
    pub fn is_looping(&self) -> bool {
        let mut seen = std::collections::HashSet::new();
        for agent in &self.agent_path {
            if !seen.insert(agent.clone()) {
                return true;
            }
        }
        false
    }

    /// Get the agent pair (last two agents in path).
    pub fn agent_pair(&self) -> Option<(String, String)> {
        if self.agent_path.len() < 2 {
            return None;
        }
        let len = self.agent_path.len();
        Some((
            self.agent_path[len - 2].clone(),
            self.agent_path[len - 1].clone(),
        ))
    }
}

/// Agent pair rate tracker.
#[derive(Debug, Default)]
struct PairRateTracker {
    /// Message count per agent pair
    counts: HashMap<(String, String), AtomicU32>,
    /// Last reset time
    last_reset: Option<Instant>,
}

/// Loop Prevention Engine.
pub struct LoopPreventer {
    config: LoopPreventionConfig,
    /// Total cost accumulated
    total_cost: AtomicU64,
    /// Pair rate tracker
    pair_tracker: parking_lot::RwLock<PairRateTracker>,
    /// Messages dropped due to loop detection
    loops_detected: AtomicU32,
    /// Messages dropped due to hop limit
    hop_limit_hit: AtomicU32,
    /// Messages dropped due to cost ceiling
    cost_ceiling_hit: AtomicU32,
    /// Circuit breaker tripped
    circuit_open: std::sync::atomic::AtomicBool,
}

impl LoopPreventer {
    /// Create a new loop preventer.
    pub fn new(config: LoopPreventionConfig) -> Self {
        Self {
            config,
            total_cost: AtomicU64::new(0),
            pair_tracker: parking_lot::RwLock::new(PairRateTracker::default()),
            loops_detected: AtomicU32::new(0),
            hop_limit_hit: AtomicU32::new(0),
            cost_ceiling_hit: AtomicU32::new(0),
            circuit_open: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Check if a message should be allowed through.
    pub fn check(&self, message: &TrackedMessage) -> Result<(), LoopPreventionError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check circuit breaker
        if self.circuit_open.load(Ordering::Relaxed) {
            return Err(LoopPreventionError::CircuitOpen);
        }

        // Check hop limit
        if message.hop_count >= self.config.max_hops {
            self.hop_limit_hit.fetch_add(1, Ordering::Relaxed);
            return Err(LoopPreventionError::HopLimitExceeded {
                hops: message.hop_count,
                max: self.config.max_hops,
            });
        }

        // Check for loops
        if message.is_looping() {
            self.loops_detected.fetch_add(1, Ordering::Relaxed);
            return Err(LoopPreventionError::LoopDetected {
                path: message.agent_path.clone(),
            });
        }

        // Check cost ceiling
        if message.accumulated_cost >= self.config.cost_ceiling {
            self.cost_ceiling_hit.fetch_add(1, Ordering::Relaxed);
            self.circuit_open.store(true, Ordering::Relaxed);
            return Err(LoopPreventionError::CostCeilingExceeded {
                cost: message.accumulated_cost,
                ceiling: self.config.cost_ceiling,
            });
        }

        // Check pair rate
        if let Some(pair) = message.agent_pair() {
            self.check_pair_rate(&pair)?;
        }

        Ok(())
    }

    /// Check and update pair rate.
    fn check_pair_rate(&self, pair: &(String, String)) -> Result<(), LoopPreventionError> {
        let mut tracker = self.pair_tracker.write();

        // Reset counters if window expired
        if let Some(last_reset) = tracker.last_reset {
            if last_reset.elapsed() > Duration::from_secs(self.config.rate_window_secs) {
                tracker.counts.clear();
                tracker.last_reset = Some(Instant::now());
            }
        } else {
            tracker.last_reset = Some(Instant::now());
        }

        // Get or create counter
        let counter = tracker
            .counts
            .entry(pair.clone())
            .or_insert_with(|| AtomicU32::new(0));

        let current = counter.fetch_add(1, Ordering::Relaxed);

        if current >= self.config.max_pair_rate {
            return Err(LoopPreventionError::PairRateLimitExceeded {
                from: pair.0.clone(),
                to: pair.1.clone(),
                rate: current,
                max: self.config.max_pair_rate,
            });
        }

        Ok(())
    }

    /// Record cost for a message.
    pub fn record_cost(&self, cost: f64) {
        let cost_bits = (cost * 100.0) as u64; // Store as cents
        self.total_cost.fetch_add(cost_bits, Ordering::Relaxed);
    }

    /// Get total cost.
    pub fn total_cost(&self) -> f64 {
        self.total_cost.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// Reset circuit breaker.
    pub fn reset_circuit(&self) {
        self.circuit_open.store(false, Ordering::Relaxed);
    }

    /// Get statistics.
    pub fn stats(&self) -> LoopPreventionStats {
        LoopPreventionStats {
            loops_detected: self.loops_detected.load(Ordering::Relaxed),
            hop_limits_hit: self.hop_limit_hit.load(Ordering::Relaxed),
            cost_ceilings_hit: self.cost_ceiling_hit.load(Ordering::Relaxed),
            total_cost: self.total_cost(),
            circuit_open: self.circuit_open.load(Ordering::Relaxed),
        }
    }
}

impl Default for LoopPreventer {
    fn default() -> Self {
        Self::new(LoopPreventionConfig::default())
    }
}

/// Loop prevention statistics.
#[derive(Debug, Clone, Default)]
pub struct LoopPreventionStats {
    pub loops_detected: u32,
    pub hop_limits_hit: u32,
    pub cost_ceilings_hit: u32,
    pub total_cost: f64,
    pub circuit_open: bool,
}

/// Loop prevention errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LoopPreventionError {
    #[error("Hop limit exceeded: {hops}/{max}")]
    HopLimitExceeded { hops: u8, max: u8 },

    #[error("Loop detected in agent path: {path:?}")]
    LoopDetected { path: Vec<String> },

    #[error("Cost ceiling exceeded: ${cost:.2} > ${ceiling:.2}")]
    CostCeilingExceeded { cost: f64, ceiling: f64 },

    #[error("Pair rate limit exceeded: {from} -> {to} ({rate}/{max} per minute)")]
    PairRateLimitExceeded {
        from: String,
        to: String,
        rate: u32,
        max: u32,
    },

    #[error("Circuit breaker is open")]
    CircuitOpen,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hop_limit() {
        let config = LoopPreventionConfig {
            max_hops: 3,
            ..Default::default()
        };
        let preventer = LoopPreventer::new(config);

        let mut msg = TrackedMessage::new("msg-1", "agent-a");
        msg.add_hop("agent-b", 1.0);
        msg.add_hop("agent-c", 1.0);
        msg.add_hop("agent-d", 1.0);

        let result = preventer.check(&msg);
        assert!(matches!(
            result,
            Err(LoopPreventionError::HopLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_loop_detection() {
        let preventer = LoopPreventer::default();

        let mut msg = TrackedMessage::new("msg-1", "agent-a");
        msg.add_hop("agent-b", 1.0);
        msg.add_hop("agent-a", 1.0); // Back to agent-a = loop!

        let result = preventer.check(&msg);
        assert!(matches!(
            result,
            Err(LoopPreventionError::LoopDetected { .. })
        ));
    }

    #[test]
    fn test_cost_ceiling() {
        let config = LoopPreventionConfig {
            cost_ceiling: 10.0,
            ..Default::default()
        };
        let preventer = LoopPreventer::new(config);

        let mut msg = TrackedMessage::new("msg-1", "agent-a");
        msg.accumulated_cost = 15.0;

        let result = preventer.check(&msg);
        assert!(matches!(
            result,
            Err(LoopPreventionError::CostCeilingExceeded { .. })
        ));

        // Circuit should be open now
        assert!(preventer.stats().circuit_open);
    }

    #[test]
    fn test_valid_message() {
        let preventer = LoopPreventer::default();

        let mut msg = TrackedMessage::new("msg-1", "agent-a");
        msg.add_hop("agent-b", 0.5);

        let result = preventer.check(&msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_the_47k_scenario() {
        // Simulating the runaway loop incident
        let config = LoopPreventionConfig::strict();
        let preventer = LoopPreventer::new(config);

        let mut msg = TrackedMessage::new("msg-1", "analysis-agent");

        // Verification requests clarification → Analysis responds → repeat
        // With AgentKern, this would be caught on the 2nd round-trip
        msg.add_hop("verification-agent", 1.0);
        msg.add_hop("analysis-agent", 1.0); // Loop detected!

        let result = preventer.check(&msg);

        // AgentKern catches the loop immediately
        assert!(matches!(
            result,
            Err(LoopPreventionError::LoopDetected { .. })
        ));

        // The $47k would have been $2 instead
        assert!(msg.accumulated_cost < 5.0);
    }

    #[test]
    fn test_circuit_breaker_blocks_all() {
        let config = LoopPreventionConfig {
            cost_ceiling: 1.0,
            ..Default::default()
        };
        let preventer = LoopPreventer::new(config);

        // Trip the circuit breaker
        let mut msg1 = TrackedMessage::new("msg-1", "agent-a");
        msg1.accumulated_cost = 10.0;
        let _ = preventer.check(&msg1);

        // New message should be blocked
        let msg2 = TrackedMessage::new("msg-2", "agent-b");
        let result = preventer.check(&msg2);

        assert!(matches!(result, Err(LoopPreventionError::CircuitOpen)));

        // Reset and verify it works again
        preventer.reset_circuit();
        let result = preventer.check(&msg2);
        assert!(result.is_ok());
    }
}
