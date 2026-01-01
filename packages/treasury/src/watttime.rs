//! WattTime API Client - Dynamic Carbon Intensity
//!
//! Per 2026 Roadmap: Replace static carbon averages with real-time grid data.
//! WattTime v3 API provides marginal emissions for electric grids worldwide.
//!
//! <https://docs.watttime.org>

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// WattTime API errors.
#[derive(Debug, Error)]
pub enum WattTimeError {
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("API request failed: {0}")]
    RequestFailed(String),
    #[error("Rate limited: retry after {0}s")]
    RateLimited(u32),
    #[error("Region not found: {0}")]
    RegionNotFound(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Carbon intensity data point from WattTime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntensityData {
    /// Marginal operating emissions rate (lbs CO2/MWh)
    pub moer: f64,
    /// Frequency of the data (e.g., "5m")
    pub frequency: String,
    /// Balancing authority (grid region)
    pub ba: String,
    /// Point time (ISO 8601)
    pub point_time: String,
}

/// Forecast data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    /// Point time (ISO 8601)
    pub point_time: String,
    /// Forecasted MOER value
    pub value: f64,
}

/// WattTime API client configuration.
#[derive(Debug, Clone)]
pub struct WattTimeConfig {
    /// API base URL (default: https://api.watttime.org)
    pub base_url: String,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
}

impl Default for WattTimeConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.watttime.org".to_string(),
            username: String::new(),
            password: String::new(),
        }
    }
}

/// WattTime API client for dynamic carbon intensity.
///
/// # Example
/// ```rust,ignore
/// let client = WattTimeClient::new(config).await?;
/// let intensity = client.get_intensity(37.7749, -122.4194).await?;
/// println!("Current intensity: {} gCO2/kWh", intensity);
/// ```
#[derive(Debug)]
pub struct WattTimeClient {
    config: WattTimeConfig,
    /// Cached auth token
    token: Option<String>,
    /// Token expiry (30 min from login)
    token_expiry: Option<std::time::Instant>,
}

impl WattTimeClient {
    /// Create a new WattTime client.
    pub fn new(config: WattTimeConfig) -> Self {
        Self {
            config,
            token: None,
            token_expiry: None,
        }
    }

    /// Create with environment variables.
    pub fn from_env() -> Result<Self, WattTimeError> {
        let username = std::env::var("WATTTIME_USERNAME")
            .map_err(|_| WattTimeError::AuthFailed("WATTTIME_USERNAME not set".into()))?;
        let password = std::env::var("WATTTIME_PASSWORD")
            .map_err(|_| WattTimeError::AuthFailed("WATTTIME_PASSWORD not set".into()))?;

        Ok(Self::new(WattTimeConfig {
            username,
            password,
            ..Default::default()
        }))
    }

    /// Get current carbon intensity for a location.
    /// Returns gCO2/kWh (converted from lbs/MWh).
    pub async fn get_intensity(&self, lat: f64, lon: f64) -> Result<u32, WattTimeError> {
        // In production, this would:
        // 1. Get auth token if expired
        // 2. Call /v3/signal-index with lat/lon
        // 3. Convert MOER (lbs CO2/MWh) to gCO2/kWh

        // Placeholder: return regional average based on rough lat/lon
        // This allows the code to compile and test without API keys
        let intensity = self.estimate_from_location(lat, lon);
        Ok(intensity)
    }

    /// Get intensity forecast for a region.
    pub async fn get_forecast(&self, _ba: &str) -> Result<Vec<ForecastPoint>, WattTimeError> {
        use chrono::Timelike;

        // Placeholder: return mock forecast
        let now = chrono::Utc::now();
        let mut forecast = Vec::new();

        for i in 0..24 {
            let point_time = now + chrono::Duration::hours(i);
            // Simulate sinusoidal pattern (lower at midday due to solar)
            let hour = point_time.hour() as f64;
            let value = 400.0 + 100.0 * (hour * std::f64::consts::PI / 12.0).sin();

            forecast.push(ForecastPoint {
                point_time: point_time.to_rfc3339(),
                value,
            });
        }

        Ok(forecast)
    }

    /// Get the balancing authority (grid region) for a location.
    pub async fn get_region(&self, lat: f64, lon: f64) -> Result<String, WattTimeError> {
        // Placeholder: estimate region from lat/lon
        let region = if lon < -100.0 {
            "CAISO_NORTH" // California
        } else if lon < -80.0 {
            "PJM" // Mid-Atlantic
        } else if lon > 100.0 {
            "CNGRID" // China
        } else if lat > 50.0 && lon > -10.0 && lon < 40.0 {
            "EUGRID" // Europe
        } else {
            "UNKNOWN"
        };

        Ok(region.to_string())
    }

    /// Estimate intensity from lat/lon (fallback when API unavailable).
    fn estimate_from_location(&self, lat: f64, lon: f64) -> u32 {
        // Rough estimates based on grid carbon intensity by region
        if lon < -100.0 && lat > 32.0 && lat < 42.0 {
            // California (high solar)
            250
        } else if lon > -10.0 && lon < 40.0 && lat > 48.0 && lat < 60.0 {
            // Northern Europe (high wind/nuclear)
            200
        } else if lon > 100.0 && lon < 140.0 && lat > 20.0 && lat < 45.0 {
            // China (coal-heavy)
            550
        } else if lon > 70.0 && lon < 90.0 && lat > 8.0 && lat < 35.0 {
            // India (coal-heavy)
            700
        } else {
            // US/World average
            400
        }
    }

    /// Check if token needs refresh.
    pub fn needs_auth(&self) -> bool {
        match self.token_expiry {
            Some(expiry) => std::time::Instant::now() > expiry,
            None => true,
        }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &WattTimeConfig {
        &self.config
    }

    /// Check if credentials are configured.
    pub fn has_credentials(&self) -> bool {
        !self.config.username.is_empty() && !self.config.password.is_empty()
    }

    /// Check if currently authenticated (has valid token).
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some() && !self.needs_auth()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = WattTimeConfig::default();
        assert_eq!(config.base_url, "https://api.watttime.org");
    }

    #[test]
    fn test_estimate_california() {
        let client = WattTimeClient::new(WattTimeConfig::default());
        let intensity = client.estimate_from_location(37.7749, -122.4194);
        assert_eq!(intensity, 250); // California is clean
    }

    #[test]
    fn test_estimate_china() {
        let client = WattTimeClient::new(WattTimeConfig::default());
        let intensity = client.estimate_from_location(31.2304, 121.4737);
        assert_eq!(intensity, 550); // Shanghai is coal-heavy
    }

    #[tokio::test]
    async fn test_get_region() {
        let client = WattTimeClient::new(WattTimeConfig::default());
        let region = client.get_region(37.7749, -122.4194).await.unwrap();
        assert_eq!(region, "CAISO_NORTH");
    }

    #[tokio::test]
    async fn test_get_forecast() {
        let client = WattTimeClient::new(WattTimeConfig::default());
        let forecast = client.get_forecast("CAISO_NORTH").await.unwrap();
        assert_eq!(forecast.len(), 24);
    }
}
