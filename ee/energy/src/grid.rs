//! Real-Time Grid API
//!
//! Live carbon intensity data from electricity grids

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Grid API for real-time carbon data.
pub struct GridApi {
    providers: Vec<GridProvider>,
    cache: HashMap<String, RegionData>,
}

/// Grid data provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridProvider {
    pub name: String,
    pub api_key: Option<String>,
    pub regions: Vec<String>,
}

impl GridApi {
    /// Create new Grid API client.
    pub fn new() -> Result<Self, GridError> {
        agentkern_connectors_ee::license::check_feature_license("grid_api")?;

        Ok(Self {
            providers: vec![
                GridProvider {
                    name: "ElectricityMaps".into(),
                    api_key: None,
                    regions: vec!["US".into(), "EU".into(), "UK".into()],
                },
                GridProvider {
                    name: "WattTime".into(),
                    api_key: None,
                    regions: vec!["US".into()],
                },
            ],
            cache: HashMap::new(),
        })
    }

    /// Configure API key for provider.
    pub fn with_api_key(&mut self, provider: &str, key: &str) -> &mut Self {
        for p in &mut self.providers {
            if p.name == provider {
                p.api_key = Some(key.to_string());
            }
        }
        self
    }

    /// Get real-time carbon intensity for region.
    pub fn get_intensity(&self, region: &str) -> Result<CarbonIntensityFeed, GridError> {
        // Would call real API (ElectricityMaps, WattTime, etc.)
        Ok(CarbonIntensityFeed {
            region: region.to_string(),
            intensity_gco2_kwh: self.get_mock_intensity(region),
            fossil_fuel_percentage: 35.0,
            renewable_percentage: 45.0,
            nuclear_percentage: 20.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            forecast_24h: self.get_mock_forecast(region),
        })
    }

    /// Get all regions' data.
    pub fn get_all_regions(&self) -> Result<Vec<RegionData>, GridError> {
        let regions = vec!["us-east-1", "eu-west-1", "ap-southeast-1"];
        regions.iter().map(|r| self.get_region_data(r)).collect()
    }

    /// Get detailed region data.
    pub fn get_region_data(&self, region: &str) -> Result<RegionData, GridError> {
        let intensity = self.get_intensity(region)?;

        Ok(RegionData {
            region: region.to_string(),
            current_intensity: intensity.intensity_gco2_kwh,
            is_low_carbon: intensity.intensity_gco2_kwh < 200.0,
            recommended: intensity.intensity_gco2_kwh < 150.0,
            details: intensity,
        })
    }

    /// Find lowest carbon region from list.
    pub fn find_greenest(&self, regions: &[&str]) -> Result<String, GridError> {
        let data: Result<Vec<_>, _> = regions.iter().map(|r| self.get_region_data(r)).collect();

        let data = data?;
        data.into_iter()
            .min_by(|a, b| {
                a.current_intensity
                    .partial_cmp(&b.current_intensity)
                    .unwrap()
            })
            .map(|d| d.region)
            .ok_or(GridError::NoRegionsAvailable)
    }

    fn get_mock_intensity(&self, region: &str) -> f64 {
        // Simulated real-time data
        match region {
            r if r.contains("eu") => 180.0,      // Europe is generally greener
            r if r.contains("us-west") => 200.0, // US West has more renewables
            r if r.contains("us-east") => 350.0, // US East is more fossil
            _ => 250.0,
        }
    }

    fn get_mock_forecast(&self, region: &str) -> Vec<ForecastPoint> {
        // 24-hour forecast
        (0..24)
            .map(|h| ForecastPoint {
                hour: h,
                intensity: self.get_mock_intensity(region) + (h as f64 * 5.0).sin() * 50.0,
            })
            .collect()
    }
}

impl Default for GridApi {
    fn default() -> Self {
        Self {
            providers: vec![],
            cache: HashMap::new(),
        }
    }
}

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

/// Grid API errors.
#[derive(Debug, thiserror::Error)]
pub enum GridError {
    #[error("No regions available")]
    NoRegionsAvailable,

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Region not supported: {0}")]
    RegionNotSupported(String),

    #[error("License error: {0}")]
    LicenseError(#[from] agentkern_connectors_ee::license::LicenseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_intensity() {
        let api = GridApi::default();
        assert!(api.get_mock_intensity("eu-west-1") < api.get_mock_intensity("us-east-1"));
    }
}
