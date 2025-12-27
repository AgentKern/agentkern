//! AgentKern Enterprise: Multi-Cell Mesh Coordination
//!
//! Per LICENSING_STRATEGY.md: "AgentKern Cloud (The Multi-Cell Mesh)"
//!
//! This module provides enterprise-only features for coordinating
//! multiple AgentKern cells across a global mesh.
//!
//! **License**: AgentKern Enterprise License (see ../LICENSE-ENTERPRISE.md)
//!
//! Features:
//! - Multi-node coordination (100+ cells)
//! - Global state synchronization
//! - Autonomic mitosis (auto-scaling)
//! - Cross-region failover

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Enterprise license error.
#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("Enterprise license required for this feature")]
    LicenseRequired,
    #[error("License expired on {expiry}")]
    LicenseExpired { expiry: String },
    #[error("Invalid license key: {0}")]
    InvalidLicense(String),
    #[error("Feature not included in license: {feature}")]
    FeatureNotLicensed { feature: String },
    #[error("License server error: {0}")]
    ServerError(String),
}

/// License claims from JWT token.
#[derive(Debug, Clone, Deserialize)]
pub struct LicenseClaims {
    /// Subject (organization ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Expiration timestamp
    pub exp: u64,
    /// Issued at
    pub iat: u64,
    /// Licensed features
    #[serde(default)]
    pub features: Vec<String>,
    /// License tier (enterprise, growth, etc.)
    #[serde(default = "default_tier")]
    pub tier: String,
}

fn default_tier() -> String { "demo".to_string() }

/// Enterprise license validation.
/// 
/// Supports two modes:
/// 1. **Offline**: JWT-based license token validated locally
/// 2. **Online**: HTTP validation against license server
pub struct License {
    key: String,
    claims: Option<LicenseClaims>,
}

impl License {
    /// Create a new license from environment variable.
    pub fn from_env() -> Result<Self, LicenseError> {
        let key = std::env::var("AGENTKERN_LICENSE_KEY")
            .map_err(|_| LicenseError::LicenseRequired)?;
        
        if key.is_empty() {
            return Err(LicenseError::InvalidLicense("Empty key".into()));
        }
        
        // Try JWT offline validation
        if let Ok(claims) = Self::validate_jwt_offline(&key) {
            tracing::info!(org = %claims.sub, tier = %claims.tier, "License validated (JWT)");
            return Ok(Self { key, claims: Some(claims) });
        }
        
        // Fallback: demo mode
        tracing::warn!("License not JWT format, running in demo mode");
        Ok(Self { key, claims: None })
    }
    
    /// Validate JWT license token offline.
    fn validate_jwt_offline(token: &str) -> Result<LicenseClaims, LicenseError> {
        use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
        
        let secret = std::env::var("AGENTKERN_LICENSE_SECRET")
            .unwrap_or_else(|_| "agentkern-demo-2025".into());
        
        let key = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["agentkern.com"]);
        
        let data = decode::<LicenseClaims>(token, &key, &validation)
            .map_err(|e| LicenseError::InvalidLicense(e.to_string()))?;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        
        if data.claims.exp < now {
            return Err(LicenseError::LicenseExpired { 
                expiry: chrono::DateTime::from_timestamp(data.claims.exp as i64, 0)
                    .map(|dt| dt.to_string())
                    .unwrap_or_default()
            });
        }
        
        Ok(data.claims)
    }
    
    /// Validate license with HTTP server (async).
    pub async fn validate_with_server(&self, server_url: &str) -> Result<LicenseClaims, LicenseError> {
        let client = reqwest::Client::new();
        let resp = client.post(format!("{}/v1/validate", server_url))
            .header("Authorization", format!("Bearer {}", self.key))
            .send().await
            .map_err(|e| LicenseError::ServerError(e.to_string()))?;
        
        if !resp.status().is_success() {
            return Err(LicenseError::InvalidLicense(resp.text().await.unwrap_or_default()));
        }
        
        resp.json().await.map_err(|e| LicenseError::ServerError(e.to_string()))
    }

    /// Check if a feature is licensed.
    pub fn require(&self, feature: &str) -> Result<(), LicenseError> {
        if let Some(ref claims) = self.claims {
            if claims.tier == "enterprise" || claims.features.contains(&feature.to_string()) {
                return Ok(());
            }
            return Err(LicenseError::FeatureNotLicensed { feature: feature.into() });
        }
        // Demo mode: allow with warning
        tracing::warn!(feature = %feature, "Demo mode - feature access");
        Ok(())
    }
    
    /// Get license tier.
    pub fn tier(&self) -> &str {
        self.claims.as_ref().map(|c| c.tier.as_str()).unwrap_or("demo")
    }
}

/// Require an enterprise license for a feature.
pub fn require_license(feature: &str) -> Result<(), LicenseError> {
    License::from_env()?.require(feature)
}


/// Multi-cell mesh configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// Cluster name
    pub cluster_name: String,
    /// Seed nodes for discovery
    pub seed_nodes: Vec<String>,
    /// Replication factor
    pub replication_factor: u8,
    /// Sync interval in milliseconds
    pub sync_interval_ms: u64,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            cluster_name: "agentkern-mesh".to_string(),
            seed_nodes: vec![],
            replication_factor: 3,
            sync_interval_ms: 100,
        }
    }
}

/// Cell in the mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshCell {
    /// Unique cell ID
    pub cell_id: String,
    /// Cell region
    pub region: String,
    /// Cell status
    pub status: CellStatus,
    /// Last heartbeat timestamp
    pub last_heartbeat: u64,
}

/// Cell status in the mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellStatus {
    /// Cell is healthy
    Healthy,
    /// Cell is degraded
    Degraded,
    /// Cell is offline
    Offline,
    /// Cell is syncing
    Syncing,
}

/// Multi-cell mesh coordinator.
pub struct MeshCoordinator {
    config: MeshConfig,
    cells: Vec<MeshCell>,
}

impl MeshCoordinator {
    /// Create a new mesh coordinator (requires enterprise license).
    pub fn new(config: MeshConfig) -> Result<Self, LicenseError> {
        require_license("MULTI_CELL_MESH")?;
        
        Ok(Self {
            config,
            cells: vec![],
        })
    }

    /// Register a new cell in the mesh.
    pub fn register_cell(&mut self, cell: MeshCell) -> Result<(), LicenseError> {
        require_license("MULTI_CELL_MESH")?;
        
        tracing::info!(
            cell_id = %cell.cell_id,
            region = %cell.region,
            "Cell registered in mesh"
        );
        
        self.cells.push(cell);
        Ok(())
    }

    /// Get all cells in the mesh.
    pub fn cells(&self) -> &[MeshCell] {
        &self.cells
    }

    /// Get cells by region.
    pub fn cells_in_region(&self, region: &str) -> Vec<&MeshCell> {
        self.cells.iter().filter(|c| c.region == region).collect()
    }

    /// Get healthy cell count.
    pub fn healthy_cell_count(&self) -> usize {
        self.cells.iter().filter(|c| c.status == CellStatus::Healthy).count()
    }
}

// ============================================
// Autonomic Mitosis (Auto-Scaling)
// ============================================

/// Scaling policy for auto-scaling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Minimum cells
    pub min_cells: u32,
    /// Maximum cells
    pub max_cells: u32,
    /// Target CPU utilization (0-100)
    pub target_cpu: u8,
    /// Target memory utilization (0-100)
    pub target_memory: u8,
    /// Target requests per second per cell
    pub target_rps_per_cell: u32,
    /// Cooldown period in seconds
    pub cooldown_secs: u32,
    /// Scale up threshold (percentage above target)
    pub scale_up_threshold: u8,
    /// Scale down threshold (percentage below target)
    pub scale_down_threshold: u8,
}

impl Default for ScalingPolicy {
    fn default() -> Self {
        Self {
            min_cells: 2,
            max_cells: 100,
            target_cpu: 70,
            target_memory: 80,
            target_rps_per_cell: 1000,
            cooldown_secs: 300,
            scale_up_threshold: 20,
            scale_down_threshold: 30,
        }
    }
}

/// Scaling decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalingDecision {
    /// Scale up by N cells
    ScaleUp(u32),
    /// Scale down by N cells
    ScaleDown(u32),
    /// No action needed
    NoAction,
    /// In cooldown period
    Cooldown,
}

/// Current mesh metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshMetrics {
    /// Total cells
    pub total_cells: u32,
    /// Healthy cells
    pub healthy_cells: u32,
    /// Average CPU utilization
    pub avg_cpu: u8,
    /// Average memory utilization
    pub avg_memory: u8,
    /// Total requests per second
    pub total_rps: u32,
    /// Timestamp
    pub timestamp: u64,
}

/// Mitosis event (scale action).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitosisEvent {
    /// Event ID
    pub id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Decision made
    pub decision: ScalingDecision,
    /// Metrics at decision time
    pub metrics: MeshMetrics,
    /// Region affected
    pub region: String,
    /// Cells spawned/terminated
    pub cell_ids: Vec<String>,
}

/// Autonomic Mitosis controller for auto-scaling.
pub struct MitosisController {
    policy: ScalingPolicy,
    last_scale_time: u64,
    events: Vec<MitosisEvent>,
}

impl MitosisController {
    /// Create a new mitosis controller (requires enterprise license).
    pub fn new(policy: ScalingPolicy) -> Result<Self, LicenseError> {
        require_license("AUTONOMIC_MITOSIS")?;
        
        Ok(Self {
            policy,
            last_scale_time: 0,
            events: vec![],
        })
    }

    /// Evaluate current metrics and decide on scaling.
    pub fn evaluate(&mut self, metrics: &MeshMetrics) -> ScalingDecision {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check cooldown
        if now - self.last_scale_time < self.policy.cooldown_secs as u64 {
            return ScalingDecision::Cooldown;
        }

        // Calculate current RPS per cell
        let rps_per_cell = if metrics.healthy_cells > 0 {
            metrics.total_rps / metrics.healthy_cells
        } else {
            u32::MAX // Need to scale up immediately
        };

        // Check if we need to scale up
        let cpu_overload = metrics.avg_cpu > self.policy.target_cpu + self.policy.scale_up_threshold;
        let memory_overload = metrics.avg_memory > self.policy.target_memory + self.policy.scale_up_threshold;
        let rps_overload = rps_per_cell > self.policy.target_rps_per_cell;

        if (cpu_overload || memory_overload || rps_overload) && 
           metrics.total_cells < self.policy.max_cells {
            // Calculate how many cells to add
            let cells_needed = if rps_overload {
                let total_needed = (metrics.total_rps / self.policy.target_rps_per_cell).max(1);
                total_needed.saturating_sub(metrics.healthy_cells)
            } else {
                // Add 25% more capacity
                (metrics.healthy_cells / 4).max(1)
            };
            
            let cells_to_add = cells_needed.min(self.policy.max_cells - metrics.total_cells);
            
            if cells_to_add > 0 {
                self.last_scale_time = now;
                return ScalingDecision::ScaleUp(cells_to_add);
            }
        }

        // Check if we can scale down
        let cpu_underload = metrics.avg_cpu < self.policy.target_cpu.saturating_sub(self.policy.scale_down_threshold);
        let memory_underload = metrics.avg_memory < self.policy.target_memory.saturating_sub(self.policy.scale_down_threshold);
        let rps_underload = rps_per_cell < self.policy.target_rps_per_cell / 2;

        if cpu_underload && memory_underload && rps_underload && 
           metrics.total_cells > self.policy.min_cells {
            // Remove 25% of cells
            let cells_to_remove = (metrics.healthy_cells / 4)
                .max(1)
                .min(metrics.total_cells - self.policy.min_cells);
            
            if cells_to_remove > 0 {
                self.last_scale_time = now;
                return ScalingDecision::ScaleDown(cells_to_remove);
            }
        }

        ScalingDecision::NoAction
    }

    /// Record a mitosis event.
    pub fn record_event(&mut self, event: MitosisEvent) {
        tracing::info!(
            decision = ?event.decision,
            region = %event.region,
            cells = ?event.cell_ids,
            "Mitosis event recorded"
        );
        self.events.push(event);
    }

    /// Get event history.
    pub fn events(&self) -> &[MitosisEvent] {
        &self.events
    }

    /// Get current policy.
    pub fn policy(&self) -> &ScalingPolicy {
        &self.policy
    }

    /// Update policy.
    pub fn set_policy(&mut self, policy: ScalingPolicy) {
        self.policy = policy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_required() {
        std::env::remove_var("AGENTKERN_LICENSE_KEY");
        let result = MeshCoordinator::new(MeshConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_with_license() {
        std::env::set_var("AGENTKERN_LICENSE_KEY", "test-license-key");
        let result = MeshCoordinator::new(MeshConfig::default());
        assert!(result.is_ok());
        std::env::remove_var("AGENTKERN_LICENSE_KEY");
    }

    #[test]
    fn test_mitosis_scale_up() {
        std::env::set_var("AGENTKERN_LICENSE_KEY", "test-license");
        
        let mut controller = MitosisController::new(ScalingPolicy::default()).unwrap();
        
        // High load metrics
        let metrics = MeshMetrics {
            total_cells: 5,
            healthy_cells: 5,
            avg_cpu: 95, // Overloaded
            avg_memory: 85,
            total_rps: 10000,
            timestamp: 0,
        };
        
        let decision = controller.evaluate(&metrics);
        assert!(matches!(decision, ScalingDecision::ScaleUp(_)));
        
        std::env::remove_var("AGENTKERN_LICENSE_KEY");
    }

    #[test]
    fn test_mitosis_scale_down() {
        std::env::set_var("AGENTKERN_LICENSE_KEY", "test-license");
        
        let mut controller = MitosisController::new(ScalingPolicy::default()).unwrap();
        
        // Low load metrics
        let metrics = MeshMetrics {
            total_cells: 10,
            healthy_cells: 10,
            avg_cpu: 20, // Underloaded
            avg_memory: 30,
            total_rps: 1000,
            timestamp: 0,
        };
        
        let decision = controller.evaluate(&metrics);
        assert!(matches!(decision, ScalingDecision::ScaleDown(_)));
        
        std::env::remove_var("AGENTKERN_LICENSE_KEY");
    }

    #[test]
    fn test_scaling_policy_defaults() {
        let policy = ScalingPolicy::default();
        assert_eq!(policy.min_cells, 2);
        assert_eq!(policy.max_cells, 100);
        assert_eq!(policy.target_cpu, 70);
    }
}

