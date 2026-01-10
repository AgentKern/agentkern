//! WattTime API Client - Dynamic Carbon Intensity
//!
//! Per 2026 Roadmap: Replace static carbon averages with real-time grid data.
//! WattTime v3 API provides marginal emissions for electric grids worldwide.
//!
//! <https://docs.watttime.org>

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

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

/// Internal client state protected by Mutex.
#[derive(Debug, Default)]
struct ClientState {
    token: Option<String>,
    token_expiry: Option<std::time::Instant>,
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
    /// Thread-safe internal state
    state: Mutex<ClientState>,
}

impl WattTimeClient {
    /// Create a new WattTime client.
    pub fn new(config: WattTimeConfig) -> Self {
        Self {
            config,
            state: Mutex::new(ClientState::default()),
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
    #[cfg(feature = "http")]
    pub async fn get_intensity(&self, lat: f64, lon: f64) -> Result<u32, WattTimeError> {
        // Authenticate if needed
        if self.needs_auth().await {
            self.authenticate().await?;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/v3/signal-index", self.config.base_url);

        let token = {
            let state = self.state.lock().await;
            state.token.clone().unwrap_or_default()
        };

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[
                ("latitude", lat.to_string()),
                ("longitude", lon.to_string()),
            ])
            .send()
            .await
            .map_err(|e| WattTimeError::RequestFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(WattTimeError::RateLimited(60));
        }

        if !response.status().is_success() {
            // Fall back to location-based estimate
            tracing::warn!(
                "WattTime API returned {}, falling back to estimate",
                response.status()
            );
            return Ok(self.estimate_from_location(lat, lon));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WattTimeError::InvalidResponse(e.to_string()))?;

        // Extract MOER and convert: lbs CO2/MWh -> gCO2/kWh
        // 1 lbs = 453.592g, 1 MWh = 1000 kWh
        // So: lbs/MWh * 453.592 / 1000 = gCO2/kWh
        let moer = data
            .get("data")
            .and_then(|d| d.get(0))
            .and_then(|d| d.get("value"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let gco2_kwh = (moer * 453.592 / 1000.0) as u32;
        Ok(gco2_kwh)
    }

    /// Fallback implementation when http feature is disabled.
    #[cfg(not(feature = "http"))]
    pub async fn get_intensity(&self, lat: f64, lon: f64) -> Result<u32, WattTimeError> {
        // Return location-based estimate when API is not available
        Ok(self.estimate_from_location(lat, lon))
    }

    /// Authenticate with WattTime v3 API.
    #[cfg(feature = "http")]
    async fn authenticate(&self) -> Result<(), WattTimeError> {
        if !self.has_credentials() {
            return Err(WattTimeError::AuthFailed(
                "No credentials configured".into(),
            ));
        }

        let client = reqwest::Client::new();
        let url = format!("{}/login", self.config.base_url);

        let response = client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await
            .map_err(|e| WattTimeError::AuthFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(WattTimeError::AuthFailed(format!(
                "Login failed with status {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WattTimeError::AuthFailed(e.to_string()))?;

        let token = data
            .get("token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| WattTimeError::AuthFailed("No token in response".into()))?;

        let mut state = self.state.lock().await;
        state.token = Some(token.to_string());
        // Token expires in 30 minutes
        state.token_expiry = Some(std::time::Instant::now() + std::time::Duration::from_secs(1800));

        tracing::info!("WattTime: Authenticated successfully");
        Ok(())
    }

    /// Get intensity forecast for a region.
    #[cfg(feature = "http")]
    pub async fn get_forecast(&self, ba: &str) -> Result<Vec<ForecastPoint>, WattTimeError> {
        if self.needs_auth().await {
            self.authenticate().await?;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/v3/forecast", self.config.base_url);

        let token = {
            let state = self.state.lock().await;
            state.token.clone().unwrap_or_default()
        };

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[("ba", ba)])
            .send()
            .await
            .map_err(|e| WattTimeError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            // Fall back to mock forecast
            return self.mock_forecast();
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WattTimeError::InvalidResponse(e.to_string()))?;

        let forecast = data
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let point_time = item.get("point_time")?.as_str()?.to_string();
                        let value = item.get("value")?.as_f64()?;
                        Some(ForecastPoint { point_time, value })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(forecast)
    }

    /// Fallback forecast when http feature is disabled.
    #[cfg(not(feature = "http"))]
    pub async fn get_forecast(&self, _ba: &str) -> Result<Vec<ForecastPoint>, WattTimeError> {
        self.mock_forecast()
    }

    /// Generate mock forecast data (fallback).
    fn mock_forecast(&self) -> Result<Vec<ForecastPoint>, WattTimeError> {
        use chrono::Timelike;
        let now = chrono::Utc::now();
        let mut forecast = Vec::new();

        for i in 0..24 {
            let point_time = now + chrono::Duration::hours(i);
            let hour = point_time.hour() as f64;
            // Sinusoidal pattern: lower at midday due to solar
            let value = 400.0 + 100.0 * (hour * std::f64::consts::PI / 12.0).sin();

            forecast.push(ForecastPoint {
                point_time: point_time.to_rfc3339(),
                value,
            });
        }

        Ok(forecast)
    }

    /// Get the balancing authority (grid region) for a location.
    #[cfg(feature = "http")]
    pub async fn get_region(&self, lat: f64, lon: f64) -> Result<String, WattTimeError> {
        if self.needs_auth().await {
            self.authenticate().await?;
        }

        let client = reqwest::Client::new();
        let url = format!("{}/v3/region-from-loc", self.config.base_url);

        let token = {
            let state = self.state.lock().await;
            state.token.clone().unwrap_or_default()
        };

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[
                ("latitude", lat.to_string()),
                ("longitude", lon.to_string()),
            ])
            .send()
            .await
            .map_err(|e| WattTimeError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            // Fall back to location-based estimate
            return Ok(self.estimate_region(lat, lon));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| WattTimeError::InvalidResponse(e.to_string()))?;

        let region = data
            .get("ba")
            .and_then(|b| b.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();

        Ok(region)
    }

    /// Fallback region lookup when http feature is disabled.
    #[cfg(not(feature = "http"))]
    pub async fn get_region(&self, lat: f64, lon: f64) -> Result<String, WattTimeError> {
        Ok(self.estimate_region(lat, lon))
    }

    /// Estimate region from lat/lon (fallback).
    fn estimate_region(&self, lat: f64, lon: f64) -> String {
        if lon < -100.0 {
            "CAISO_NORTH".to_string() // California
        } else if lon < -80.0 {
            "PJM".to_string() // Mid-Atlantic
        } else if lon > 100.0 {
            "CNGRID".to_string() // China
        } else if lat > 50.0 && lon > -10.0 && lon < 40.0 {
            "EUGRID".to_string() // Europe
        } else {
            "UNKNOWN".to_string()
        }
    }

    /// Estimate intensity from lat/lon (fallback when API unavailable).
    fn estimate_from_location(&self, lat: f64, lon: f64) -> u32 {
        // Rough estimates based on grid carbon intensity by region
        if lon < -100.0 && lat > 32.0 && lat < 42.0 {
            250 // California (high solar)
        } else if lon > -10.0 && lon < 40.0 && lat > 48.0 && lat < 60.0 {
            200 // Northern Europe (high wind/nuclear)
        } else if lon > 100.0 && lon < 140.0 && lat > 20.0 && lat < 45.0 {
            550 // China (coal-heavy)
        } else if lon > 70.0 && lon < 90.0 && lat > 8.0 && lat < 35.0 {
            700 // India (coal-heavy)
        } else {
            400 // US/World average
        }
    }

    /// Check if token needs refresh.
    pub async fn needs_auth(&self) -> bool {
        let state = self.state.lock().await;
        match state.token_expiry {
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
    pub async fn is_authenticated(&self) -> bool {
        let state = self.state.lock().await;
        state.token.is_some()
            && match state.token_expiry {
                Some(expiry) => std::time::Instant::now() <= expiry,
                None => false,
            }
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
    #[ignore] // Requires WATTTIME_USERNAME/PASSWORD
    async fn test_get_region() {
        let client = WattTimeClient::new(WattTimeConfig::default());
        let region = client.get_region(37.7749, -122.4194).await.unwrap();
        assert_eq!(region, "CAISO_NORTH");
    }

    #[tokio::test]
    #[ignore] // Requires WATTTIME_USERNAME/PASSWORD
    async fn test_get_forecast() {
        let client = WattTimeClient::new(WattTimeConfig::default());
        let forecast = client.get_forecast("CAISO_NORTH").await.unwrap();
        assert_eq!(forecast.len(), 24);
    }
}
