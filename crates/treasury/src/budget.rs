//! Budget Manager for Spending Limits
//!
//! Per MANDATE.md Section 6: "Budgeting: Strict Gas Limits for tokens, API calls, and cloud costs"

use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::{AgentId, Amount};

/// Budget period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetPeriod {
    /// Per transaction
    Transaction,
    /// Per hour
    Hourly,
    /// Per day
    Daily,
    /// Per week
    Weekly,
    /// Per month
    Monthly,
}

impl BudgetPeriod {
    /// Get duration for this period.
    pub fn duration(&self) -> Option<Duration> {
        match self {
            BudgetPeriod::Transaction => None,
            BudgetPeriod::Hourly => Some(Duration::hours(1)),
            BudgetPeriod::Daily => Some(Duration::days(1)),
            BudgetPeriod::Weekly => Some(Duration::weeks(1)),
            BudgetPeriod::Monthly => Some(Duration::days(30)),
        }
    }
}

/// Spending limit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendingLimit {
    /// Maximum amount
    pub max_amount: Amount,
    /// Period for this limit
    pub period: BudgetPeriod,
    /// Currently spent
    pub spent: Amount,
    /// Period start time
    pub period_start: DateTime<Utc>,
}

impl SpendingLimit {
    /// Create a new spending limit.
    pub fn new(max_amount: Amount, period: BudgetPeriod) -> Self {
        Self {
            max_amount,
            period,
            spent: Amount::new(0, max_amount.decimals),
            period_start: Utc::now(),
        }
    }

    /// Check if period has reset.
    pub fn should_reset(&self) -> bool {
        if let Some(duration) = self.period.duration() {
            Utc::now() > self.period_start + duration
        } else {
            false
        }
    }

    /// Reset the period.
    pub fn reset(&mut self) {
        self.spent = Amount::new(0, self.max_amount.decimals);
        self.period_start = Utc::now();
    }

    /// Get remaining budget.
    pub fn remaining(&self) -> Amount {
        self.max_amount
            .sub(&self.spent)
            .unwrap_or(Amount::new(0, self.max_amount.decimals))
    }

    /// Check if amount can be spent.
    pub fn can_spend(&self, amount: &Amount) -> bool {
        let remaining = self.remaining();
        remaining.value >= amount.value
    }

    /// Record spending.
    pub fn record_spend(&mut self, amount: &Amount) -> Result<(), BudgetError> {
        if !self.can_spend(amount) {
            return Err(BudgetError::LimitExceeded {
                limit: self.max_amount,
                requested: *amount,
                remaining: self.remaining(),
            });
        }

        self.spent = self.spent.add(amount).ok_or(BudgetError::InvalidAmount)?;
        Ok(())
    }
}

/// Budget manager for agents.
pub struct BudgetManager {
    /// Limits by agent ID
    limits: Arc<RwLock<HashMap<AgentId, Vec<SpendingLimit>>>>,
}

impl Default for BudgetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BudgetManager {
    /// Create a new budget manager.
    pub fn new() -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a spending limit for an agent.
    pub fn set_limit(&self, agent_id: &str, limit: SpendingLimit) {
        let mut limits = self.limits.write();
        let agent_limits = limits.entry(agent_id.to_string()).or_default();

        // Remove existing limit for same period
        agent_limits.retain(|l| l.period != limit.period);
        agent_limits.push(limit);
    }

    /// Check if agent can spend amount.
    pub fn can_spend(&self, agent_id: &str, amount: &Amount) -> Result<(), BudgetError> {
        let mut limits = self.limits.write();

        if let Some(agent_limits) = limits.get_mut(agent_id) {
            for limit in agent_limits.iter_mut() {
                // Reset if period expired
                if limit.should_reset() {
                    limit.reset();
                }

                if !limit.can_spend(amount) {
                    return Err(BudgetError::LimitExceeded {
                        limit: limit.max_amount,
                        requested: *amount,
                        remaining: limit.remaining(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Record spending for an agent.
    pub fn record_spend(&self, agent_id: &str, amount: &Amount) -> Result<(), BudgetError> {
        // First check all limits
        self.can_spend(agent_id, amount)?;

        // Then record against all limits
        let mut limits = self.limits.write();
        if let Some(agent_limits) = limits.get_mut(agent_id) {
            for limit in agent_limits.iter_mut() {
                limit.record_spend(amount)?;
            }
        }

        Ok(())
    }

    /// Get remaining budget for an agent (returns minimum across all limits).
    pub fn get_remaining(&self, agent_id: &str) -> Option<Amount> {
        let limits = self.limits.read();
        limits.get(agent_id).and_then(|agent_limits| {
            agent_limits
                .iter()
                .map(|l| l.remaining())
                .min_by_key(|a| a.value)
        })
    }
}

/// Budget errors.
#[derive(Debug, thiserror::Error)]
pub enum BudgetError {
    #[error(
        "Spending limit exceeded: limit={limit}, requested={requested}, remaining={remaining}"
    )]
    LimitExceeded {
        limit: Amount,
        requested: Amount,
        remaining: Amount,
    },
    #[error("Invalid amount")]
    InvalidAmount,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spending_limit() {
        let mut limit = SpendingLimit::new(Amount::from_float(100.0, 2), BudgetPeriod::Daily);

        assert!(limit.can_spend(&Amount::from_float(50.0, 2)));
        limit.record_spend(&Amount::from_float(50.0, 2)).unwrap();

        assert!(limit.can_spend(&Amount::from_float(50.0, 2)));
        assert!(!limit.can_spend(&Amount::from_float(51.0, 2)));
    }

    #[test]
    fn test_budget_manager() {
        let manager = BudgetManager::new();

        manager.set_limit(
            "agent-1",
            SpendingLimit::new(Amount::from_float(100.0, 2), BudgetPeriod::Daily),
        );

        // Should succeed
        manager
            .record_spend("agent-1", &Amount::from_float(30.0, 2))
            .unwrap();
        manager
            .record_spend("agent-1", &Amount::from_float(30.0, 2))
            .unwrap();

        // Should fail
        let result = manager.record_spend("agent-1", &Amount::from_float(50.0, 2));
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_limits() {
        let manager = BudgetManager::new();

        // Per-transaction limit
        manager.set_limit(
            "agent-1",
            SpendingLimit::new(Amount::from_float(50.0, 2), BudgetPeriod::Transaction),
        );

        // Daily limit
        manager.set_limit(
            "agent-1",
            SpendingLimit::new(Amount::from_float(200.0, 2), BudgetPeriod::Daily),
        );

        // Exceeds per-transaction limit
        let result = manager.can_spend("agent-1", &Amount::from_float(60.0, 2));
        assert!(result.is_err());
    }
}
