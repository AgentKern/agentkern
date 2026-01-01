//! Raft Consensus for Global Lock Manager
//!
//! Per ARCHITECTURE.md Section 3: "The Speed of Light"
//! - **Arbiter (Traffic)**: Raft Consensus for "Atomic Business Locks"
//! - Used ONLY for strong consistency operations (e.g., spending money)
//!
//! This module implements Raft-based distributed locking.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Raft node ID.
pub type NodeId = u64;

/// Raft log entry for lock operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockCommand {
    Acquire {
        resource: String,
        agent_id: String,
        priority: i32,
        ttl_ms: u64,
    },
    Release {
        resource: String,
        agent_id: String,
    },
    Heartbeat {
        resource: String,
        agent_id: String,
    },
}

/// Raft log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: LockCommand,
}

/// Raft node state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

/// Raft cluster configuration.
#[derive(Debug, Clone)]
pub struct RaftConfig {
    pub node_id: NodeId,
    pub peers: Vec<NodeId>,
    pub election_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
}

impl Default for RaftConfig {
    fn default() -> Self {
        Self {
            node_id: 1,
            peers: vec![],
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        }
    }
}

/// Distributed Lock State Machine.
#[derive(Debug, Default)]
pub struct LockStateMachine {
    locks: HashMap<String, LockEntry>,
}

#[derive(Debug, Clone)]
pub struct LockEntry {
    pub agent_id: String,
    pub priority: i32,
    pub acquired_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl LockStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a command to the state machine.
    pub fn apply(&mut self, command: &LockCommand) -> Result<bool, &'static str> {
        match command {
            LockCommand::Acquire {
                resource,
                agent_id,
                priority,
                ttl_ms,
            } => {
                // Check if lock exists and is still valid
                if let Some(existing) = self.locks.get(resource) {
                    if existing.expires_at > chrono::Utc::now() {
                        // Lock exists - check priority for preemption
                        if *priority > existing.priority {
                            // Preempt lower priority lock
                            self.locks.insert(
                                resource.clone(),
                                LockEntry {
                                    agent_id: agent_id.clone(),
                                    priority: *priority,
                                    acquired_at: chrono::Utc::now(),
                                    expires_at: chrono::Utc::now()
                                        + chrono::Duration::milliseconds(*ttl_ms as i64),
                                },
                            );
                            return Ok(true);
                        }
                        return Err("Resource locked by higher priority agent");
                    }
                }

                // Acquire lock
                self.locks.insert(
                    resource.clone(),
                    LockEntry {
                        agent_id: agent_id.clone(),
                        priority: *priority,
                        acquired_at: chrono::Utc::now(),
                        expires_at: chrono::Utc::now()
                            + chrono::Duration::milliseconds(*ttl_ms as i64),
                    },
                );
                Ok(true)
            }
            LockCommand::Release { resource, agent_id } => {
                if let Some(existing) = self.locks.get(resource) {
                    if existing.agent_id == *agent_id {
                        self.locks.remove(resource);
                        return Ok(true);
                    }
                    return Err("Cannot release lock held by another agent");
                }
                Ok(false) // Lock doesn't exist
            }
            LockCommand::Heartbeat { resource, agent_id } => {
                if let Some(existing) = self.locks.get_mut(resource) {
                    if existing.agent_id == *agent_id {
                        existing.expires_at = chrono::Utc::now() + chrono::Duration::seconds(30);
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    /// Get lock status for a resource.
    pub fn get_lock(&self, resource: &str) -> Option<&LockEntry> {
        self.locks
            .get(resource)
            .filter(|e| e.expires_at > chrono::Utc::now())
    }

    /// Clean up expired locks.
    pub fn cleanup_expired(&mut self) {
        let now = chrono::Utc::now();
        self.locks.retain(|_, v| v.expires_at > now);
    }
}

/// Raft-based Global Lock Manager.
pub struct RaftLockManager {
    config: RaftConfig,
    state: RaftState,
    current_term: u64,
    voted_for: Option<NodeId>,
    log: Vec<LogEntry>,
    state_machine: Arc<RwLock<LockStateMachine>>,
    commit_index: u64,
    last_applied: u64,
}

impl RaftLockManager {
    /// Create a new Raft lock manager.
    pub fn new(config: RaftConfig) -> Self {
        Self {
            config,
            state: RaftState::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            state_machine: Arc::new(RwLock::new(LockStateMachine::new())),
            commit_index: 0,
            last_applied: 0,
        }
    }

    /// Get current Raft state.
    pub fn state(&self) -> RaftState {
        self.state
    }

    /// Check if this node is the leader.
    pub fn is_leader(&self) -> bool {
        self.state == RaftState::Leader
    }

    /// Get the state machine.
    pub fn state_machine(&self) -> Arc<RwLock<LockStateMachine>> {
        Arc::clone(&self.state_machine)
    }

    /// Get the current term number.
    pub fn current_term(&self) -> u64 {
        self.current_term
    }

    /// Get which node this node voted for in the current term.
    /// Returns None if no vote has been cast yet.
    pub fn voted_for(&self) -> Option<NodeId> {
        self.voted_for
    }

    /// Propose a command (must be leader).
    pub fn propose(&mut self, command: LockCommand) -> Result<u64, &'static str> {
        if !self.is_leader() {
            return Err("Not the leader");
        }

        let entry = LogEntry {
            term: self.current_term,
            index: self.log.len() as u64 + 1,
            command,
        };

        let index = entry.index;
        self.log.push(entry);

        // In single-node mode, immediately commit
        if self.config.peers.is_empty() {
            self.commit_index = index;
            self.apply_committed();
        }

        Ok(index)
    }

    /// Apply committed entries to state machine.
    fn apply_committed(&mut self) {
        let mut sm = self.state_machine.write();
        while self.last_applied < self.commit_index {
            self.last_applied += 1;
            if let Some(entry) = self.log.get((self.last_applied - 1) as usize) {
                let _ = sm.apply(&entry.command);
            }
        }
    }

    /// Become leader (for single-node or after election).
    pub fn become_leader(&mut self) {
        self.state = RaftState::Leader;
        tracing::info!(node_id = self.config.node_id, "Became Raft leader");
    }

    /// Acquire a lock through Raft consensus.
    pub fn acquire_lock(
        &mut self,
        resource: &str,
        agent_id: &str,
        priority: i32,
        ttl_ms: u64,
    ) -> Result<u64, &'static str> {
        self.propose(LockCommand::Acquire {
            resource: resource.to_string(),
            agent_id: agent_id.to_string(),
            priority,
            ttl_ms,
        })
    }

    /// Release a lock through Raft consensus.
    pub fn release_lock(&mut self, resource: &str, agent_id: &str) -> Result<u64, &'static str> {
        self.propose(LockCommand::Release {
            resource: resource.to_string(),
            agent_id: agent_id.to_string(),
        })
    }
}

impl Default for RaftLockManager {
    fn default() -> Self {
        Self::new(RaftConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_acquire_release() {
        let mut sm = LockStateMachine::new();

        // Acquire lock
        let result = sm.apply(&LockCommand::Acquire {
            resource: "resource-1".to_string(),
            agent_id: "agent-1".to_string(),
            priority: 5,
            ttl_ms: 30000,
        });
        assert!(result.is_ok());

        // Verify lock exists
        let lock = sm.get_lock("resource-1");
        assert!(lock.is_some());
        assert_eq!(lock.unwrap().agent_id, "agent-1");

        // Release lock
        let result = sm.apply(&LockCommand::Release {
            resource: "resource-1".to_string(),
            agent_id: "agent-1".to_string(),
        });
        assert!(result.is_ok());

        // Verify lock is gone
        assert!(sm.get_lock("resource-1").is_none());
    }

    #[test]
    fn test_priority_preemption() {
        let mut sm = LockStateMachine::new();

        // Low priority acquires
        sm.apply(&LockCommand::Acquire {
            resource: "resource-1".to_string(),
            agent_id: "low-priority".to_string(),
            priority: 1,
            ttl_ms: 30000,
        })
        .unwrap();

        // High priority preempts
        sm.apply(&LockCommand::Acquire {
            resource: "resource-1".to_string(),
            agent_id: "high-priority".to_string(),
            priority: 10,
            ttl_ms: 30000,
        })
        .unwrap();

        let lock = sm.get_lock("resource-1").unwrap();
        assert_eq!(lock.agent_id, "high-priority");
    }

    #[test]
    fn test_raft_lock_manager() {
        let mut manager = RaftLockManager::new(RaftConfig::default());
        manager.become_leader();

        // Acquire through Raft
        let index = manager.acquire_lock("db:accounts", "agent-1", 5, 30000);
        assert!(index.is_ok());

        // Check state machine
        let sm = manager.state_machine();
        let lock = sm.read().get_lock("db:accounts").cloned();
        assert!(lock.is_some());
    }
}
