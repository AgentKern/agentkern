//! Offline Agent Support
//!
//! Offline operation and sync strategies for edge agents

use serde::{Deserialize, Serialize};

#[cfg(feature = "embedded")]
use alloc::{string::String, vec::Vec};

/// Offline agent state.
pub struct OfflineAgent {
    /// Agent ID
    agent_id: String,
    /// Current state
    state: OfflineState,
    /// Pending actions queue
    pending_actions: Vec<PendingAction>,
    /// Sync strategy
    sync_strategy: SyncStrategy,
}

/// Offline state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfflineState {
    /// Connected to cloud
    Online,
    /// Disconnected, operating autonomously
    Offline,
    /// Syncing pending actions
    Syncing,
    /// Sync failed, retry pending
    SyncFailed,
}

/// Sync strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStrategy {
    /// Sync immediately when online
    Immediate,
    /// Batch sync at intervals
    Batched,
    /// Manual sync only
    Manual,
    /// Sync on low battery / shutdown
    OnShutdown,
}

/// Pending action to sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    /// Action ID
    pub id: String,
    /// Action type
    pub action_type: String,
    /// Payload
    pub payload: String,
    /// Timestamp (Unix ms)
    pub timestamp: u64,
    /// Retry count
    pub retries: u32,
}

impl OfflineAgent {
    /// Create new offline agent.
    pub fn new(agent_id: String, strategy: SyncStrategy) -> Self {
        Self {
            agent_id,
            state: OfflineState::Offline,
            pending_actions: Vec::new(),
            sync_strategy: strategy,
        }
    }

    /// Get current state.
    pub fn state(&self) -> OfflineState {
        self.state
    }

    /// Go online.
    pub fn go_online(&mut self) {
        self.state = OfflineState::Online;
    }

    /// Go offline.
    pub fn go_offline(&mut self) {
        self.state = OfflineState::Offline;
    }

    /// Queue action for later sync.
    pub fn queue_action(&mut self, action_type: String, payload: String, timestamp: u64) -> String {
        let id = format!("{}-{}", self.agent_id, self.pending_actions.len());

        self.pending_actions.push(PendingAction {
            id: id.clone(),
            action_type,
            payload,
            timestamp,
            retries: 0,
        });

        id
    }

    /// Get pending actions count.
    pub fn pending_count(&self) -> usize {
        self.pending_actions.len()
    }

    /// Get pending actions.
    pub fn pending_actions(&self) -> &[PendingAction] {
        &self.pending_actions
    }

    /// Clear synced actions.
    pub fn clear_synced(&mut self, ids: &[String]) {
        self.pending_actions.retain(|a| !ids.contains(&a.id));
    }

    /// Should sync now?
    pub fn should_sync(&self) -> bool {
        if self.state != OfflineState::Online {
            return false;
        }

        match self.sync_strategy {
            SyncStrategy::Immediate => !self.pending_actions.is_empty(),
            SyncStrategy::Batched => self.pending_actions.len() >= 10,
            SyncStrategy::Manual => false,
            SyncStrategy::OnShutdown => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_agent_create() {
        let agent = OfflineAgent::new("agent-1".into(), SyncStrategy::Immediate);
        assert_eq!(agent.state(), OfflineState::Offline);
        assert_eq!(agent.pending_count(), 0);
    }

    #[test]
    fn test_queue_action() {
        let mut agent = OfflineAgent::new("agent-1".into(), SyncStrategy::Immediate);

        let id = agent.queue_action("sensor_read".into(), "{}".into(), 12345);

        assert!(!id.is_empty());
        assert_eq!(agent.pending_count(), 1);
    }

    #[test]
    fn test_sync_decision() {
        let mut agent = OfflineAgent::new("agent-1".into(), SyncStrategy::Immediate);

        // Offline, shouldn't sync
        agent.queue_action("test".into(), "{}".into(), 0);
        assert!(!agent.should_sync());

        // Online, should sync
        agent.go_online();
        assert!(agent.should_sync());
    }
}
