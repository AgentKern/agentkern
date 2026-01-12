//! AgentKern-Arbiter: Coordinator
//!
//! High-level coordination API combining locks and queues.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::antifragile::AntifragileEngine;
use crate::carbon::CarbonScheduler;
use crate::consensus::ConsensusEngine;
use crate::cost::CostTracker;
use crate::locks::{LockError, LockManager};
use crate::queue::PriorityQueue;
use crate::types::{BusinessLock, CoordinationRequest, CoordinationResult, LockType};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

use agentkern_gate::NeuroSymbolicValidator;
use agentkern_pulse::{HealthStatus, Pulse, PulseManager, SemanticHealthReport};
use agentkern_synapse::drift::DriftDetector;
use agentkern_synapse::intent::IntentPath;

/// The Arbiter Coordinator.
pub struct Coordinator {
    lock_manager: LockManager,
    queue: Arc<RwLock<PriorityQueue>>,
    avg_lock_duration_ms: u64,
    antifragile: Arc<AntifragileEngine>,
    cost_tracker: Arc<CostTracker>,
    /// Carbon scheduler for sustainability
    carbon_scheduler: Arc<CarbonScheduler>,
    /// Neuro-symbolic validator for intent-based safety
    validator: Arc<NeuroSymbolicValidator>,
    /// Drift detector for behavioral tracking
    drift_detector: Arc<DriftDetector>,
    /// Active intent paths for agents
    intent_paths: Arc<RwLock<HashMap<String, IntentPath>>>,
    /// Pulse manager for observability
    pulse: PulseManager,
    /// Consensus engine for multi-agent governance
    consensus: Arc<ConsensusEngine>,
}

impl Default for Coordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Coordinator {
    pub fn new() -> Self {
        Self {
            lock_manager: LockManager::new(),
            queue: Arc::new(RwLock::new(PriorityQueue::new())),
            avg_lock_duration_ms: 5000, // 5 seconds default
            antifragile: Arc::new(AntifragileEngine::new()),
            cost_tracker: Arc::new(CostTracker::new()),
            carbon_scheduler: Arc::new(CarbonScheduler::new()),
            // Initializing complex components.
            // In production, these might be injected or loaded from config.
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
            // Report to Pulse
            self.pulse.report_carbon(intensity as f64);

            if intensity > 500.0 && request.priority < 50 {
                return CoordinationResult::denied(String::from(
                    "Sustainability: High grid carbon intensity, task deferred",
                ));
            }
        }

        // 4. Intent Drift Check (Synapse Integration)
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

        // 5. Semantic Intent Check (Gate Integration)
        let intent_text = request
            .intent
            .clone()
            .unwrap_or_else(|| format!("{:?} on {}", request.operation, request.resource));

        if let Ok(validation) = self.validator.validate(&intent_text) {
            if !validation.allowed {
                // Check for Consensus Override
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

        // Try to acquire lock
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
                let mut queue = self.queue.write().await;
                queue.dequeue(&request.agent_id, &request.resource);
                CoordinationResult::granted(lock)
            }
            Err(LockError::ResourceLocked { .. }) => {
                let mut queue = self.queue.write().await;
                let position = queue.enqueue(request.clone()) as u32;
                let wait_ms = queue.estimate_wait_ms(position as usize, self.avg_lock_duration_ms);
                CoordinationResult::queued(position, wait_ms)
            }
            Err(e) => {
                // Record failure in antifragile engine
                let failure = crate::antifragile::Failure::new(
                    &request.resource,
                    format!("Lock failed: {}", e),
                );
                self.antifragile.handle_failure(failure).await;
                CoordinationResult::denied(e.to_string())
            }
        }
    }

    /// Acquire a lock directly (bypass queue).
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
        let mut queue = self.queue.write().await;
        if let Some(next_request) = queue.pop(resource) {
            drop(queue); // Release lock before recursive call

            // Auto-grant to next in queue
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

    /// Get queue position for an agent.
    pub async fn get_queue_position(&self, agent_id: &str, resource: &str) -> Option<usize> {
        let queue = self.queue.read().await;
        queue.get_position(agent_id, resource)
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
impl Pulse for Coordinator {
    async fn get_health(&self) -> SemanticHealthReport {
        let intensity = self
            .carbon_scheduler
            .get_current_intensity("us-east-1")
            .await
            .unwrap_or(0.0) as f64;

        // Compute cost index (simplified)
        let cost_total = self.cost_tracker.get_global_summary().total_usd;
        let cost_index = (cost_total / Decimal::from(1000))
            .to_f64()
            .unwrap_or(0.0)
            .min(1.0);

        SemanticHealthReport {
            component: "Arbiter::Coordinator".to_string(),
            status: if intensity > 500.0 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            },
            timestamp: Utc::now(),
            carbon_intensity: intensity,
            cost_index,
            latency_ms: 5,     // Mock latency
            uptime_secs: 3600, // Mock uptime
            message: "Autonomous Coordination Engine active".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_request_granted() {
        let coord = Coordinator::new();

        let request = CoordinationRequest::new("agent-1", "resource-1");
        let result = coord.request(request).await;

        assert!(result.granted);
        assert!(result.lock.is_some());
    }

    #[tokio::test]
    async fn test_coordinator_request_queued() {
        let coord = Coordinator::new();

        // First request gets lock
        let req1 = CoordinationRequest::new("agent-1", "resource-1");
        let result1 = coord.request(req1).await;
        assert!(result1.granted);

        // Second request gets queued
        let req2 = CoordinationRequest::new("agent-2", "resource-1");
        let result2 = coord.request(req2).await;
        assert!(!result2.granted);
        assert_eq!(result2.queue_position, Some(1));
    }

    #[tokio::test]
    async fn test_coordinator_release_grants_next() {
        let coord = Coordinator::new();

        // First agent gets lock
        let req1 = CoordinationRequest::new("agent-1", "resource-1");
        coord.request(req1).await;

        // Second agent queued
        let req2 = CoordinationRequest::new("agent-2", "resource-1");
        let result2 = coord.request(req2).await;
        assert!(!result2.granted);

        // First agent releases
        coord.release_lock("agent-1", "resource-1").await.unwrap();

        // Second agent should now have the lock
        let status = coord.get_lock_status("resource-1").await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().locked_by, "agent-2");
    }
}
