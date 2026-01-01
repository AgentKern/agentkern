//! AgentKern-Gate: Agent Budget and Gas Limits
//!
//! Per EXECUTION_MANDATE.md §6: "Strict Gas Limits for tokens, API calls, and cloud costs"
//!
//! Features:
//! - Token usage limits
//! - API call rate limits
//! - Cost/spend limits
//! - Time limits
//! - Automatic enforcement
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::budget::{AgentBudget, BudgetConfig};
//!
//! let mut budget = AgentBudget::new("agent-123", BudgetConfig::default());
//! budget.consume_tokens(1000)?;
//! budget.consume_api_call()?;
//! budget.consume_cost(0.05)?;
//! ```

use serde::{Deserialize, Serialize};
use std::time::Instant;
use thiserror::Error;

/// Budget exceeded error.
#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("Token limit exceeded: used {used}, limit {limit}")]
    TokenLimitExceeded { used: u64, limit: u64 },
    #[error("API call limit exceeded: used {used}, limit {limit}")]
    ApiCallLimitExceeded { used: u64, limit: u64 },
    #[error("Cost limit exceeded: spent ${spent:.4}, limit ${limit:.4}")]
    CostLimitExceeded { spent: f64, limit: f64 },
    #[error("Time limit exceeded: ran for {elapsed_secs}s, limit {limit_secs}s")]
    TimeLimitExceeded { elapsed_secs: u64, limit_secs: u64 },
    #[error("Budget exhausted")]
    BudgetExhausted,
}

/// Budget configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Maximum tokens allowed
    pub max_tokens: u64,
    /// Maximum API calls allowed
    pub max_api_calls: u64,
    /// Maximum cost in USD
    pub max_cost_usd: f64,
    /// Maximum runtime in seconds
    pub max_runtime_secs: u64,
    /// Whether to enforce limits (vs just track)
    pub enforce: bool,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,    // 100k tokens
            max_api_calls: 1_000,   // 1000 API calls
            max_cost_usd: 10.0,     // $10 USD
            max_runtime_secs: 3600, // 1 hour
            enforce: true,
        }
    }
}

impl BudgetConfig {
    /// Create a minimal budget for testing.
    pub fn minimal() -> Self {
        Self {
            max_tokens: 1_000,
            max_api_calls: 10,
            max_cost_usd: 0.10,
            max_runtime_secs: 60,
            enforce: true,
        }
    }

    /// Create an unlimited budget (use with caution!).
    pub fn unlimited() -> Self {
        Self {
            max_tokens: u64::MAX,
            max_api_calls: u64::MAX,
            max_cost_usd: f64::MAX,
            max_runtime_secs: u64::MAX,
            enforce: false,
        }
    }

    /// Create a budget for enterprise tier.
    pub fn enterprise() -> Self {
        Self {
            max_tokens: 10_000_000,  // 10M tokens
            max_api_calls: 100_000,  // 100k API calls
            max_cost_usd: 1000.0,    // $1000 USD
            max_runtime_secs: 86400, // 24 hours
            enforce: true,
        }
    }
}

/// Current budget usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUsage {
    /// Tokens consumed
    pub tokens_used: u64,
    /// API calls made
    pub api_calls_used: u64,
    /// Cost incurred in USD
    pub cost_usd: f64,
    /// Runtime in seconds
    pub runtime_secs: u64,
}

impl Default for BudgetUsage {
    fn default() -> Self {
        Self {
            tokens_used: 0,
            api_calls_used: 0,
            cost_usd: 0.0,
            runtime_secs: 0,
        }
    }
}

/// Agent budget tracker.
#[derive(Debug)]
pub struct AgentBudget {
    /// Agent ID
    pub agent_id: String,
    /// Configuration
    pub config: BudgetConfig,
    /// Current usage
    pub usage: BudgetUsage,
    /// Start time
    start_time: Instant,
    /// Is exhausted flag
    exhausted: bool,
}

impl AgentBudget {
    /// Create a new agent budget.
    pub fn new(agent_id: impl Into<String>, config: BudgetConfig) -> Self {
        Self {
            agent_id: agent_id.into(),
            config,
            usage: BudgetUsage::default(),
            start_time: Instant::now(),
            exhausted: false,
        }
    }

    /// Check if budget is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.exhausted
    }

    /// Get remaining tokens.
    pub fn remaining_tokens(&self) -> u64 {
        self.config
            .max_tokens
            .saturating_sub(self.usage.tokens_used)
    }

    /// Get remaining API calls.
    pub fn remaining_api_calls(&self) -> u64 {
        self.config
            .max_api_calls
            .saturating_sub(self.usage.api_calls_used)
    }

    /// Get remaining budget in USD.
    pub fn remaining_cost(&self) -> f64 {
        (self.config.max_cost_usd - self.usage.cost_usd).max(0.0)
    }

    /// Get remaining runtime in seconds.
    pub fn remaining_runtime_secs(&self) -> u64 {
        let elapsed = self.start_time.elapsed().as_secs();
        self.config.max_runtime_secs.saturating_sub(elapsed)
    }

    /// Consume tokens.
    pub fn consume_tokens(&mut self, tokens: u64) -> Result<(), BudgetError> {
        if self.exhausted {
            return Err(BudgetError::BudgetExhausted);
        }

        let new_total = self.usage.tokens_used + tokens;

        if self.config.enforce && new_total > self.config.max_tokens {
            self.exhausted = true;
            return Err(BudgetError::TokenLimitExceeded {
                used: new_total,
                limit: self.config.max_tokens,
            });
        }

        self.usage.tokens_used = new_total;
        Ok(())
    }

    /// Consume an API call.
    pub fn consume_api_call(&mut self) -> Result<(), BudgetError> {
        self.consume_api_calls(1)
    }

    /// Consume multiple API calls.
    pub fn consume_api_calls(&mut self, calls: u64) -> Result<(), BudgetError> {
        if self.exhausted {
            return Err(BudgetError::BudgetExhausted);
        }

        let new_total = self.usage.api_calls_used + calls;

        if self.config.enforce && new_total > self.config.max_api_calls {
            self.exhausted = true;
            return Err(BudgetError::ApiCallLimitExceeded {
                used: new_total,
                limit: self.config.max_api_calls,
            });
        }

        self.usage.api_calls_used = new_total;
        Ok(())
    }

    /// Consume cost.
    pub fn consume_cost(&mut self, cost_usd: f64) -> Result<(), BudgetError> {
        if self.exhausted {
            return Err(BudgetError::BudgetExhausted);
        }

        let new_total = self.usage.cost_usd + cost_usd;

        if self.config.enforce && new_total > self.config.max_cost_usd {
            self.exhausted = true;
            return Err(BudgetError::CostLimitExceeded {
                spent: new_total,
                limit: self.config.max_cost_usd,
            });
        }

        self.usage.cost_usd = new_total;
        Ok(())
    }

    /// Check time limit (call periodically).
    pub fn check_time_limit(&mut self) -> Result<(), BudgetError> {
        if self.exhausted {
            return Err(BudgetError::BudgetExhausted);
        }

        let elapsed = self.start_time.elapsed().as_secs();
        self.usage.runtime_secs = elapsed;

        if self.config.enforce && elapsed > self.config.max_runtime_secs {
            self.exhausted = true;
            return Err(BudgetError::TimeLimitExceeded {
                elapsed_secs: elapsed,
                limit_secs: self.config.max_runtime_secs,
            });
        }

        Ok(())
    }

    /// Get usage percentage (0.0 - 1.0, max across all limits).
    pub fn usage_percentage(&self) -> f64 {
        let token_pct = self.usage.tokens_used as f64 / self.config.max_tokens as f64;
        let api_pct = self.usage.api_calls_used as f64 / self.config.max_api_calls as f64;
        let cost_pct = self.usage.cost_usd / self.config.max_cost_usd;
        let time_pct =
            self.start_time.elapsed().as_secs() as f64 / self.config.max_runtime_secs as f64;

        token_pct.max(api_pct).max(cost_pct).max(time_pct)
    }

    /// Get a summary of the budget status.
    pub fn summary(&self) -> BudgetSummary {
        BudgetSummary {
            agent_id: self.agent_id.clone(),
            exhausted: self.exhausted,
            usage_percentage: self.usage_percentage(),
            tokens_remaining: self.remaining_tokens(),
            api_calls_remaining: self.remaining_api_calls(),
            cost_remaining_usd: self.remaining_cost(),
            runtime_remaining_secs: self.remaining_runtime_secs(),
        }
    }
}

/// Budget summary for reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummary {
    pub agent_id: String,
    pub exhausted: bool,
    pub usage_percentage: f64,
    pub tokens_remaining: u64,
    pub api_calls_remaining: u64,
    pub cost_remaining_usd: f64,
    pub runtime_remaining_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_creation() {
        let budget = AgentBudget::new("agent-1", BudgetConfig::default());
        assert_eq!(budget.remaining_tokens(), 100_000);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn test_token_consumption() {
        let mut budget = AgentBudget::new("agent-1", BudgetConfig::minimal());

        budget.consume_tokens(500).unwrap();
        assert_eq!(budget.remaining_tokens(), 500);

        budget.consume_tokens(400).unwrap();
        assert_eq!(budget.remaining_tokens(), 100);

        // This should fail
        let result = budget.consume_tokens(200);
        assert!(result.is_err());
        assert!(budget.is_exhausted());
    }

    #[test]
    fn test_api_call_limit() {
        let mut budget = AgentBudget::new("agent-1", BudgetConfig::minimal());

        for _ in 0..10 {
            budget.consume_api_call().unwrap();
        }

        // 11th call should fail
        let result = budget.consume_api_call();
        assert!(result.is_err());
    }

    #[test]
    fn test_cost_limit() {
        let mut budget = AgentBudget::new("agent-1", BudgetConfig::minimal());

        budget.consume_cost(0.05).unwrap();
        budget.consume_cost(0.04).unwrap();

        // This should fail ($0.10 limit)
        let result = budget.consume_cost(0.02);
        assert!(result.is_err());
    }

    #[test]
    fn test_usage_percentage() {
        let mut budget = AgentBudget::new(
            "agent-1",
            BudgetConfig {
                max_tokens: 100,
                max_api_calls: 10,
                max_cost_usd: 1.0,
                max_runtime_secs: 100,
                enforce: true,
            },
        );

        budget.consume_tokens(50).unwrap();
        assert!((budget.usage_percentage() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_exhausted_blocks_all() {
        let mut budget = AgentBudget::new("agent-1", BudgetConfig::minimal());

        // Exhaust tokens
        let _ = budget.consume_tokens(2000);
        assert!(budget.is_exhausted());

        // Now everything should fail
        assert!(budget.consume_api_call().is_err());
        assert!(budget.consume_cost(0.01).is_err());
    }
}
