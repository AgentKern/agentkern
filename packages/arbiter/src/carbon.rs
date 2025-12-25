//! VeriMantle-Arbiter: Carbon-Aware Computing
//!
//! Per EXECUTION_MANDATE.md §7: "Carbon-Aware & Sustainable Computing"
//!
//! Features:
//! - Green region preference
//! - Carbon intensity tracking
//! - Emissions per transaction
//! - Sustainable scheduling
//!
//! # Example
//!
//! ```rust,ignore
//! use verimantle_arbiter::carbon::{CarbonScheduler, Region};
//!
//! let scheduler = CarbonScheduler::new();
//! let best_region = scheduler.select_greenest_region(&["us-east-1", "eu-west-1"]);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Carbon intensity level (gCO2eq/kWh).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CarbonIntensity {
    /// Green (< 100 gCO2eq/kWh)
    Green,
    /// Low (100-300 gCO2eq/kWh)
    Low,
    /// Medium (300-500 gCO2eq/kWh)
    Medium,
    /// High (> 500 gCO2eq/kWh)
    High,
}

impl CarbonIntensity {
    /// Get the approximate gCO2eq/kWh for this level.
    pub fn grams_per_kwh(&self) -> u32 {
        match self {
            Self::Green => 50,
            Self::Low => 200,
            Self::Medium => 400,
            Self::High => 600,
        }
    }
}

/// Region with carbon data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonRegion {
    /// Region identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Current carbon intensity
    pub intensity: CarbonIntensity,
    /// Renewable percentage
    pub renewable_pct: u8,
    /// Real-time gCO2eq/kWh
    pub current_grams_per_kwh: u32,
    /// Is this a green region?
    pub is_green: bool,
}

/// Emissions record for a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionsRecord {
    /// Transaction ID
    pub transaction_id: String,
    /// Region where executed
    pub region: String,
    /// Energy consumed (kWh)
    pub energy_kwh: f64,
    /// Carbon emitted (gCO2eq)
    pub carbon_grams: f64,
    /// Timestamp
    pub timestamp: u64,
    /// Scope (1, 2, or 3)
    pub scope: EmissionScope,
}

/// GHG Protocol emission scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmissionScope {
    /// Direct emissions
    Scope1,
    /// Indirect (electricity)
    Scope2,
    /// Value chain
    Scope3,
}

/// Carbon scheduler for sustainable execution.
#[derive(Debug)]
pub struct CarbonScheduler {
    /// Region data
    regions: HashMap<String, CarbonRegion>,
    /// Total emissions tracked
    total_emissions_grams: f64,
    /// Transaction count
    transaction_count: u64,
}

impl Default for CarbonScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl CarbonScheduler {
    /// Create a new carbon scheduler with default region data.
    pub fn new() -> Self {
        let mut regions = HashMap::new();
        
        // Green regions (100% renewable)
        regions.insert("eu-north-1".to_string(), CarbonRegion {
            id: "eu-north-1".to_string(),
            name: "Stockholm (Sweden)".to_string(),
            intensity: CarbonIntensity::Green,
            renewable_pct: 98,
            current_grams_per_kwh: 20,
            is_green: true,
        });
        
        regions.insert("eu-west-1-ireland".to_string(), CarbonRegion {
            id: "eu-west-1-ireland".to_string(),
            name: "Ireland (Wind)".to_string(),
            intensity: CarbonIntensity::Green,
            renewable_pct: 85,
            current_grams_per_kwh: 80,
            is_green: true,
        });
        
        regions.insert("us-west-2-oregon".to_string(), CarbonRegion {
            id: "us-west-2-oregon".to_string(),
            name: "Oregon (Hydro)".to_string(),
            intensity: CarbonIntensity::Green,
            renewable_pct: 90,
            current_grams_per_kwh: 50,
            is_green: true,
        });
        
        // Low carbon regions
        regions.insert("eu-central-1".to_string(), CarbonRegion {
            id: "eu-central-1".to_string(),
            name: "Frankfurt (Germany)".to_string(),
            intensity: CarbonIntensity::Low,
            renewable_pct: 55,
            current_grams_per_kwh: 250,
            is_green: false,
        });
        
        // Medium carbon regions
        regions.insert("us-east-1".to_string(), CarbonRegion {
            id: "us-east-1".to_string(),
            name: "Virginia (US)".to_string(),
            intensity: CarbonIntensity::Medium,
            renewable_pct: 30,
            current_grams_per_kwh: 380,
            is_green: false,
        });
        
        // High carbon regions
        regions.insert("ap-south-1".to_string(), CarbonRegion {
            id: "ap-south-1".to_string(),
            name: "Mumbai (India)".to_string(),
            intensity: CarbonIntensity::High,
            renewable_pct: 20,
            current_grams_per_kwh: 700,
            is_green: false,
        });
        
        Self {
            regions,
            total_emissions_grams: 0.0,
            transaction_count: 0,
        }
    }

    /// Get all green regions.
    pub fn green_regions(&self) -> Vec<&CarbonRegion> {
        self.regions.values().filter(|r| r.is_green).collect()
    }

    /// Select the greenest region from candidates.
    pub fn select_greenest<'a>(&self, candidates: &[&'a str]) -> Option<&'a str> {
        candidates
            .iter()
            .filter_map(|&id| {
                self.regions.get(id).map(|r| (id, r.current_grams_per_kwh))
            })
            .min_by_key(|(_, grams)| *grams)
            .map(|(id, _)| id)
    }

    /// Calculate emissions for a workload.
    pub fn calculate_emissions(
        &self,
        region_id: &str,
        energy_kwh: f64,
    ) -> Option<f64> {
        self.regions.get(region_id).map(|r| {
            energy_kwh * r.current_grams_per_kwh as f64
        })
    }

    /// Record emissions for a transaction.
    pub fn record_transaction(
        &mut self,
        transaction_id: String,
        region_id: &str,
        energy_kwh: f64,
    ) -> Option<EmissionsRecord> {
        let region = self.regions.get(region_id)?;
        let carbon_grams = energy_kwh * region.current_grams_per_kwh as f64;
        
        self.total_emissions_grams += carbon_grams;
        self.transaction_count += 1;
        
        Some(EmissionsRecord {
            transaction_id,
            region: region_id.to_string(),
            energy_kwh,
            carbon_grams,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            scope: EmissionScope::Scope2,
        })
    }

    /// Get total emissions (gCO2eq).
    pub fn total_emissions(&self) -> f64 {
        self.total_emissions_grams
    }

    /// Get average emissions per transaction.
    pub fn avg_emissions_per_transaction(&self) -> f64 {
        if self.transaction_count == 0 {
            return 0.0;
        }
        self.total_emissions_grams / self.transaction_count as f64
    }

    /// Get carbon savings by using green region vs average.
    pub fn carbon_savings(&self, region_id: &str, energy_kwh: f64) -> f64 {
        let region = match self.regions.get(region_id) {
            Some(r) => r,
            None => return 0.0,
        };
        
        // Average global grid intensity ~500 gCO2eq/kWh
        let average_intensity = 500.0;
        let actual_intensity = region.current_grams_per_kwh as f64;
        
        (average_intensity - actual_intensity) * energy_kwh
    }

    /// Check if scheduling in off-peak hours is recommended.
    pub fn recommend_off_peak(&self, region_id: &str) -> bool {
        self.regions.get(region_id)
            .map(|r| r.intensity == CarbonIntensity::High || r.intensity == CarbonIntensity::Medium)
            .unwrap_or(false)
    }
}

/// Carbon metrics for reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonMetrics {
    pub total_emissions_kg: f64,
    pub transactions: u64,
    pub avg_emissions_per_transaction_g: f64,
    pub green_region_percentage: f64,
    pub carbon_saved_kg: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_green_regions() {
        let scheduler = CarbonScheduler::new();
        let green = scheduler.green_regions();
        
        assert!(!green.is_empty());
        assert!(green.iter().all(|r| r.is_green));
    }

    #[test]
    fn test_select_greenest() {
        let scheduler = CarbonScheduler::new();
        
        let candidates = ["us-east-1", "eu-north-1", "ap-south-1"];
        let greenest = scheduler.select_greenest(&candidates);
        
        // eu-north-1 (Stockholm) should be greenest
        assert_eq!(greenest, Some("eu-north-1"));
    }

    #[test]
    fn test_calculate_emissions() {
        let scheduler = CarbonScheduler::new();
        
        // 1 kWh in Stockholm (20 gCO2/kWh)
        let emissions = scheduler.calculate_emissions("eu-north-1", 1.0);
        assert_eq!(emissions, Some(20.0));
        
        // 1 kWh in Mumbai (700 gCO2/kWh)
        let emissions = scheduler.calculate_emissions("ap-south-1", 1.0);
        assert_eq!(emissions, Some(700.0));
    }

    #[test]
    fn test_record_transaction() {
        let mut scheduler = CarbonScheduler::new();
        
        let record = scheduler.record_transaction(
            "tx-1".to_string(),
            "eu-north-1",
            0.001, // 1 Wh
        );
        
        assert!(record.is_some());
        let record = record.unwrap();
        assert_eq!(record.carbon_grams, 0.02); // 0.001 kWh * 20 g/kWh
    }

    #[test]
    fn test_carbon_savings() {
        let scheduler = CarbonScheduler::new();
        
        // Using Stockholm (20 g/kWh) vs global average (500 g/kWh)
        let savings = scheduler.carbon_savings("eu-north-1", 1.0);
        assert_eq!(savings, 480.0); // 500 - 20 = 480 gCO2 saved per kWh
    }

    #[test]
    fn test_off_peak_recommendation() {
        let scheduler = CarbonScheduler::new();
        
        // Green region - no off-peak needed
        assert!(!scheduler.recommend_off_peak("eu-north-1"));
        
        // High carbon region - recommend off-peak
        assert!(scheduler.recommend_off_peak("ap-south-1"));
    }
}
