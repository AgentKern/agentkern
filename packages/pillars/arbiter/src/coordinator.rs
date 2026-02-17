//! AgentKern-Arbiter: Coordinator
//!
//! High-level coordination API combining locks and queues.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::consensus::ConsensusEngine;
use crate::cost::CostTracker;
use crate::escalation::{EscalationConnector, WebhookNotifier};
use crate::locks::{LockError, LockManager};
use crate::queue::PriorityQueue;
use crate::types::{BusinessLock, CoordinationRequest, CoordinationResult, LockType};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

use agentkern_gate::NeuroSymbolicValidator;
use agentkern_pulse::{HealthStatus, Pulse, SemanticHealthReport};
use agentkern_synapse::drift::DriftDetector;
use agentkern_synapse::intent::IntentPath;

/// The Arbiter Coordinator.
pub struct Coordinator {
    lock_manager: LockManager,
    queue: Arc<RwLock<PriorityQueue>>,
    _avg_lock_duration_ms: u64,
    cost_tracker: Arc<CostTracker>,
    /// Neuro-symbolic validator for intent-based safety
    validator: Arc<NeuroSymbolicValidator>,
    /// Drift detector for behavioral tracking
    drift_detector: Arc<DriftDetector>,
    /// Active intent paths for agents
    intent_paths: Arc<RwLock<HashMap<String, IntentPath>>>,
    /// Consensus engine for multi-agent governance
    consensus: Arc<ConsensusEngine>,
    /// Optional Raft Manager for distributed consistency
    raft_manager: Option<Arc<crate::RaftLockManager>>,
    /// Escalation notifier
    notifier: Arc<RwLock<WebhookNotifier>>,
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
            _avg_lock_duration_ms: 5000,
            cost_tracker: Arc::new(CostTracker::new()),
            validator: Arc::new(match NeuroSymbolicValidator::new() {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to initialize NeuroSymbolicValidator: {}", e);
                    std::process::exit(1);
                }
            }),
            drift_detector: Arc::new(DriftDetector::new()),
            intent_paths: Arc::new(RwLock::new(HashMap::new())),
            consensus: Arc::new(ConsensusEngine::new()),
            raft_manager: None,
            notifier: Arc::new(RwLock::new(WebhookNotifier::new())),
        }
    }

    /// Register an escalation connector.
    pub async fn add_escalation_connector(&self, connector: Arc<dyn EscalationConnector>) {
        let mut notifier = self.notifier.write().await;
        notifier.add_connector(connector);
    }

    pub fn with_raft(mut self, raft: Arc<crate::RaftLockManager>) -> Self {
        self.raft_manager = Some(raft);
        self
    }

    /// Request coordination for a resource.
    pub async fn request(&self, request: CoordinationRequest) -> CoordinationResult {
        // 1. Cost/Budget Check
        let current_cost = self.cost_tracker.get_agent_total(&request.agent_id);
        let budget_override = self.consensus.get_budget_override(&request.agent_id).await;
        let total_budget = Decimal::from(10) + budget_override;

        if current_cost >= total_budget {
            return CoordinationResult::denied(format!(
                "FinOps: Agent budget exceeded (${})",
                total_budget
            ));
        }

        // 2. Intent Drift Check
        let paths = self.intent_paths.read().await;
        if let Some(path) = paths.get(&request.agent_id) {
            let drift = self.drift_detector.check(path);
            if drift.drifted && drift.score > 70 {
                return CoordinationResult::denied(format!(
                    "Sovereign Governance: Critical intent drift ({})",
                    drift.score
                ));
            }
        }

        // 3. Semantic Intent Check
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

        // Try to acquire lock
        if let Some(raft) = &self.raft_manager {
            match raft
                .acquire_lock(
                    &request.agent_id,
                    &request.resource,
                    request.priority,
                    30000,
                )
                .await
            {
                Ok(true) => {
                    let lock = BusinessLock {
                        id: uuid::Uuid::new_v4(),
                        resource: request.resource.clone(),
                        locked_by: request.agent_id.clone(),
                        acquired_at: Utc::now(),
                        expires_at: Utc::now() + chrono::Duration::seconds(30),
                        priority: request.priority,
                        lock_type: request.operation,
                    };
                    return CoordinationResult::granted(lock);
                }
                Ok(false) => {
                    let mut queue = self.queue.write().await;
                    let position = queue.enqueue(request.clone()) as u32;
                    let wait_ms = queue.estimate_wait_ms(position as usize, 5000);
                    return CoordinationResult::queued(position, wait_ms);
                }
                Err(e) => {
                    return CoordinationResult::denied(format!("Raft consensus error: {}", e));
                }
            }
        }

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
                let wait_ms = queue.estimate_wait_ms(position as usize, 5000);
                CoordinationResult::queued(position, wait_ms)
            }
            Err(e) => CoordinationResult::denied(e.to_string()),
        }
    }

    /// Acquire a lock directly (bypass queue).
    pub async fn acquire_lock(
        &self,
        agent_id: &str,
        resource: &str,
        priority: i32,
    ) -> Result<BusinessLock, String> {
        if let Some(raft) = &self.raft_manager {
            match raft.acquire_lock(agent_id, resource, priority, 30000).await {
                Ok(true) => {
                    return Ok(BusinessLock {
                        id: uuid::Uuid::new_v4(),
                        resource: resource.to_string(),
                        locked_by: agent_id.to_string(),
                        acquired_at: Utc::now(),
                        expires_at: Utc::now() + chrono::Duration::seconds(30),
                        priority,
                        lock_type: LockType::Write,
                    });
                }
                Ok(false) => return Err("Lock conflict via Raft consensus".into()),
                Err(e) => return Err(format!("Raft consensus error: {}", e)),
            }
        }

        self.lock_manager
            .acquire(agent_id, resource, priority, LockType::Write, None)
            .await
            .map_err(|e| e.to_string())
    }

    /// Release a lock and grant to next in queue if any.
    pub async fn release_lock(&self, agent_id: &str, resource: &str) -> Result<(), String> {
        if let Some(raft) = &self.raft_manager {
            match raft.release_lock(agent_id, resource).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => return Err(format!("Raft consensus error: {}", e)),
            }
        }

        self.lock_manager
            .release(agent_id, resource)
            .await
            .map_err(|e| e.to_string())?;

        let mut queue = self.queue.write().await;
        if let Some(next_request) = queue.pop(resource) {
            drop(queue);
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
        let cost_total = self.cost_tracker.get_global_summary().total_usd;
        let cost_index = (cost_total / Decimal::from(1000))
            .to_f64()
            .unwrap_or(0.0)
            .min(1.0);

        SemanticHealthReport {
            component: "Arbiter::Coordinator".to_string(),
            status: HealthStatus::Healthy,
            timestamp: Utc::now(),
            carbon_intensity: 0.0,
            cost_index,
            latency_ms: 0,
            uptime_secs: 0,
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
}
