//! Raft Consensus for Global Lock Manager
//!
//! Per ARCHITECTURE.md Section 3: "The Speed of Light"
//! - **Arbiter (Traffic)**: Raft Consensus for "Atomic Business Locks"
//! - Used ONLY for strong consistency operations (e.g., spending money)
//!
//! This module implements Raft-based distributed locking using OpenRaft.

use openraft::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::storage::{SledStore, TypeConfig};

pub use openraft::Config as RaftConfig;
pub use openraft::RaftState;
pub use openraft::ServerState;

/// Raft node ID.
pub type NodeId = u64;

/// Raft log entry for lock operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LockCommand {
    Acquire {
        resource: String,
        agent_id: String,
        priority: i32,
        ttl_ms: u64,
        /// Timestamp provided by the leader for deterministic expiration
        timestamp_ms: i64,
    },
    Release {
        resource: String,
        agent_id: String,
    },
    Heartbeat {
        resource: String,
        agent_id: String,
        /// Timestamp provided by the leader
        timestamp_ms: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub agent_id: String,
    pub priority: i32,
    pub acquired_at_ms: i64,
    pub expires_at_ms: i64,
}

/// Distributed Lock State Machine (Business Logic)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LockStateMachine {
    locks: HashMap<String, LockEntry>,
}

impl LockStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a command to the state machine using a deterministic timestamp.
    pub fn apply(&mut self, command: &LockCommand) -> Result<bool, &'static str> {
        match command {
            LockCommand::Acquire {
                resource,
                agent_id,
                priority,
                ttl_ms,
                timestamp_ms,
            } => {
                // Check if lock exists and is still valid using deterministic timestamp
                if let Some(existing) = self.locks.get(resource) {
                    if existing.expires_at_ms > *timestamp_ms {
                        // Lock exists - check priority for preemption
                        if *priority > existing.priority {
                            // Preempt lower priority lock
                            self.locks.insert(
                                resource.clone(),
                                LockEntry {
                                    agent_id: agent_id.clone(),
                                    priority: *priority,
                                    acquired_at_ms: *timestamp_ms,
                                    expires_at_ms: *timestamp_ms + *ttl_ms as i64,
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
                        acquired_at_ms: *timestamp_ms,
                        expires_at_ms: *timestamp_ms + *ttl_ms as i64,
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
            LockCommand::Heartbeat {
                resource,
                agent_id,
                timestamp_ms,
            } => {
                if let Some(existing) = self.locks.get_mut(resource) {
                    if existing.agent_id == *agent_id {
                        // Deterministic heartbeat expiration (e.g., extend by 30s)
                        existing.expires_at_ms = *timestamp_ms + 30_000;
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    pub fn get_lock(&self, resource: &str, current_timestamp_ms: i64) -> Option<&LockEntry> {
        self.locks
            .get(resource)
            .filter(|e| e.expires_at_ms > current_timestamp_ms)
    }
}

pub type RaftType = openraft::Raft<TypeConfig>;
pub type Network = crate::network::Network;

pub struct RaftLockManager {
    pub raft: RaftType,
    pub store: SledStore,
    pub network: Network,
}

impl RaftLockManager {
    pub async fn new(node_id: NodeId, _addr: String, path: String) -> Self {
        let config = Config {
            heartbeat_interval: 100,
            election_timeout_min: 200,
            election_timeout_max: 300,
            ..Default::default()
        };

        let config = Arc::new(config);
        let store = SledStore::new(path);
        let network = Network::new();

        let raft = RaftType::new(
            node_id,
            config,
            network.clone(),
            store.clone(),
            store.clone(),
        )
        .await
        .unwrap();

        Self {
            raft,
            store,
            network,
        }
    }

    /// Acquire a lock via Raft consensus.
    pub async fn acquire_lock(
        &self,
        agent_id: &str,
        resource: &str,
        priority: i32,
        ttl_ms: u64,
    ) -> Result<bool, String> {
        let cmd = LockCommand::Acquire {
            resource: resource.to_string(),
            agent_id: agent_id.to_string(),
            priority,
            ttl_ms,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        };
        self.raft
            .client_write(cmd)
            .await
            .map(|resp| resp.data)
            .map_err(|e| e.to_string())
    }

    /// Release a lock via Raft consensus.
    pub async fn release_lock(&self, agent_id: &str, resource: &str) -> Result<bool, String> {
        let cmd = LockCommand::Release {
            resource: resource.to_string(),
            agent_id: agent_id.to_string(),
        };
        self.raft
            .client_write(cmd)
            .await
            .map(|resp| resp.data)
            .map_err(|e| e.to_string())
    }

    /// Update lock expiration via Raft consensus (heartbeat).
    pub async fn heartbeat(&self, agent_id: &str, resource: &str) -> Result<bool, String> {
        let cmd = LockCommand::Heartbeat {
            resource: resource.to_string(),
            agent_id: agent_id.to_string(),
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        };
        self.raft
            .client_write(cmd)
            .await
            .map(|resp| resp.data)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_raft_initialization_with_sled() {
        let temp_dir = std::env::temp_dir().join("raft_test_arbiter");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.to_str().unwrap().to_string();

        // Initialize Raft node 1
        let manager = RaftLockManager::new(1, "127.0.0.1:9000".to_string(), path.clone()).await;

        // Initialize single-node cluster
        let nodes = std::collections::BTreeMap::from([(1, ())]);
        let _ = manager.raft.initialize(nodes).await;

        // Give it a moment to elect leader
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Assert leader state or successful init
        let metrics = manager.raft.metrics().borrow().clone();
        assert!(metrics.current_term >= 1);

        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
