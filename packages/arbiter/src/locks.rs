//! AgentKern-Arbiter: Lock Manager
//!
//! Manages business locks with TTL and priority-based preemption.
//!
//! Per ARCHITECTURE.md:
//! - Atomic Business Locks
//! - Priority-based scheduling

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::types::{BusinessLock, LockType};

/// Lock manager for business resources.
pub struct LockManager {
    locks: Arc<RwLock<HashMap<String, BusinessLock>>>,
    default_ttl_seconds: i64,
}

impl Default for LockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            default_ttl_seconds: 30,
        }
    }

    pub fn with_ttl(mut self, seconds: i64) -> Self {
        self.default_ttl_seconds = seconds;
        self
    }

    /// Try to acquire a lock on a resource.
    pub async fn acquire(
        &self,
        agent_id: &str,
        resource: &str,
        priority: i32,
        lock_type: LockType,
        duration_ms: Option<u64>,
    ) -> Result<BusinessLock, LockError> {
        let mut locks = self.locks.write().await;

        // Check if resource is already locked
        if let Some(existing) = locks.get(resource) {
            if !existing.is_expired() {
                // Check if same agent
                if existing.locked_by == agent_id {
                    // Extend the lock
                    let mut lock = existing.clone();
                    lock.expires_at = Utc::now() + Duration::milliseconds(
                        duration_ms.unwrap_or(self.default_ttl_seconds as u64 * 1000) as i64
                    );
                    locks.insert(resource.to_string(), lock.clone());
                    return Ok(lock);
                }

                // Check priority for preemption
                if priority > existing.priority {
                    // Preempt the existing lock
                    tracing::info!(
                        "Agent {} preempting lock on {} from {} (priority {} > {})",
                        agent_id, resource, existing.locked_by, priority, existing.priority
                    );
                } else {
                    return Err(LockError::ResourceLocked {
                        resource: resource.to_string(),
                        locked_by: existing.locked_by.clone(),
                        remaining_seconds: existing.remaining_seconds(),
                    });
                }
            }
        }

        // Create new lock
        let ttl_ms = duration_ms.unwrap_or(self.default_ttl_seconds as u64 * 1000);
        let lock = BusinessLock {
            id: Uuid::new_v4(),
            resource: resource.to_string(),
            locked_by: agent_id.to_string(),
            acquired_at: Utc::now(),
            expires_at: Utc::now() + Duration::milliseconds(ttl_ms as i64),
            priority,
            lock_type,
        };

        locks.insert(resource.to_string(), lock.clone());
        Ok(lock)
    }

    /// Release a lock on a resource.
    pub async fn release(&self, agent_id: &str, resource: &str) -> Result<(), LockError> {
        let mut locks = self.locks.write().await;

        if let Some(lock) = locks.get(resource) {
            if lock.locked_by != agent_id {
                return Err(LockError::NotOwner {
                    resource: resource.to_string(),
                    owner: lock.locked_by.clone(),
                    requester: agent_id.to_string(),
                });
            }
            locks.remove(resource);
            Ok(())
        } else {
            Err(LockError::NotFound {
                resource: resource.to_string(),
            })
        }
    }

    /// Get the current lock status for a resource.
    pub async fn get_status(&self, resource: &str) -> Option<BusinessLock> {
        let locks = self.locks.read().await;
        locks.get(resource).filter(|l| !l.is_expired()).cloned()
    }

    /// Clean up expired locks.
    pub async fn cleanup_expired(&self) -> usize {
        let mut locks = self.locks.write().await;
        let before = locks.len();
        locks.retain(|_, lock| !lock.is_expired());
        before - locks.len()
    }
}

/// Lock operation errors.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("Resource {resource} is locked by {locked_by} for {remaining_seconds}s")]
    ResourceLocked {
        resource: String,
        locked_by: String,
        remaining_seconds: i64,
    },

    #[error("Lock on {resource} not owned by {requester} (owner: {owner})")]
    NotOwner {
        resource: String,
        owner: String,
        requester: String,
    },

    #[error("No lock found for resource {resource}")]
    NotFound {
        resource: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acquire_and_release() {
        let manager = LockManager::new();

        // Acquire
        let lock = manager.acquire("agent-1", "resource-1", 0, LockType::Write, None).await.unwrap();
        assert_eq!(lock.resource, "resource-1");
        assert_eq!(lock.locked_by, "agent-1");

        // Release
        manager.release("agent-1", "resource-1").await.unwrap();
        assert!(manager.get_status("resource-1").await.is_none());
    }

    #[tokio::test]
    async fn test_lock_conflict() {
        let manager = LockManager::new();

        // First agent acquires
        manager.acquire("agent-1", "resource-1", 0, LockType::Write, None).await.unwrap();

        // Second agent tries to acquire (same priority)
        let result = manager.acquire("agent-2", "resource-1", 0, LockType::Write, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_priority_preemption() {
        let manager = LockManager::new();

        // Low priority agent acquires
        manager.acquire("agent-1", "resource-1", 5, LockType::Write, None).await.unwrap();

        // High priority agent preempts
        let lock = manager.acquire("agent-2", "resource-1", 10, LockType::Write, None).await.unwrap();
        assert_eq!(lock.locked_by, "agent-2");
    }

    #[tokio::test]
    async fn test_wrong_owner_release() {
        let manager = LockManager::new();

        manager.acquire("agent-1", "resource-1", 0, LockType::Write, None).await.unwrap();

        let result = manager.release("agent-2", "resource-1").await;
        assert!(result.is_err());
    }
}
