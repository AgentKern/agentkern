//! Demo Grid API Implementation
//!
//! Works without credentials - returns realistic carbon intensity data

use super::grid::*;
use crate::core::{ConnectionMode, ConnectionStatus, GracefulService};

/// Demo grid API that works without credentials.
pub struct DemoGridApi {
    mode: ConnectionMode,
}

impl DemoGridApi {
    /// Create new demo grid API.
    pub fn new() -> Self {
        Self {
            mode: ConnectionMode::detect("grid"),
        }
    }
}

impl Default for DemoGridApi {
    fn default() -> Self {
        Self::new()
    }
}

impl GracefulService for DemoGridApi {
    fn mode(&self) -> ConnectionMode {
        self.mode
    }
    
    fn status(&self) -> ConnectionStatus {
        ConnectionStatus::new("grid")
    }
}

impl DemoGridApi {
    /// Get demo carbon intensity.
    pub fn get_intensity(&self, region: &str) -> CarbonIntensityFeed {
        let is_live = self.mode.is_live();
        let suffix = if is_live { "" } else { " [Demo]" };
        
        CarbonIntensityFeed {
            region: format!("{}{}", region, suffix),
            intensity_gco2_kwh: self.get_demo_intensity(region),
            fossil_fuel_percentage: 35.0,
            renewable_percentage: 45.0,
            nuclear_percentage: 20.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
            forecast_24h: (0..24).map(|h| ForecastPoint {
                hour: h,
                intensity: self.get_demo_intensity(region) + (h as f64 * 5.0).sin() * 30.0,
            }).collect(),
        }
    }
    
    fn get_demo_intensity(&self, region: &str) -> f64 {
        match region {
            r if r.contains("eu") => 180.0,
            r if r.contains("us-west") => 200.0,
            r if r.contains("us-east") => 350.0,
            _ => 250.0,
        }
    }
    
    /// Get all regions demo data.
    pub fn get_all_regions(&self) -> Vec<RegionData> {
        vec!["us-east-1", "eu-west-1", "ap-southeast-1"]
            .into_iter()
            .map(|r| {
                let intensity = self.get_intensity(r);
                RegionData {
                    region: r.to_string(),
                    current_intensity: intensity.intensity_gco2_kwh,
                    is_low_carbon: intensity.intensity_gco2_kwh < 200.0,
                    recommended: intensity.intensity_gco2_kwh < 150.0,
                    details: intensity,
                }
            })
            .collect()
    }
    
    /// Find greenest region.
    pub fn find_greenest(&self, regions: &[&str]) -> String {
        let data: Vec<_> = regions.iter()
            .map(|r| (r, self.get_intensity(r)))
            .collect();
        
        data.into_iter()
            .min_by(|a, b| a.1.intensity_gco2_kwh.partial_cmp(&b.1.intensity_gco2_kwh).unwrap())
            .map(|(r, _)| r.to_string())
            .unwrap_or_default()
    }
}

/// Factory to get the best available grid API.
pub struct GridFactory;

impl GridFactory {
    /// Get grid API with graceful fallback.
    pub fn get() -> DemoGridApi {
        DemoGridApi::new()
    }
    
    /// Get connection status.
    pub fn status() -> ConnectionStatus {
        ConnectionStatus::new("grid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_grid_works_without_credentials() {
        let api = DemoGridApi::new();
        assert!(api.is_available());
    }

    #[test]
    fn test_demo_grid_get_intensity() {
        let api = DemoGridApi::new();
        let intensity = api.get_intensity("eu-west-1");
        assert!(intensity.intensity_gco2_kwh > 0.0);
    }
}
