//! AgentKern-Gate: Energy-Aware Veto
//!
//! Per FAANG_GAPS.md: "Energy-Aware Veto Integration"
//! Connects Carbon Ledger to Gate policy engine for kernel-level "stop"
//! before execution based on ESG budgets.

use crate::types::AgentId;
use agentkern_treasury::carbon::{CarbonLedger, CarbonRegion, ComputeType};
use serde::{Deserialize, Serialize};

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
/// environmental guardrails with optional real-time WattTime data.
pub struct CarbonVeto {
    ledger: CarbonLedger,
    default_region: CarbonRegion,
    /// Optional WattTime client for dynamic grid intensity
    watttime: Option<agentkern_treasury::watttime::WattTimeClient>,
    /// Location for WattTime lookups
    location: Option<(f64, f64)>,
}

impl CarbonVeto {
    /// Create a new Carbon Veto controller.
    pub fn new(ledger: CarbonLedger) -> Self {
        Self {
            ledger,
            default_region: CarbonRegion::UsAverage,
            watttime: None,
            location: None,
        }
    }

    /// Set default region.
    pub fn with_default_region(mut self, region: CarbonRegion) -> Self {
        self.default_region = region;
        self
    }

    /// Enable dynamic carbon intensity via WattTime API.
    pub fn with_watttime(mut self, client: agentkern_treasury::watttime::WattTimeClient, lat: f64, lon: f64) -> Self {
        self.watttime = Some(client);
        self.location = Some((lat, lon));
        self
    }

    /// Evaluate an action against carbon budgets (sync, uses static intensity).
    pub fn evaluate(
        &self,
        agent_id: &AgentId,
        _action: &str,
        compute_estimate: ComputeType,
        duration_ms: u64,
    ) -> CarbonCheckResult {
        let budget = match self.ledger.get_budget(agent_id) {
            Some(b) => b,
            None => {
                return CarbonCheckResult {
                    allowed: true,
                    estimated_co2: 0.0,
                    message: None,
                };
            }
        };

        // Estimate footprint using default region
        let footprint = CarbonLedger::estimate(compute_estimate, duration_ms, self.default_region);
        let co2_grams = footprint
            .co2_grams
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);

        self.check_budget(agent_id, &budget, co2_grams)
    }

    /// Evaluate with real-time WattTime intensity (async).
    pub async fn evaluate_dynamic(
        &self,
        agent_id: &AgentId,
        _action: &str,
        compute_estimate: ComputeType,
        duration_ms: u64,
    ) -> CarbonCheckResult {
        let budget = match self.ledger.get_budget(agent_id) {
            Some(b) => b,
            None => {
                return CarbonCheckResult {
                    allowed: true,
                    estimated_co2: 0.0,
                    message: None,
                };
            }
        };

        // Get dynamic intensity from WattTime if available
        let intensity = if let (Some(client), Some((lat, lon))) = (&self.watttime, self.location) {
            match client.get_intensity(lat, lon).await {
                Ok(i) => i,
                Err(_) => 400, // Fallback to US average
            }
        } else {
            400 // Static fallback
        };

        // Use Dynamic region with real-time intensity
        let region = CarbonRegion::Dynamic(intensity);
        let footprint = CarbonLedger::estimate(compute_estimate, duration_ms, region);
        let co2_grams = footprint
            .co2_grams
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);

        self.check_budget(agent_id, &budget, co2_grams)
    }

    /// Check if action would exceed budget.
    fn check_budget(
        &self,
        agent_id: &AgentId,
        budget: &agentkern_treasury::carbon::CarbonBudget,
        co2_grams: f64,
    ) -> CarbonCheckResult {
        let daily = self.ledger.get_daily_usage(agent_id);
        let budget_limit = budget
            .daily_limit_grams
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);
        let current_usage = daily
            .total_co2_grams
            .to_string()
            .parse::<f64>()
            .unwrap_or(0.0);

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
                .block_on_exceed(),
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
                .block_on_exceed(),
        );

        let veto = CarbonVeto::new(ledger);
        let result = veto.evaluate(&agent_id, "small_op", ComputeType::Cpu, 100);

        assert!(result.allowed);
    }
}
