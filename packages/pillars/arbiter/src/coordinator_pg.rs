//! AgentKern-Arbiter: Postgres-Backed Coordinator
//!
//! Per Phase 16 Plan: Replaces in-memory Coordinator with Postgres persistence.

use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;

use crate::antifragile::AntifragileEngine;
use crate::carbon::CarbonScheduler;
use crate::consensus::ConsensusEngine;
use crate::cost::CostTracker;
use crate::locks_pg::{PgLockManager, LockError};
use crate::queue_pg::PgQueue;
use crate::types::{BusinessLock, CoordinationRequest, CoordinationResult, LockType};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

use agentkern_gate::NeuroSymbolicValidator;
use agentkern_pulse::{HealthStatus, Pulse, PulseManager, SemanticHealthReport};
use agentkern_synapse::drift::DriftDetector;
use agentkern_synapse::intent::IntentPath;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Postgres-backed Arbiter Coordinator.
pub struct PgCoordinator {
    lock_manager: PgLockManager,
    queue: PgQueue,
    avg_lock_duration_ms: u64,
    antifragile: Arc<AntifragileEngine>,
    cost_tracker: Arc<CostTracker>,
    carbon_scheduler: Arc<CarbonScheduler>,
    validator: Arc<NeuroSymbolicValidator>,
    drift_detector: Arc<DriftDetector>,
    intent_paths: Arc<RwLock<HashMap<String, IntentPath>>>,
    pulse: PulseManager,
    consensus: Arc<ConsensusEngine>,
}

impl PgCoordinator {
    pub fn new(pool: PgPool) -> Self {
        Self {
            lock_manager: PgLockManager::new(pool.clone()),
            queue: PgQueue::new(pool),
            avg_lock_duration_ms: 5000,
            antifragile: Arc::new(AntifragileEngine::new()),
            cost_tracker: Arc::new(CostTracker::new()),
            carbon_scheduler: Arc::new(CarbonScheduler::new()),
            validator: Arc::new(
                NeuroSymbolicValidator::new().expect("Failed to load NeuroSymbolicValidator"),
            ),
            drift_detector: Arc::new(DriftDetector::new()),
            intent_paths: Arc::new(RwLock::new(HashMap::new())),
            pulse: PulseManager::new(),
            consensus: Arc::new(ConsensusEngine::new()),
        }
    }

    /// Request coordination for a resource.
    pub async fn request(&self, request: CoordinationRequest) -> CoordinationResult {
        // 1. Antifragile Check (Circuit Breaker)
        if !self
            .antifragile
            .is_service_available(&request.resource)
            .await
        {
            return CoordinationResult::denied(String::from(
                "Antifragile: Service circuit breaker is OPEN",
            ));
        }

        // 2. Cost/Budget Check
        let current_cost = self.cost_tracker.get_agent_total(&request.agent_id);
        let budget_override = self.consensus.get_budget_override(&request.agent_id).await;
        let total_budget = Decimal::from(10) + budget_override;

        if current_cost >= total_budget {
            return CoordinationResult::denied(format!(
                "FinOps: Agent budget exceeded (${})",
                total_budget
            ));
        }

        // 3. Carbon/Sustainability Check
        if let Some(intensity) = self
            .carbon_scheduler
            .get_current_intensity("us-east-1")
            .await
        {
            self.pulse.report_carbon(intensity as f64);

            if intensity > 500.0 && request.priority < 50 {
                return CoordinationResult::denied(String::from(
                    "Sustainability: High grid carbon intensity, task deferred",
                ));
            }
        }

        // 4. Intent Drift Check
        let paths = self.intent_paths.read().await;
        if let Some(path) = paths.get(&request.agent_id) {
            let drift = self.drift_detector.check(path);
            if drift.drifted && drift.score > 70 {
                let failure = crate::antifragile::Failure::new(
                    &request.resource,
                    "Critical intent drift detected",
                );
                self.antifragile.handle_failure(failure).await;
                return CoordinationResult::denied(format!(
                    "Sovereign Governance: Critical intent drift ({})",
                    drift.score
                ));
            }
        }

        // 5. Semantic Intent Check
        let intent_text = request
            .intent
            .clone()
            .unwrap_or_else(|| format!("{:?} on {}", request.operation, request.resource));

        if let Ok(validation) = self.validator.validate(&intent_text) {
            if !validation.allowed {
                if self
                    .consensus
                    .is_security_override_active(&request.resource)
                    .await
                {
                    tracing::info!(resource = %request.resource, "Security gate overridden by Multi-Agent Consensus");
                } else {
                    return CoordinationResult::denied(format!(
                        "Sovereign Security: Action blocked ({})",
                        validation.reason
                    ));
                }
            }
        }

        // Try to acquire lock (PERSISTENT via Postgres)
        match self
            .lock_manager
            .acquire(
                &request.agent_id,
                &request.resource,
                request.priority,
                request.operation,
                Some(request.expected_duration_ms),
            )
            .await
        {
            Ok(lock) => {
                // Remove from queue if present
                let _ = self.queue.dequeue(&request.agent_id, &request.resource).await;
                CoordinationResult::granted(lock)
            }
            Err(LockError::ResourceLocked { .. }) => {
                // Enqueue in PERSISTENT queue
                match self.queue.enqueue(request.clone()).await {
                    Ok(position) => {
                        let wait_ms = self.queue.estimate_wait_ms(position, self.avg_lock_duration_ms);
                        CoordinationResult::queued(position as u32, wait_ms)
                    }
                    Err(e) => {
                        CoordinationResult::denied(format!("Queue error: {}", e))
                    }
                }
            }
            Err(e) => {
                let failure = crate::antifragile::Failure::new(
                    &request.resource,
                    format!("Lock failed: {}", e),
                );
                self.antifragile.handle_failure(failure).await;
                CoordinationResult::denied(e.to_string())
            }
        }
    }

    /// Acquire a lock directly.
    pub async fn acquire_lock(
        &self,
        agent_id: &str,
        resource: &str,
        priority: i32,
    ) -> Result<BusinessLock, String> {
        self.lock_manager
            .acquire(agent_id, resource, priority, LockType::Write, None)
            .await
            .map_err(|e| e.to_string())
    }

    /// Release a lock and grant to next in queue if any.
    pub async fn release_lock(&self, agent_id: &str, resource: &str) -> Result<(), String> {
        self.lock_manager
            .release(agent_id, resource)
            .await
            .map_err(|e| e.to_string())?;

        // Check queue for next waiter
        if let Some(next_request) = self.queue.pop(resource).await {
            let _ = self
                .lock_manager
                .acquire(
                    &next_request.agent_id,
                    resource,
                    next_request.priority,
                    next_request.operation,
                    Some(next_request.expected_duration_ms),
                )
                .await;
        }

        Ok(())
    }

    /// Get the status of a lock.
    pub async fn get_lock_status(&self, resource: &str) -> Option<BusinessLock> {
        self.lock_manager.get_status(resource).await
    }

    /// Register an intent path for an agent.
    pub async fn register_intent(&self, path: IntentPath) {
        let mut paths = self.intent_paths.write().await;
        paths.insert(path.agent_id.clone(), path);
    }

    /// Access the consensus engine.
    pub fn consensus(&self) -> Arc<ConsensusEngine> {
        self.consensus.clone()
    }
}

#[async_trait::async_trait]
impl Pulse for PgCoordinator {
    async fn get_health(&self) -> SemanticHealthReport {
        let intensity = self
            .carbon_scheduler
            .get_current_intensity("us-east-1")
            .await
            .unwrap_or(0.0) as f64;

        let cost_total = self.cost_tracker.get_global_summary().total_usd;
        let cost_index = (cost_total / Decimal::from(1000))
            .to_f64()
            .unwrap_or(0.0)
            .min(1.0);

        SemanticHealthReport {
            component: "Arbiter::PgCoordinator".to_string(),
            status: if intensity > 500.0 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            },
            timestamp: Utc::now(),
            carbon_intensity: intensity,
            cost_index,
            latency_ms: 10, // Slightly higher due to DB
            uptime_secs: 3600,
            message: "Persistent Distributed Coordination Engine active".to_string(),
        }
    }
}
