//! Global Mesh Orchestrator
//!
//! Actively manages agent placement across the multi-cloud mesh.
//! Per GLOBAL_GAPS.md: "Sovereign Orchestration"

use super::{DataRegion, GlobalMesh, MeshError, MigrationManager, MigrationTicket};
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
    /// Agent should migrate to a new region
    Migrate(MigrationTicket),
    /// Agent should stay in current region (passport returned)
    Stay(MemoryPassport),
    /// Agent stayed but performed self-healing (passport renewed)
    Healed(MemoryPassport),
}

/// Orchestrator for the autonomous mesh.
pub struct MeshOrchestrator {
    _mesh: Arc<GlobalMesh>,
    migration: Arc<MigrationManager>,
    /// Track health reports per region
    region_health: Arc<RwLock<HashMap<DataRegion, SemanticHealthReport>>>,
}

impl MeshOrchestrator {
    /// Create a new orchestrator.
    pub fn new(mesh: Arc<GlobalMesh>, migration: Arc<MigrationManager>) -> Self {
        Self {
            mesh,
            migration,
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

                        let ticket = self.migration.hibernate(passport, target_region)?;
                        return Ok(MigrationDecision::Migrate(ticket));
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
        current_region: DataRegion,
    ) -> Result<MemoryPassport, MeshError> {
        let start = std::time::Instant::now();
        tracing::info!(
            "Performing self-healing restart for agent {}",
            passport.identity.did
        );

        // 1. Hibernate (Persistence Check)
        let ticket = self.migration.hibernate(passport, current_region)?;

        // 2. Restart (Reload)
        // Simulates a clean slate reload from encrypted state
        let restored = self.migration.wakeup(ticket, current_region)?;

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
            .filter(|(&region, _)| region != exclude_region)
            .filter(|(_, report)| report.status == HealthStatus::Healthy)
            // Prioritize low carbon intensity
            .min_by(|a, b| {
                a.1.carbon_intensity
                    .partial_cmp(&b.1.carbon_intensity)
                    .unwrap()
            })
            .map(|(&region, _)| region)
    }

    /// Get current health map.
    pub async fn get_health_map(&self) -> HashMap<DataRegion, SemanticHealthReport> {
        self.region_health.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::EncryptionEngine;
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

    #[tokio::test]
    async fn test_orchestrator_auto_migration() {
        let mesh = Arc::new(GlobalMesh::new("local".into(), DataRegion::UsEast));
        let encryption = EncryptionEngine::new();
        let migration = Arc::new(MigrationManager::new(encryption));
        let orchestrator = MeshOrchestrator::new(mesh, migration);

        // Mark US East as Critical
        orchestrator
            .update_region_health(
                DataRegion::UsEast,
                SemanticHealthReport {
                    component: "mesh".into(),
                    status: HealthStatus::Critical,
                    timestamp: Utc::now(),
                    carbon_intensity: 450.0,
                    cost_index: 0.5,
                    latency_ms: 500,
                    uptime_secs: 100,
                    message: "Power outage".into(),
                },
            )
            .await;

        // Mark EU Frankfurt as Healthy
        orchestrator
            .update_region_health(
                DataRegion::EuFrankfurt,
                SemanticHealthReport {
                    component: "mesh".into(),
                    status: HealthStatus::Healthy,
                    timestamp: Utc::now(),
                    carbon_intensity: 200.0,
                    cost_index: 0.4,
                    latency_ms: 50,
                    uptime_secs: 1000,
                    message: "All systems green".into(),
                },
            )
            .await;

        let passport = MemoryPassport::new(sample_identity(), "US".to_string());

        let decision = orchestrator
            .check_and_migrate("agent-1", passport, DataRegion::UsEast)
            .await
            .unwrap();

        match decision {
            MigrationDecision::Migrate(ticket) => {
                assert_eq!(ticket.target_region, DataRegion::EuFrankfurt);
            }
            _ => panic!("Expected migration, got {:?}", decision),
        }
    }

    #[tokio::test]
    async fn test_orchestrator_healing() {
        let mesh = Arc::new(GlobalMesh::new("local".into(), DataRegion::UsEast));
        let encryption = EncryptionEngine::new();
        let migration = Arc::new(MigrationManager::new(encryption));
        let orchestrator = MeshOrchestrator::new(mesh, migration);

        // Mark US East as Degraded
        orchestrator
            .update_region_health(
                DataRegion::UsEast,
                SemanticHealthReport {
                    component: "mesh".into(),
                    status: HealthStatus::Degraded,
                    timestamp: Utc::now(),
                    carbon_intensity: 100.0,
                    cost_index: 0.5,
                    latency_ms: 100,
                    uptime_secs: 100,
                    message: "Memory leak detected".into(),
                },
            )
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
}
