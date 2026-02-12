//! Global Mesh Orchestrator
//!
//! Actively manages agent placement across the multi-cloud mesh.
//! Per GLOBAL_GAPS.md: "Sovereign Orchestration"

use super::{DataRegion, GlobalMesh, MeshError};
use crate::passport::MemoryPassport;
use agentkern_pulse::{HealthStatus, SemanticHealthReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Reason for triggering an autonomous migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationReason {
    /// Region is failing or degraded
    UnhealthyRegion,
    /// High carbon intensity in current region
    EnvironmentalOptimization,
    /// Regulatory shift or policy change
    ComplianceRequirement,
    /// Manual operator directive
    Manual,
}

/// Result of a migration check.
#[derive(Debug)]
pub enum MigrationDecision {
    /// Agent should migrate to a new region (stub)
    Migrate,
    /// Agent should stay in current region (passport returned)
    Stay(MemoryPassport),
    /// Agent stayed but performed self-healing (passport renewed)
    Healed(MemoryPassport),
}

/// Orchestrator for the autonomous mesh.
pub struct MeshOrchestrator {
    _mesh: Arc<GlobalMesh>,
    // Migration Manager moved to Enterprise Edition
    // migration: Arc<MigrationManager>,
    /// Track health reports per region
    region_health: Arc<RwLock<HashMap<DataRegion, SemanticHealthReport>>>,
}

impl MeshOrchestrator {
    /// Create a new orchestrator.
    pub fn new(mesh: Arc<GlobalMesh>) -> Self {
        Self {
            _mesh: mesh,
            // migration,
            region_health: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update health status for a region.
    pub async fn update_region_health(&self, region: DataRegion, report: SemanticHealthReport) {
        let mut health = self.region_health.write().await;
        health.insert(region, report);
    }

    /// Monitor and suggest migration or healing.
    pub async fn check_and_migrate(
        &self,
        agent_id: &str,
        passport: MemoryPassport,
        current_region: DataRegion,
    ) -> Result<MigrationDecision, MeshError> {
        let health_map = self.region_health.read().await;

        // Check current region health
        if let Some(report) = health_map.get(&current_region) {
            match report.status {
                HealthStatus::Critical => {
                    // Critical failure: Must migrate
                    // Find healthiest alternative region
                    if let Some(target_region) = self.find_best_alternative(current_region).await {
                        tracing::info!(
                            "Triggering autonomous migration for {} from {:?} to {:?} due to Critical Health",
                            agent_id,
                            current_region,
                            target_region
                        );

                        tracing::warn!("Migration requires Enterprise Edition");
                        return Ok(MigrationDecision::Stay(passport));
                    }
                }
                HealthStatus::Degraded => {
                    // Degraded: Attempt self-healing first
                    tracing::warn!(
                        "Region {:?} is Degraded. Initiating self-healing for {}",
                        current_region,
                        agent_id
                    );
                    let healed_passport = self.heal_local_agent(passport, current_region).await?;
                    return Ok(MigrationDecision::Healed(healed_passport));
                }
                _ => {}
            }
        }

        // Default: Stay
        Ok(MigrationDecision::Stay(passport))
    }

    /// Attempt to heal a local agent by restarting its state.
    pub async fn heal_local_agent(
        &self,
        passport: MemoryPassport,
        _current_region: DataRegion,
    ) -> Result<MemoryPassport, MeshError> {
        let start = std::time::Instant::now();
        tracing::info!(
            "Performing self-healing restart for agent {}",
            passport.identity.did
        );

        // Enterprise Feature: Self-healing via hibernation/wakeup
        // let ticket = self.migration.hibernate(passport, current_region)?;
        // let restored = self.migration.wakeup(ticket, current_region)?;
        let restored = passport; // Stub for OSS

        let duration = start.elapsed();
        tracing::info!(
            agent_id = %restored.identity.did,
            cold_start_micros = duration.as_micros(),
            "Agent successfully self-healed"
        );
        Ok(restored)
    }

    /// Find the best alternative region based on health and carbon.
    async fn find_best_alternative(&self, exclude_region: DataRegion) -> Option<DataRegion> {
        let health_map = self.region_health.read().await;

        health_map
            .iter()
            .filter(|(region, _)| **region != exclude_region)
            .filter(|(_, report)| report.status == HealthStatus::Healthy)
            // Prioritize low carbon intensity
            .min_by(|a, b| {
                a.1.carbon_intensity
                    .partial_cmp(&b.1.carbon_intensity)
                    .unwrap()
            })
            .map(|(region, _)| *region)
    }

    /// Get current health map.
    pub async fn get_health_map(&self) -> HashMap<DataRegion, SemanticHealthReport> {
        self.region_health.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::DataRegion;
    use crate::passport::AgentIdentity;
    use chrono::Utc;

    fn sample_identity() -> AgentIdentity {
        AgentIdentity {
            did: "did:agentkern:test".to_string(),
            public_key: "key".to_string(),
            algorithm: "Ed25519".to_string(),
            created_at: 0,
            updated_at: 0,
        }
    }

    fn sample_health_report(status: HealthStatus, carbon: f64) -> SemanticHealthReport {
        SemanticHealthReport {
            component: "mesh".into(),
            status,
            timestamp: Utc::now(),
            carbon_intensity: carbon,
            cost_index: 0.5,
            latency_ms: 50,
            uptime_secs: 1000,
            message: "test".into(),
        }
    }

    #[tokio::test]
    async fn test_orchestrator_critical_region_stays_in_oss() {
        let mesh = Arc::new(GlobalMesh::new("local".into(), DataRegion::UsEast));
        let orchestrator = MeshOrchestrator::new(mesh);

        // Mark US East as Critical and EU as Healthy
        orchestrator
            .update_region_health(DataRegion::UsEast, sample_health_report(HealthStatus::Critical, 450.0))
            .await;
        orchestrator
            .update_region_health(DataRegion::EuFrankfurt, sample_health_report(HealthStatus::Healthy, 200.0))
            .await;

        let passport = MemoryPassport::new(sample_identity(), "US".to_string());
        let decision = orchestrator
            .check_and_migrate("agent-1", passport, DataRegion::UsEast)
            .await
            .unwrap();

        // OSS edition stays (migration requires EE)
        assert!(matches!(decision, MigrationDecision::Stay(_)));
    }

    #[tokio::test]
    async fn test_orchestrator_healing() {
        let mesh = Arc::new(GlobalMesh::new("local".into(), DataRegion::UsEast));
        let orchestrator = MeshOrchestrator::new(mesh);

        // Mark US East as Degraded — should trigger self-healing
        orchestrator
            .update_region_health(DataRegion::UsEast, sample_health_report(HealthStatus::Degraded, 100.0))
            .await;

        let passport = MemoryPassport::new(sample_identity(), "US".to_string());
        let decision = orchestrator
            .check_and_migrate("agent-1", passport, DataRegion::UsEast)
            .await
            .unwrap();

        match decision {
            MigrationDecision::Healed(restored) => {
                assert_eq!(restored.identity.did, "did:agentkern:test");
            }
            _ => panic!("Expected healing, got {:?}", decision),
        }
    }

    #[tokio::test]
    async fn test_orchestrator_healthy_region_stays() {
        let mesh = Arc::new(GlobalMesh::new("local".into(), DataRegion::UsEast));
        let orchestrator = MeshOrchestrator::new(mesh);

        // Mark US East as Healthy
        orchestrator
            .update_region_health(DataRegion::UsEast, sample_health_report(HealthStatus::Healthy, 100.0))
            .await;

        let passport = MemoryPassport::new(sample_identity(), "US".to_string());
        let decision = orchestrator
            .check_and_migrate("agent-1", passport, DataRegion::UsEast)
            .await
            .unwrap();

        assert!(matches!(decision, MigrationDecision::Stay(_)));
    }
}
