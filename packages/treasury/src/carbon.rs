//! AgentKern-Treasury: Carbon Footprint Ledger
//!
//! Per FUTURE_INNOVATION_ROADMAP.md Innovation #8:
//! - Per-action carbon tracking (CO2, energy, water)
//! - Regional carbon intensity awareness
//! - Budget enforcement
//! - Carbon-aware scheduling
//! - Offset integration
//!
//! This addresses ESG requirements and positions AgentKern
//! as the only agent platform with native sustainability tracking.

use chrono::{DateTime, Duration, Timelike, Utc};
use parking_lot::RwLock;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::AgentId;

// ============================================================================
// CORE TYPES
// ============================================================================

/// Carbon intensity by region (gCO2/kWh).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CarbonRegion {
    /// US Average (~400 gCO2/kWh)
    #[default]
    UsAverage,
    /// US West (cleaner, ~250 gCO2/kWh)
    UsWest,
    /// US East (~350 gCO2/kWh)
    UsEast,
    /// EU Average (~300 gCO2/kWh)
    EuAverage,
    /// Nordic (very clean, ~50 gCO2/kWh)
    Nordic,
    /// Germany (~400 gCO2/kWh)
    Germany,
    /// France (nuclear, ~60 gCO2/kWh)
    France,
    /// UK (~200 gCO2/kWh)  
    Uk,
    /// China (~550 gCO2/kWh)
    China,
    /// India (~700 gCO2/kWh)
    India,
    /// Custom region with specified intensity
    Custom(u32),
    /// Dynamic intensity from WattTime API (real-time)
    Dynamic(u32),
}

impl CarbonRegion {
    /// Get carbon intensity in gCO2/kWh.
    pub fn intensity(&self) -> u32 {
        match self {
            CarbonRegion::UsAverage => 400,
            CarbonRegion::UsWest => 250,
            CarbonRegion::UsEast => 350,
            CarbonRegion::EuAverage => 300,
            CarbonRegion::Nordic => 50,
            CarbonRegion::Germany => 400,
            CarbonRegion::France => 60,
            CarbonRegion::Uk => 200,
            CarbonRegion::China => 550,
            CarbonRegion::India => 700,
            CarbonRegion::Custom(intensity) => *intensity,
            CarbonRegion::Dynamic(intensity) => *intensity,
        }
    }

    /// Get region name.
    pub fn name(&self) -> &'static str {
        match self {
            CarbonRegion::UsAverage => "US Average",
            CarbonRegion::UsWest => "US West",
            CarbonRegion::UsEast => "US East",
            CarbonRegion::EuAverage => "EU Average",
            CarbonRegion::Nordic => "Nordic",
            CarbonRegion::Germany => "Germany",
            CarbonRegion::France => "France",
            CarbonRegion::Uk => "UK",
            CarbonRegion::China => "China",
            CarbonRegion::India => "India",
            CarbonRegion::Custom(_) => "Custom",
            CarbonRegion::Dynamic(_) => "Dynamic (WattTime)",
        }
    }
}

/// Compute resource type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComputeType {
    /// CPU inference
    Cpu,
    /// GPU inference (generic, uses H100 default)
    Gpu,
    /// GPU with specific model
    GpuModel(GpuModel),
    /// TPU inference
    Tpu,
    /// Network transfer
    Network,
    /// Storage operations
    Storage,
}

/// Specific GPU model for accurate power profiling (Dec 2025).
///
/// Power consumption varies significantly between GPU generations.
/// Source: NVIDIA Datacenter GPU specifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpuModel {
    /// NVIDIA H100 SXM5 - 700W TDP (Hopper architecture)
    H100,
    /// NVIDIA A100 SXM4 - 400W TDP (Ampere architecture)
    A100,
    /// NVIDIA T4 - 70W TDP (Turing, inference-optimized)
    T4,
    /// NVIDIA V100 - 300W TDP (Volta, legacy)
    V100,
    /// NVIDIA L4 - 72W TDP (Ada Lovelace, inference)
    L4,
    /// AMD MI300X - 750W TDP
    Mi300x,
    /// Google TPU v5e - 200W (cloud-only)
    TpuV5e,
}

impl GpuModel {
    /// Get TDP (Thermal Design Power) in watts.
    pub fn tdp_watts(&self) -> u32 {
        match self {
            GpuModel::H100 => 700,
            GpuModel::A100 => 400,
            GpuModel::T4 => 70,
            GpuModel::V100 => 300,
            GpuModel::L4 => 72,
            GpuModel::Mi300x => 750,
            GpuModel::TpuV5e => 200,
        }
    }

    /// Get typical inference utilization percentage.
    pub fn inference_utilization(&self) -> u8 {
        match self {
            GpuModel::H100 => 80,
            GpuModel::A100 => 75,
            GpuModel::T4 => 90,   // Inference-optimized
            GpuModel::V100 => 70,
            GpuModel::L4 => 85,   // Inference-optimized
            GpuModel::Mi300x => 75,
            GpuModel::TpuV5e => 85,
        }
    }

    /// Get relative carbon efficiency (lower is better, T4 = 1.0 baseline).
    pub fn carbon_efficiency_ratio(&self) -> f64 {
        // CO2 per TFLOP relative to T4
        match self {
            GpuModel::T4 => 1.0,      // Baseline
            GpuModel::L4 => 0.9,      // Slightly better
            GpuModel::A100 => 1.5,    // Higher power but more throughput
            GpuModel::H100 => 1.8,    // Highest power
            GpuModel::V100 => 2.0,    // Legacy, less efficient
            GpuModel::Mi300x => 2.1,  // Very high power
            GpuModel::TpuV5e => 0.8,  // Cloud-optimized
        }
    }
}

impl ComputeType {
    /// Typical power draw in watts.
    pub fn typical_watts(&self) -> u32 {
        match self {
            ComputeType::Cpu => 150,              // Server CPU
            ComputeType::Gpu => 400,              // Default to A100
            ComputeType::GpuModel(model) => model.tdp_watts(),
            ComputeType::Tpu => 300,              // TPU v4
            ComputeType::Network => 10,           // Per operation
            ComputeType::Storage => 5,            // Per operation
        }
    }

    /// Water usage ratio (L/kWh for cooling).
    pub fn water_ratio(&self) -> Decimal {
        match self {
            ComputeType::Cpu => dec!(1.8),
            ComputeType::Gpu | ComputeType::GpuModel(_) => dec!(2.5), // More cooling needed
            ComputeType::Tpu => dec!(2.2),
            ComputeType::Network => dec!(0.5),
            ComputeType::Storage => dec!(0.3),
        }
    }
}

// ============================================================================
// SOLAR CURVE SCHEDULING (Dec 2025 GreenOps Innovation)
// ============================================================================

/// Solar curve for time-of-day renewable peak scheduling.
///
/// Carbon-aware scheduling that shifts compute to periods of peak
/// renewable generation, following the "solar curve" pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolarCurve {
    /// Hour of day when renewable generation peaks (0-23 UTC)
    pub peak_hour_utc: u8,
    /// Duration of peak window in hours
    pub peak_duration_hours: u8,
    /// Percentage reduction in carbon intensity during peak
    pub peak_reduction_pct: u8,
}

impl Default for SolarCurve {
    fn default() -> Self {
        // Default: Solar peak around 12:00-15:00 UTC (midday)
        Self {
            peak_hour_utc: 12,
            peak_duration_hours: 4,
            peak_reduction_pct: 30,
        }
    }
}

impl SolarCurve {
    /// Create a solar curve for a specific region.
    pub fn for_region(region: CarbonRegion) -> Self {
        match region {
            CarbonRegion::Nordic => Self {
                peak_hour_utc: 11,        // Solar + wind peaks earlier
                peak_duration_hours: 6,   // Long renewable window
                peak_reduction_pct: 50,   // Very clean grid
            },
            CarbonRegion::France => Self {
                peak_hour_utc: 12,
                peak_duration_hours: 4,
                peak_reduction_pct: 25,   // Nuclear base, solar supplement
            },
            CarbonRegion::UsWest => Self {
                peak_hour_utc: 19,        // 12:00 PT = 19:00 UTC
                peak_duration_hours: 5,   // California solar peak
                peak_reduction_pct: 40,
            },
            CarbonRegion::Germany => Self {
                peak_hour_utc: 12,
                peak_duration_hours: 4,
                peak_reduction_pct: 35,   // High solar capacity
            },
            _ => Self::default(),
        }
    }

    /// Check if current time is within the renewable peak window.
    pub fn is_peak_now(&self) -> bool {
        let now = Utc::now();
        let current_hour = now.hour() as u8;
        self.is_peak_hour(current_hour)
    }

    /// Check if a specific hour is within the peak window.
    pub fn is_peak_hour(&self, hour_utc: u8) -> bool {
        let end_hour = (self.peak_hour_utc + self.peak_duration_hours) % 24;
        
        if self.peak_hour_utc < end_hour {
            hour_utc >= self.peak_hour_utc && hour_utc < end_hour
        } else {
            // Window wraps around midnight
            hour_utc >= self.peak_hour_utc || hour_utc < end_hour
        }
    }

    /// Get effective carbon intensity reduction (0.0 - 1.0).
    pub fn intensity_multiplier(&self) -> f64 {
        if self.is_peak_now() {
            1.0 - (self.peak_reduction_pct as f64 / 100.0)
        } else {
            1.0
        }
    }

    /// Recommend delay in hours until next peak window.
    pub fn hours_until_peak(&self) -> u8 {
        let now = Utc::now();
        let current_hour = now.hour() as u8;
        
        if self.is_peak_hour(current_hour) {
            return 0;
        }
        
        if current_hour < self.peak_hour_utc {
            self.peak_hour_utc - current_hour
        } else {
            (24 - current_hour) + self.peak_hour_utc
        }
    }
}

/// Carbon footprint for a single action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonFootprint {
    /// Unique ID
    pub id: String,
    /// Agent that performed the action
    pub agent_id: AgentId,
    /// Action name
    pub action: String,
    /// CO2 emissions in grams
    pub co2_grams: Decimal,
    /// Energy consumed in kWh
    pub energy_kwh: Decimal,
    /// Water used in liters
    pub water_liters: Decimal,
    /// Compute duration in milliseconds
    pub duration_ms: u64,
    /// Compute type used
    pub compute_type: ComputeType,
    /// Region where compute occurred
    pub region: CarbonRegion,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Was this offset?
    pub offset: bool,
}

impl CarbonFootprint {
    /// Calculate footprint from compute parameters.
    pub fn calculate(
        agent_id: AgentId,
        action: &str,
        compute_type: ComputeType,
        duration_ms: u64,
        region: CarbonRegion,
    ) -> Self {
        // Energy = Power * Time
        let hours = Decimal::from(duration_ms) / dec!(3_600_000);
        let watts = Decimal::from(compute_type.typical_watts());
        let energy_kwh = watts * hours / dec!(1000);

        // CO2 = Energy * Carbon Intensity
        let intensity = Decimal::from(region.intensity());
        let co2_grams = energy_kwh * intensity;

        // Water = Energy * Water Ratio
        let water_liters = energy_kwh * compute_type.water_ratio();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            action: action.to_string(),
            co2_grams,
            energy_kwh,
            water_liters,
            duration_ms,
            compute_type,
            region,
            timestamp: Utc::now(),
            offset: false,
        }
    }

    /// Mark as offset.
    pub fn mark_offset(&mut self) {
        self.offset = true;
    }
}

/// Carbon budget for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonBudget {
    /// Agent ID
    pub agent_id: AgentId,
    /// Maximum CO2 per day in grams
    pub daily_limit_grams: Decimal,
    /// Maximum CO2 per month in grams
    pub monthly_limit_grams: Decimal,
    /// Alert threshold (percentage)
    pub alert_threshold_pct: u8,
    /// Block on exceed
    pub block_on_exceed: bool,
}

impl CarbonBudget {
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            daily_limit_grams: dec!(1000),    // 1kg CO2/day default
            monthly_limit_grams: dec!(25000), // 25kg CO2/month default
            alert_threshold_pct: 80,
            block_on_exceed: false,
        }
    }

    pub fn with_daily_limit(mut self, grams: Decimal) -> Self {
        self.daily_limit_grams = grams;
        self
    }

    pub fn with_monthly_limit(mut self, grams: Decimal) -> Self {
        self.monthly_limit_grams = grams;
        self
    }

    pub fn block_on_exceed(mut self) -> Self {
        self.block_on_exceed = true;
        self
    }
}

/// Carbon usage summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonUsage {
    /// Total CO2 in grams
    pub total_co2_grams: Decimal,
    /// Total energy in kWh
    pub total_energy_kwh: Decimal,
    /// Total water in liters
    pub total_water_liters: Decimal,
    /// Number of actions
    pub action_count: u64,
    /// Period start
    pub period_start: DateTime<Utc>,
    /// Period end
    pub period_end: DateTime<Utc>,
}

// ============================================================================
// CARBON LEDGER
// ============================================================================

/// The Carbon Footprint Ledger.
pub struct CarbonLedger {
    /// Recorded footprints
    footprints: Arc<RwLock<Vec<CarbonFootprint>>>,
    /// Agent budgets
    budgets: Arc<RwLock<HashMap<AgentId, CarbonBudget>>>,
    /// Default region
    default_region: CarbonRegion,
    /// Maximum history size
    max_history: usize,
}

impl Default for CarbonLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl CarbonLedger {
    /// Create a new carbon ledger.
    pub fn new() -> Self {
        Self {
            footprints: Arc::new(RwLock::new(Vec::new())),
            budgets: Arc::new(RwLock::new(HashMap::new())),
            default_region: CarbonRegion::UsAverage,
            max_history: 100_000,
        }
    }

    /// Set default region.
    pub fn with_default_region(mut self, region: CarbonRegion) -> Self {
        self.default_region = region;
        self
    }

    /// Record a carbon footprint.
    pub fn record(&self, footprint: CarbonFootprint) -> Result<(), CarbonError> {
        // Check budget
        if let Some(budget) = self.get_budget(&footprint.agent_id) {
            let daily = self.get_daily_usage(&footprint.agent_id);
            let new_total = daily.total_co2_grams + footprint.co2_grams;

            if new_total > budget.daily_limit_grams && budget.block_on_exceed {
                return Err(CarbonError::BudgetExceeded {
                    agent_id: footprint.agent_id.clone(),
                    limit: budget.daily_limit_grams,
                    current: daily.total_co2_grams,
                    requested: footprint.co2_grams,
                });
            }
        }

        let mut footprints = self.footprints.write();
        footprints.push(footprint);

        // Trim if needed
        if footprints.len() > self.max_history {
            footprints.remove(0);
        }

        Ok(())
    }

    /// Record compute and calculate footprint automatically.
    pub fn record_compute(
        &self,
        agent_id: AgentId,
        action: &str,
        compute_type: ComputeType,
        duration_ms: u64,
        region: Option<CarbonRegion>,
    ) -> Result<CarbonFootprint, CarbonError> {
        let footprint = CarbonFootprint::calculate(
            agent_id,
            action,
            compute_type,
            duration_ms,
            region.unwrap_or(self.default_region),
        );

        self.record(footprint.clone())?;
        Ok(footprint)
    }

    /// Set budget for an agent.
    pub fn set_budget(&self, budget: CarbonBudget) {
        let mut budgets = self.budgets.write();
        budgets.insert(budget.agent_id.clone(), budget);
    }

    /// Get budget for an agent.
    pub fn get_budget(&self, agent_id: &AgentId) -> Option<CarbonBudget> {
        let budgets = self.budgets.read();
        budgets.get(agent_id).cloned()
    }

    /// Get daily usage for an agent.
    pub fn get_daily_usage(&self, agent_id: &AgentId) -> CarbonUsage {
        let now = Utc::now();
        let day_start = now - Duration::hours(24);
        self.get_usage_for_period(agent_id, day_start, now)
    }

    /// Get monthly usage for an agent.
    pub fn get_monthly_usage(&self, agent_id: &AgentId) -> CarbonUsage {
        let now = Utc::now();
        let month_start = now - Duration::days(30);
        self.get_usage_for_period(agent_id, month_start, now)
    }

    /// Get usage for a specific period.
    pub fn get_usage_for_period(
        &self,
        agent_id: &AgentId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> CarbonUsage {
        let footprints = self.footprints.read();

        let relevant: Vec<_> = footprints
            .iter()
            .filter(|f| &f.agent_id == agent_id && f.timestamp >= start && f.timestamp <= end)
            .collect();

        let total_co2_grams = relevant.iter().map(|f| f.co2_grams).sum();
        let total_energy_kwh = relevant.iter().map(|f| f.energy_kwh).sum();
        let total_water_liters = relevant.iter().map(|f| f.water_liters).sum();

        CarbonUsage {
            total_co2_grams,
            total_energy_kwh,
            total_water_liters,
            action_count: relevant.len() as u64,
            period_start: start,
            period_end: end,
        }
    }

    /// Get total fleet carbon footprint.
    pub fn get_fleet_usage(&self) -> CarbonUsage {
        let footprints = self.footprints.read();

        let total_co2_grams = footprints.iter().map(|f| f.co2_grams).sum();
        let total_energy_kwh = footprints.iter().map(|f| f.energy_kwh).sum();
        let total_water_liters = footprints.iter().map(|f| f.water_liters).sum();

        let (start, end) = if footprints.is_empty() {
            (Utc::now(), Utc::now())
        } else {
            (
                footprints.first().unwrap().timestamp,
                footprints.last().unwrap().timestamp,
            )
        };

        CarbonUsage {
            total_co2_grams,
            total_energy_kwh,
            total_water_liters,
            action_count: footprints.len() as u64,
            period_start: start,
            period_end: end,
        }
    }

    /// Find the cleanest region for scheduling.
    pub fn recommend_region(&self) -> CarbonRegion {
        // In a real implementation, this would query real-time grid data
        // For now, return the cleanest static option
        CarbonRegion::Nordic
    }

    /// Check if action should be delayed for cleaner energy.
    pub fn should_delay_for_green(&self, region: CarbonRegion) -> bool {
        // Delay if intensity is above threshold
        region.intensity() > 300
    }

    /// Estimate carbon for a hypothetical action.
    pub fn estimate(
        compute_type: ComputeType,
        duration_ms: u64,
        region: CarbonRegion,
    ) -> CarbonFootprint {
        CarbonFootprint::calculate(
            "estimate".to_string(),
            "estimate",
            compute_type,
            duration_ms,
            region,
        )
    }

    /// Get recent footprints.
    pub fn get_history(&self, limit: usize) -> Vec<CarbonFootprint> {
        let footprints = self.footprints.read();
        footprints.iter().rev().take(limit).cloned().collect()
    }

    /// Export metrics in OpenTelemetry-compatible format.
    pub fn export_metrics(&self) -> CarbonMetrics {
        let fleet = self.get_fleet_usage();
        let solar = SolarCurve::default();
        
        CarbonMetrics {
            total_co2_grams: fleet.total_co2_grams.to_string().parse().unwrap_or(0.0),
            total_energy_kwh: fleet.total_energy_kwh.to_string().parse().unwrap_or(0.0),
            total_water_liters: fleet.total_water_liters.to_string().parse().unwrap_or(0.0),
            action_count: fleet.action_count,
            is_renewable_peak: solar.is_peak_now(),
            recommended_region: self.recommend_region().name().to_string(),
        }
    }
}

// ============================================================================
// OPENTELEMETRY METRICS EXPORT
// ============================================================================

/// Carbon metrics for OpenTelemetry export.
///
/// Compatible with OTLP (OpenTelemetry Protocol) for Prometheus/Grafana.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonMetrics {
    /// Total CO2 emissions in grams
    pub total_co2_grams: f64,
    /// Total energy consumed in kWh
    pub total_energy_kwh: f64,
    /// Total water used in liters
    pub total_water_liters: f64,
    /// Number of recorded actions
    pub action_count: u64,
    /// Is current time within renewable peak window
    pub is_renewable_peak: bool,
    /// Recommended region for compute
    pub recommended_region: String,
}

impl CarbonMetrics {
    /// Export as OpenTelemetry key-value attributes.
    pub fn export_otlp_attributes(&self) -> Vec<(&'static str, String)> {
        vec![
            ("agentkern.carbon.co2_grams", self.total_co2_grams.to_string()),
            ("agentkern.carbon.energy_kwh", self.total_energy_kwh.to_string()),
            ("agentkern.carbon.water_liters", self.total_water_liters.to_string()),
            ("agentkern.carbon.action_count", self.action_count.to_string()),
            ("agentkern.carbon.renewable_peak", self.is_renewable_peak.to_string()),
            ("agentkern.carbon.recommended_region", self.recommended_region.clone()),
        ]
    }

    /// Export as Prometheus-compatible gauge format.
    pub fn to_prometheus_format(&self) -> String {
        format!(
            "# HELP agentkern_carbon_co2_grams Total CO2 emissions in grams\n\
             # TYPE agentkern_carbon_co2_grams gauge\n\
             agentkern_carbon_co2_grams {:.4}\n\
             # HELP agentkern_carbon_energy_kwh Total energy consumed in kWh\n\
             # TYPE agentkern_carbon_energy_kwh gauge\n\
             agentkern_carbon_energy_kwh {:.6}\n\
             # HELP agentkern_carbon_water_liters Total water used in liters\n\
             # TYPE agentkern_carbon_water_liters gauge\n\
             agentkern_carbon_water_liters {:.6}\n\
             # HELP agentkern_carbon_action_count Number of recorded actions\n\
             # TYPE agentkern_carbon_action_count counter\n\
             agentkern_carbon_action_count {}\n\
             # HELP agentkern_carbon_renewable_peak Is current time within renewable peak\n\
             # TYPE agentkern_carbon_renewable_peak gauge\n\
             agentkern_carbon_renewable_peak {}\n",
            self.total_co2_grams,
            self.total_energy_kwh,
            self.total_water_liters,
            self.action_count,
            if self.is_renewable_peak { 1 } else { 0 }
        )
    }
}

// ============================================================================
// ERRORS
// ============================================================================


/// Carbon ledger errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CarbonError {
    #[error("Carbon budget exceeded for agent {agent_id}: limit={limit}, current={current}, requested={requested}")]
    BudgetExceeded {
        agent_id: AgentId,
        limit: Decimal,
        current: Decimal,
        requested: Decimal,
    },
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carbon_calculation() {
        let footprint = CarbonFootprint::calculate(
            "agent-1".to_string(),
            "inference",
            ComputeType::Gpu,
            60_000, // 1 minute
            CarbonRegion::UsAverage,
        );

        assert!(footprint.co2_grams > dec!(0));
        assert!(footprint.energy_kwh > dec!(0));
        assert!(footprint.water_liters > dec!(0));
    }

    #[test]
    fn test_region_intensity() {
        assert_eq!(CarbonRegion::Nordic.intensity(), 50);
        assert_eq!(CarbonRegion::India.intensity(), 700);
        assert!(CarbonRegion::Nordic.intensity() < CarbonRegion::UsAverage.intensity());
    }

    #[test]
    fn test_ledger_recording() {
        let ledger = CarbonLedger::new();

        let result = ledger.record_compute(
            "agent-1".to_string(),
            "test_action",
            ComputeType::Cpu,
            1000,
            None,
        );

        assert!(result.is_ok());

        let usage = ledger.get_daily_usage(&"agent-1".to_string());
        assert!(usage.total_co2_grams > dec!(0));
        assert_eq!(usage.action_count, 1);
    }

    #[test]
    fn test_budget_enforcement() {
        let ledger = CarbonLedger::new();

        // Set a very low budget
        ledger.set_budget(
            CarbonBudget::new("agent-1".to_string())
                .with_daily_limit(dec!(0.001))
                .block_on_exceed(),
        );

        // First small action should work
        let _ = ledger.record_compute(
            "agent-1".to_string(),
            "small",
            ComputeType::Storage,
            1,
            None,
        );

        // Large action should be blocked
        let result = ledger.record_compute(
            "agent-1".to_string(),
            "large",
            ComputeType::Gpu,
            3600_000, // 1 hour
            None,
        );

        assert!(matches!(result, Err(CarbonError::BudgetExceeded { .. })));
    }

    #[test]
    fn test_fleet_usage() {
        let ledger = CarbonLedger::new();

        for i in 0..5 {
            let _ = ledger.record_compute(
                format!("agent-{}", i),
                "action",
                ComputeType::Cpu,
                1000,
                None,
            );
        }

        let fleet = ledger.get_fleet_usage();
        assert_eq!(fleet.action_count, 5);
    }

    #[test]
    fn test_region_recommendation() {
        let ledger = CarbonLedger::new();
        let recommended = ledger.recommend_region();

        // Should recommend cleanest
        assert_eq!(recommended.intensity(), 50);
    }

    #[test]
    fn test_estimate() {
        let estimate = CarbonLedger::estimate(
            ComputeType::Gpu,
            3600_000, // 1 hour
            CarbonRegion::UsAverage,
        );

        // H100 at 400W for 1 hour = 0.4 kWh
        // At 400 gCO2/kWh = 160g CO2
        assert!(estimate.co2_grams > dec!(100));
        assert!(estimate.co2_grams < dec!(200));
    }
}
