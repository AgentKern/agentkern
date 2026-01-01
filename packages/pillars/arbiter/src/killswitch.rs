//! AgentKern-Arbiter: Kill Switch
//!
//! Per EXECUTION_MANDATE.md §6: "Hardware-level Red Button to terminate rogue agent swarms"
//!
//! Features:
//! - Immediate agent termination
//! - Swarm-wide shutdown
//! - Graceful vs forced termination
//! - Audit logging of all kills
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_arbiter::killswitch::{KillSwitch, KillReason};
//!
//! let mut ks = KillSwitch::new();
//! ks.terminate_agent("agent-123", KillReason::PolicyViolation);
//! ks.terminate_swarm("swarm-evil", KillReason::BudgetExceeded);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Reason for agent termination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KillReason {
    /// Policy violation detected
    PolicyViolation,
    /// Agent exceeded budget (tokens, API calls, cost)
    BudgetExceeded,
    /// Prompt injection detected
    PromptInjection,
    /// Rogue behavior detected
    RogueBehavior,
    /// Manual operator termination
    ManualTermination,
    /// System-wide emergency shutdown
    EmergencyShutdown,
    /// Agent exceeded time limit
    TimeoutExceeded,
    /// Parent agent/swarm terminated
    ParentTerminated,
    /// Custom reason
    Custom(String),
}

/// Termination type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminationType {
    /// Graceful shutdown (allow cleanup)
    Graceful,
    /// Forced immediate termination
    Forced,
    /// Hardware-level kill (no cleanup)
    HardwareKill,
}

/// Record of a kill event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillRecord {
    /// Unique kill ID
    pub id: Uuid,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Agent or swarm ID
    pub target_id: String,
    /// Target type
    pub target_type: TargetType,
    /// Kill reason
    pub reason: KillReason,
    /// Termination type
    pub termination_type: TerminationType,
    /// Operator who initiated (if manual)
    pub initiated_by: Option<String>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Type of kill target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType {
    /// Single agent
    Agent,
    /// Agent swarm
    Swarm,
    /// Entire region
    Region,
    /// Global (all agents)
    Global,
}

/// Kill switch for agent termination.
#[derive(Debug)]
pub struct KillSwitch {
    /// Terminated agents (still tracked for audit)
    terminated_agents: Arc<RwLock<HashSet<String>>>,
    /// Terminated swarms
    terminated_swarms: Arc<RwLock<HashSet<String>>>,
    /// Kill history
    history: Arc<RwLock<Vec<KillRecord>>>,
    /// Emergency shutdown flag
    emergency_shutdown: Arc<RwLock<bool>>,
}

impl Default for KillSwitch {
    fn default() -> Self {
        Self::new()
    }
}

impl KillSwitch {
    /// Create a new kill switch.
    pub fn new() -> Self {
        Self {
            terminated_agents: Arc::new(RwLock::new(HashSet::new())),
            terminated_swarms: Arc::new(RwLock::new(HashSet::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            emergency_shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Check if an agent is allowed to operate.
    pub async fn is_agent_alive(&self, agent_id: &str) -> bool {
        // Check emergency shutdown first
        if *self.emergency_shutdown.read().await {
            return false;
        }

        // Check if specifically terminated
        !self.terminated_agents.read().await.contains(agent_id)
    }

    /// Check if a swarm is allowed to operate.
    pub async fn is_swarm_alive(&self, swarm_id: &str) -> bool {
        if *self.emergency_shutdown.read().await {
            return false;
        }
        !self.terminated_swarms.read().await.contains(swarm_id)
    }

    /// Terminate a single agent.
    pub async fn terminate_agent(
        &self,
        agent_id: &str,
        reason: KillReason,
        termination_type: TerminationType,
        initiated_by: Option<String>,
    ) -> KillRecord {
        let record = KillRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            target_id: agent_id.to_string(),
            target_type: TargetType::Agent,
            reason,
            termination_type,
            initiated_by,
            success: true,
            error: None,
        };

        // Add to terminated set
        self.terminated_agents
            .write()
            .await
            .insert(agent_id.to_string());

        // Log the kill
        self.history.write().await.push(record.clone());

        tracing::warn!(
            agent_id = %agent_id,
            reason = ?record.reason,
            termination_type = ?termination_type,
            "Agent terminated"
        );

        record
    }

    /// Terminate an entire swarm (all agents in the swarm).
    pub async fn terminate_swarm(
        &self,
        swarm_id: &str,
        reason: KillReason,
        termination_type: TerminationType,
        initiated_by: Option<String>,
    ) -> KillRecord {
        let record = KillRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            target_id: swarm_id.to_string(),
            target_type: TargetType::Swarm,
            reason,
            termination_type,
            initiated_by,
            success: true,
            error: None,
        };

        self.terminated_swarms
            .write()
            .await
            .insert(swarm_id.to_string());
        self.history.write().await.push(record.clone());

        tracing::error!(
            swarm_id = %swarm_id,
            reason = ?record.reason,
            "Swarm terminated"
        );

        record
    }

    /// EMERGENCY: Terminate all agents globally.
    pub async fn emergency_shutdown(&self, initiated_by: Option<String>) -> KillRecord {
        let record = KillRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            target_id: "GLOBAL".to_string(),
            target_type: TargetType::Global,
            reason: KillReason::EmergencyShutdown,
            termination_type: TerminationType::HardwareKill,
            initiated_by,
            success: true,
            error: None,
        };

        // Set emergency flag
        *self.emergency_shutdown.write().await = true;
        self.history.write().await.push(record.clone());

        tracing::error!("🚨 EMERGENCY SHUTDOWN ACTIVATED - ALL AGENTS TERMINATED");

        record
    }

    /// Lift emergency shutdown (careful!)
    pub async fn lift_emergency(&self) {
        *self.emergency_shutdown.write().await = false;
        tracing::warn!("Emergency shutdown lifted - agents may resume");
    }

    /// Get kill history.
    pub async fn get_history(&self) -> Vec<KillRecord> {
        self.history.read().await.clone()
    }

    /// Get count of terminated agents.
    pub async fn terminated_count(&self) -> usize {
        self.terminated_agents.read().await.len()
    }

    /// Check if system is in emergency shutdown.
    pub async fn is_emergency(&self) -> bool {
        *self.emergency_shutdown.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_termination() {
        let ks = KillSwitch::new();

        assert!(ks.is_agent_alive("agent-1").await);

        ks.terminate_agent(
            "agent-1",
            KillReason::PolicyViolation,
            TerminationType::Graceful,
            None,
        )
        .await;

        assert!(!ks.is_agent_alive("agent-1").await);
        assert!(ks.is_agent_alive("agent-2").await);
    }

    #[tokio::test]
    async fn test_swarm_termination() {
        let ks = KillSwitch::new();

        assert!(ks.is_swarm_alive("swarm-1").await);

        ks.terminate_swarm(
            "swarm-1",
            KillReason::BudgetExceeded,
            TerminationType::Forced,
            Some("operator-123".to_string()),
        )
        .await;

        assert!(!ks.is_swarm_alive("swarm-1").await);
    }

    #[tokio::test]
    async fn test_emergency_shutdown() {
        let ks = KillSwitch::new();

        assert!(ks.is_agent_alive("agent-1").await);
        assert!(!ks.is_emergency().await);

        ks.emergency_shutdown(Some("admin".to_string())).await;

        assert!(ks.is_emergency().await);
        assert!(!ks.is_agent_alive("agent-1").await);
        assert!(!ks.is_agent_alive("any-agent").await);
    }

    #[tokio::test]
    async fn test_kill_history() {
        let ks = KillSwitch::new();

        ks.terminate_agent(
            "a1",
            KillReason::RogueBehavior,
            TerminationType::Forced,
            None,
        )
        .await;
        ks.terminate_agent(
            "a2",
            KillReason::TimeoutExceeded,
            TerminationType::Graceful,
            None,
        )
        .await;

        let history = ks.get_history().await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].target_id, "a1");
        assert_eq!(history[1].target_id, "a2");
    }
}
