//! VeriMantle-Synapse: Core Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Agent state stored in Synapse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Agent identifier
    pub agent_id: String,
    /// Key-value state storage
    pub state: HashMap<String, serde_json::Value>,
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
            updated_at: Utc::now(),
            version: 1,
            vector_clock: HashMap::new(),
        }
    }

    /// Merge with another state (CRDT LWW-Register semantics)
    pub fn merge(&mut self, other: &AgentState) {
        // Last Write Wins based on version
        if other.version > self.version {
            self.state = other.state.clone();
            self.version = other.version;
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
    fn test_agent_state_merge_lww() {
        let mut state1 = AgentState::new("agent-1");
        state1.state.insert("key1".to_string(), serde_json::json!("value1"));
        state1.version = 1;

        let mut state2 = AgentState::new("agent-1");
        state2.state.insert("key1".to_string(), serde_json::json!("value2"));
        state2.version = 2;

        state1.merge(&state2);
        
        // state2 wins because higher version
        assert_eq!(state1.state.get("key1").unwrap(), "value2");
        assert_eq!(state1.version, 2);
    }
}
