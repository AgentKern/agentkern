//! AgentKern-Synapse: Core Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent state stored in Synapse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Agent identifier
    pub agent_id: String,
    /// Key-value state storage
    pub state: HashMap<String, serde_json::Value>,
    /// Per-key metadata for conflict resolution (Last-Write-Wins)
    #[serde(default)]
    pub state_metadata: HashMap<String, DateTime<Utc>>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Version for CRDT conflict resolution
    pub version: u64,
    /// Vector clock for distributed ordering
    pub vector_clock: HashMap<String, u64>,
}

impl AgentState {
    /// Create a new empty agent state.
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            state: HashMap::new(),
            state_metadata: HashMap::new(),
            updated_at: Utc::now(),
            version: 1,
            vector_clock: HashMap::new(),
        }
    }

    /// Merge with another state (CRDT key-level LWW semantics)
    pub fn merge(&mut self, other: &AgentState) {
        if self.agent_id != other.agent_id {
            tracing::warn!(
                "Attempted to merge states of different agents: {} and {}",
                self.agent_id,
                other.agent_id
            );
            return;
        }

        // Key-level Last Write Wins
        for (key, other_val) in &other.state {
            let other_ts = other
                .state_metadata
                .get(key)
                .cloned()
                .unwrap_or(other.updated_at);

            let mut should_update = false;
            if let Some(local_ts) = self.state_metadata.get(key) {
                if other_ts > *local_ts {
                    should_update = true;
                }
            } else {
                should_update = true;
            }

            if should_update {
                self.state.insert(key.clone(), other_val.clone());
                self.state_metadata.insert(key.clone(), other_ts);
            }
        }

        // Update global metadata
        if other.version > self.version {
            self.version = other.version;
        }
        if other.updated_at > self.updated_at {
            self.updated_at = other.updated_at;
        }

        // Merge vector clocks
        for (node, clock) in &other.vector_clock {
            let entry = self.vector_clock.entry(node.clone()).or_insert(0);
            *entry = (*entry).max(*clock);
        }
    }
}

/// Query for retrieving agent state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateQuery {
    /// Agent ID to query
    pub agent_id: String,
    /// Optional specific keys to retrieve
    pub keys: Option<Vec<String>>,
}

/// Update to agent state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    /// Agent ID to update
    pub agent_id: String,
    /// Key-value pairs to update
    pub updates: HashMap<String, serde_json::Value>,
    /// Optional keys to delete
    pub deletes: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_new() {
        let state = AgentState::new("agent-1");
        assert_eq!(state.agent_id, "agent-1");
        assert_eq!(state.version, 1);
        assert!(state.state.is_empty());
    }

    #[test]
    fn test_agent_state_concurrent_merge() {
        let mut state1 = AgentState::new("agent-1");
        let now = Utc::now();
        state1
            .state
            .insert("key1".to_string(), serde_json::json!("v1"));
        state1.state_metadata.insert("key1".to_string(), now);
        state1.version = 1;

        let mut state2 = AgentState::new("agent-1");
        let later = now + chrono::Duration::seconds(1);
        state2
            .state
            .insert("key2".to_string(), serde_json::json!("v2"));
        state2.state_metadata.insert("key2".to_string(), later);
        state2.version = 2;

        state1.merge(&state2);

        // Both keys should exist (no overwrite of the map)
        assert_eq!(state1.state.get("key1").unwrap(), "v1");
        assert_eq!(state1.state.get("key2").unwrap(), "v2");
        assert_eq!(state1.version, 2);
    }

    #[test]
    fn test_agent_state_lww_conflict() {
        let mut state1 = AgentState::new("agent-1");
        let now = Utc::now();
        state1
            .state
            .insert("key1".to_string(), serde_json::json!("v1"));
        state1.state_metadata.insert("key1".to_string(), now);

        let mut state2 = AgentState::new("agent-1");
        let later = now + chrono::Duration::seconds(1);
        state2
            .state
            .insert("key1".to_string(), serde_json::json!("v2"));
        state2.state_metadata.insert("key1".to_string(), later);

        state1.merge(&state2);

        // Later update should win
        assert_eq!(state1.state.get("key1").unwrap(), "v2");
    }
}
