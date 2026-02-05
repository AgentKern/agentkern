//! Environmental, Social, and Governance (ESG) Standards
//!
//! Shared types and traits for Carbon Tracking and GreenOps.

use serde::{Deserialize, Serialize};

/// Real-time carbon intensity feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonIntensityFeed {
    pub region: String,
    pub intensity_gco2_kwh: f64,
    pub fossil_fuel_percentage: f64,
    pub renewable_percentage: f64,
    pub nuclear_percentage: f64,
    pub timestamp: String,
    pub forecast_24h: Vec<ForecastPoint>,
}

/// Forecast data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub hour: u32,
    pub intensity: f64,
}

/// Region data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionData {
    pub region: String,
    pub current_intensity: f64,
    pub is_low_carbon: bool,
    pub recommended: bool,
    pub details: CarbonIntensityFeed,
}

/// Interface for Carbon Grid Data Providers.
///
/// Allows decoupling the Treasury (OS) from specific Energy (EE) implementations.
/// Interface for Carbon Grid Data Providers.
///
/// Allows decoupling the Treasury (OS) from specific Energy (EE) implementations.
#[async_trait::async_trait]
pub trait GridApi: Send + Sync {
    /// Get real-time carbon intensity for a region.
    async fn get_intensity(&self, region: &str) -> Result<CarbonIntensityFeed, String>;

    /// Get all regions data.
    async fn get_all_regions(&self) -> Result<Vec<RegionData>, String>;

    /// Find the greenest region among candidates.
    async fn find_greenest(&self, regions: &[&str]) -> Result<String, String>;
}
