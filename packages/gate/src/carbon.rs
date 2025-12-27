//! AgentKern-Gate: Energy-Aware Veto
//!
//! Per FAANG_GAPS.md: "Energy-Aware Veto Integration"
//! Connects Carbon Ledger to Gate policy engine for kernel-level "stop"
//! before execution based on ESG budgets.

use serde::{Deserialize, Serialize};
use agentkern_treasury::carbon::{CarbonLedger, CarbonRegion, ComputeType, CarbonError};
use crate::types::{AgentId, VerificationResult};

/// Results of a carbon check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonCheckResult {
    /// Is the action allowed under current carbon budget?
    pub allowed: bool,
    /// Estimated CO2 in grams
    pub estimated_co2: f64,
    /// Reason for denial (if any)
    pub message: Option<String>,
}

/// Carbon Policy Controller.
/// 
/// Interacts with the Treasury's Carbon Ledger to enforce
/// environmental guardrails.
pub struct CarbonVeto {
    ledger: CarbonLedger,
    default_region: CarbonRegion,
}

impl CarbonVeto {
    /// Create a new Carbon Veto controller.
    pub fn new(ledger: CarbonLedger) -> Self {
        Self {
            ledger,
            default_region: CarbonRegion::UsAverage,
        }
    }

    /// Set default region.
    pub fn with_default_region(mut self, region: CarbonRegion) -> Self {
        self.default_region = region;
        self
    }

    /// Evaluate an action against carbon budgets.
    pub fn evaluate(
        &self,
        agent_id: &AgentId,
        action: &str,
        compute_estimate: ComputeType,
        duration_ms: u64,
    ) -> CarbonCheckResult {
        let budget = match self.ledger.get_budget(agent_id) {
            Some(b) => b,
            None => {
                // If no budget is set, we allow the action but log it
                // (or we could be strict and deny by default)
                return CarbonCheckResult {
                    allowed: true,
                    estimated_co2: 0.0, // Should be estimation
                    message: None,
                };
            }
        };

        // Estimate footprint
        let footprint = CarbonLedger::estimate(compute_estimate, duration_ms, self.default_region);
        let co2_grams = footprint.co2_grams.to_string().parse::<f64>().unwrap_or(0.0);

        // Check if recording would exceed budget
        // We use record() without actually committing if we just want a check
        // But the ledger's record() method already does the check.
        // For a "veto", we simulate the recording.
        
        let daily = self.ledger.get_daily_usage(agent_id);
        let budget_limit = budget.daily_limit_grams.to_string().parse::<f64>().unwrap_or(0.0);
        let current_usage = daily.total_co2_grams.to_string().parse::<f64>().unwrap_or(0.0);

        if budget.block_on_exceed && (current_usage + co2_grams) > budget_limit {
            CarbonCheckResult {
                allowed: false,
                estimated_co2: co2_grams,
                message: Some(format!(
                    "Carbon budget exceeded. Daily limit: {}g, Current: {}g, Requested: {}g",
                    budget_limit, current_usage, co2_grams
                )),
            }
        } else {
            CarbonCheckResult {
                allowed: true,
                estimated_co2: co2_grams,
                message: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentkern_treasury::carbon::CarbonBudget;
    use rust_decimal_macros::dec;

    #[test]
    fn test_carbon_veto_logic() {
        let ledger = CarbonLedger::new();
        let agent_id = "agent-1".to_string();
        
        // Set a strict budget
        ledger.set_budget(
            CarbonBudget::new(agent_id.clone())
                .with_daily_limit(dec!(0.1)) // 0.1g limit
                .block_on_exceed()
        );

        let veto = CarbonVeto::new(ledger);
        
        // Inference is heavy
        let result = veto.evaluate(&agent_id, "inference", ComputeType::Gpu, 60_000);
        
        assert!(!result.allowed);
        assert!(result.message.unwrap().contains("budget exceeded"));
    }

    #[test]
    fn test_allow_when_under_budget() {
        let ledger = CarbonLedger::new();
        let agent_id = "agent-2".to_string();
        
        ledger.set_budget(
            CarbonBudget::new(agent_id.clone())
                .with_daily_limit(dec!(1000.0))
                .block_on_exceed()
        );

        let veto = CarbonVeto::new(ledger);
        let result = veto.evaluate(&agent_id, "small_op", ComputeType::Cpu, 100);
        
        assert!(result.allowed);
    }
}
