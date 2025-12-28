//! AgentKern-Synapse: State Store
//!
//! In-memory state storage with CRDT-like merge semantics.
//!
//! Per ARCHITECTURE.md:
//! - Uses CRDTs (LWW-Register) for eventual consistency
//! - Supports distributed sync via vector clocks

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::drift::{DriftDetector, DriftResult};
use crate::intent::IntentPath;
use crate::types::{AgentState, StateQuery, StateUpdate};

/// The Synapse state store.
pub struct StateStore {
    /// Agent states
    states: Arc<RwLock<HashMap<String, AgentState>>>,
    /// Intent paths
    intents: Arc<RwLock<HashMap<String, IntentPath>>>,
    /// Drift detector
    drift_detector: DriftDetector,
    /// Node ID for vector clocks
    node_id: String,
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StateStore {
    /// Create a new state store.
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            intents: Arc::new(RwLock::new(HashMap::new())),
            drift_detector: DriftDetector::new(),
            node_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Set the node ID for distributed operations.
    pub fn with_node_id(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = node_id.into();
        self
    }

    // =========================================================================
    // State Operations
    // =========================================================================

    /// Get the state for an agent.
    pub async fn get_state(&self, agent_id: &str) -> Option<AgentState> {
        let states = self.states.read().await;
        states.get(agent_id).cloned()
    }

    /// Update the state for an agent.
    pub async fn update_state(&self, update: StateUpdate) -> AgentState {
        let mut states = self.states.write().await;

        let state = states
            .entry(update.agent_id.clone())
            .or_insert_with(|| AgentState::new(&update.agent_id));

        // Apply updates
        for (key, value) in update.updates {
            state.state.insert(key, value);
        }

        // Apply deletes
        if let Some(keys) = update.deletes {
            for key in keys {
                state.state.remove(&key);
            }
        }

        // Increment version and update clock
        state.version += 1;
        state.updated_at = Utc::now();
        let clock = state.vector_clock.entry(self.node_id.clone()).or_insert(0);
        *clock += 1;

        state.clone()
    }

    /// Merge remote state (for distributed sync).
    pub async fn merge_state(&self, remote: AgentState) {
        let mut states = self.states.write().await;

        let local = states
            .entry(remote.agent_id.clone())
            .or_insert_with(|| AgentState::new(&remote.agent_id));

        local.merge(&remote);
    }

    // =========================================================================
    // Intent Operations
    // =========================================================================

    /// Start a new intent path.
    pub async fn start_intent(
        &self,
        agent_id: impl Into<String>,
        intent: impl Into<String>,
        expected_steps: u32,
    ) -> IntentPath {
        let path = IntentPath::new(agent_id, intent, expected_steps);
        let mut intents = self.intents.write().await;
        intents.insert(path.agent_id.clone(), path.clone());
        path
    }

    /// Get the current intent path for an agent.
    pub async fn get_intent(&self, agent_id: &str) -> Option<IntentPath> {
        let intents = self.intents.read().await;
        intents.get(agent_id).cloned()
    }

    /// Record a step in the intent path.
    pub async fn record_step(
        &self,
        agent_id: &str,
        action: impl Into<String>,
        result: Option<String>,
    ) -> Option<IntentPath> {
        let mut intents = self.intents.write().await;

        if let Some(path) = intents.get_mut(agent_id) {
            path.record_step(action, result);

            // Check for drift
            let drift_result = self.drift_detector.check(path);
            path.drift_detected = drift_result.drifted;
            path.drift_score = drift_result.score;

            Some(path.clone())
        } else {
            None
        }
    }

    /// Check for intent drift.
    pub async fn check_drift(&self, agent_id: &str) -> Option<DriftResult> {
        let intents = self.intents.read().await;
        intents
            .get(agent_id)
            .map(|path| self.drift_detector.check(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_store_crud() {
        let store = StateStore::new();

        // Create
        let update = StateUpdate {
            agent_id: "agent-1".to_string(),
            updates: [("key1".to_string(), serde_json::json!("value1"))].into(),
            deletes: None,
        };
        let state = store.update_state(update).await;
        assert_eq!(state.agent_id, "agent-1");
        assert_eq!(state.state.get("key1").unwrap(), "value1");

        // Read
        let retrieved = store.get_state("agent-1").await.unwrap();
        assert_eq!(retrieved.state.get("key1").unwrap(), "value1");

        // Update
        let update2 = StateUpdate {
            agent_id: "agent-1".to_string(),
            updates: [("key2".to_string(), serde_json::json!("value2"))].into(),
            deletes: None,
        };
        let state2 = store.update_state(update2).await;
        assert_eq!(state2.state.get("key1").unwrap(), "value1");
        assert_eq!(state2.state.get("key2").unwrap(), "value2");

        // Delete
        let update3 = StateUpdate {
            agent_id: "agent-1".to_string(),
            updates: HashMap::new(),
            deletes: Some(vec!["key1".to_string()]),
        };
        let state3 = store.update_state(update3).await;
        assert!(state3.state.get("key1").is_none());
        assert_eq!(state3.state.get("key2").unwrap(), "value2");
    }

    #[tokio::test]
    async fn test_intent_tracking() {
        let store = StateStore::new();

        // Start intent
        let path = store.start_intent("agent-1", "Process order", 3).await;
        assert_eq!(path.original_intent, "Process order");
        assert_eq!(path.current_step, 0);

        // Record steps
        store
            .record_step("agent-1", "validate", Some("ok".to_string()))
            .await;
        store
            .record_step("agent-1", "process", Some("ok".to_string()))
            .await;

        let path = store.get_intent("agent-1").await.unwrap();
        assert_eq!(path.current_step, 2);
        assert_eq!(path.history.len(), 2);
    }

    #[tokio::test]
    async fn test_drift_detection() {
        let store = StateStore::new();

        store.start_intent("agent-1", "Simple task", 2).await;
        store.record_step("agent-1", "step1", None).await;
        store.record_step("agent-1", "step2", None).await;
        store.record_step("agent-1", "step3", None).await;
        store.record_step("agent-1", "step4", None).await; // Overrun

        let drift = store.check_drift("agent-1").await.unwrap();
        assert!(drift.score > 0);
    }
}
