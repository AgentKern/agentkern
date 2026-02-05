//! AgentKern-Arbiter: Lock Manager
//!
//! Manages business locks with TTL and priority-based preemption.
//!
//! Per ARCHITECTURE.md:
//! - Atomic Business Locks
//! - Priority-based scheduling

use chrono::{DateTime, Duration, Utc};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::types::{BusinessLock, LockType};

/// Internal state for LockManager.
struct LockManagerInner {
    locks: HashMap<String, BusinessLock>,
    expiration_index: BTreeSet<(DateTime<Utc>, String)>,
}

/// Lock manager for business resources.
pub struct LockManager {
    inner: Arc<RwLock<LockManagerInner>>,
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
            inner: Arc::new(RwLock::new(LockManagerInner {
                locks: HashMap::new(),
                expiration_index: BTreeSet::new(),
            })),
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
        let mut inner = self.inner.write().await;

        // Check if resource is already locked
        let existing_opt = inner.locks.get(resource).cloned();
        if let Some(existing) = existing_opt {
            if !existing.is_expired() {
                // Check if same agent
                if existing.locked_by == agent_id {
                    // Extend the lock
                    let mut lock = existing.clone();

                    // Remove old index entry
                    inner
                        .expiration_index
                        .remove(&(existing.expires_at, resource.to_string()));

                    lock.expires_at = Utc::now()
                        + Duration::milliseconds(
                            duration_ms.unwrap_or(self.default_ttl_seconds as u64 * 1000) as i64,
                        );

                    // Add new index entry
                    inner
                        .expiration_index
                        .insert((lock.expires_at, resource.to_string()));

                    inner.locks.insert(resource.to_string(), lock.clone());
                    return Ok(lock);
                }

                // Check priority for preemption
                if priority > existing.priority {
                    // Preempt the existing lock
                    tracing::info!(
                        "Agent {} preempting lock on {} from {} (priority {} > {})",
                        agent_id,
                        resource,
                        existing.locked_by,
                        priority,
                        existing.priority
                    );

                    // Remove old index entry
                    inner
                        .expiration_index
                        .remove(&(existing.expires_at, resource.to_string()));
                } else {
                    return Err(LockError::ResourceLocked {
                        resource: resource.to_string(),
                        locked_by: existing.locked_by.clone(),
                        remaining_seconds: existing.remaining_seconds(),
                    });
                }
            } else {
                // Remove expired but still present lock from index
                inner
                    .expiration_index
                    .remove(&(existing.expires_at, resource.to_string()));
            }
        }

        // Create new lock
        let ttl_ms = duration_ms.unwrap_or(self.default_ttl_seconds as u64 * 1000);
        let now = Utc::now();
        let expires_at = now + Duration::milliseconds(ttl_ms as i64);

        let lock = BusinessLock {
            id: Uuid::new_v4(),
            resource: resource.to_string(),
            locked_by: agent_id.to_string(),
            acquired_at: now,
            expires_at,
            priority,
            lock_type,
        };

        inner.locks.insert(resource.to_string(), lock.clone());
        inner
            .expiration_index
            .insert((expires_at, resource.to_string()));

        Ok(lock)
    }

    /// Release a lock on a resource.
    pub async fn release(&self, agent_id: &str, resource: &str) -> Result<(), LockError> {
        let mut inner = self.inner.write().await;

        let lock_opt = inner.locks.get(resource).cloned();
        if let Some(lock) = lock_opt {
            if lock.locked_by != agent_id {
                return Err(LockError::NotOwner {
                    resource: resource.to_string(),
                    owner: lock.locked_by.clone(),
                    requester: agent_id.to_string(),
                });
            }

            // Remove from index
            inner
                .expiration_index
                .remove(&(lock.expires_at, resource.to_string()));
            inner.locks.remove(resource);
            Ok(())
        } else {
            Err(LockError::NotFound {
                resource: resource.to_string(),
            })
        }
    }

    /// Get the current lock status for a resource.
    pub async fn get_status(&self, resource: &str) -> Option<BusinessLock> {
        let inner = self.inner.read().await;
        inner
            .locks
            .get(resource)
            .filter(|l| !l.is_expired())
            .cloned()
    }

    /// Clean up expired locks using the expiration index.
    pub async fn cleanup_expired(&self) -> usize {
        let mut inner = self.inner.write().await;
        let now = Utc::now();

        let mut expired_keys = Vec::new();

        // Use the index to find all expired items
        for (expires_at, resource) in inner.expiration_index.range(..=(now, String::new())) {
            expired_keys.push((*expires_at, resource.clone()));
        }

        let count = expired_keys.len();

        for entry in expired_keys {
            inner.expiration_index.remove(&entry);
            inner.locks.remove(&entry.1);
        }

        count
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
    NotFound { resource: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_acquire_and_release() {
        let manager = LockManager::new();

        // Acquire
        let lock = manager
            .acquire("agent-1", "resource-1", 0, LockType::Write, None)
            .await
            .unwrap();
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
        manager
            .acquire("agent-1", "resource-1", 0, LockType::Write, None)
            .await
            .unwrap();

        // Second agent tries to acquire (same priority)
        let result = manager
            .acquire("agent-2", "resource-1", 0, LockType::Write, None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_priority_preemption() {
        let manager = LockManager::new();

        // Low priority agent acquires
        manager
            .acquire("agent-1", "resource-1", 5, LockType::Write, None)
            .await
            .unwrap();

        // High priority agent preempts
        let lock = manager
            .acquire("agent-2", "resource-1", 10, LockType::Write, None)
            .await
            .unwrap();
        assert_eq!(lock.locked_by, "agent-2");
    }

    #[tokio::test]
    async fn test_wrong_owner_release() {
        let manager = LockManager::new();

        manager
            .acquire("agent-1", "resource-1", 0, LockType::Write, None)
            .await
            .unwrap();

        let result = manager.release("agent-2", "resource-1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let manager = LockManager::new();

        // Acquire with very short TTL
        manager
            .acquire("agent-1", "resource-1", 0, LockType::Write, Some(10))
            .await
            .unwrap();

        // Acquire with long TTL
        manager
            .acquire("agent-1", "resource-2", 0, LockType::Write, Some(10000))
            .await
            .unwrap();

        // Wait for short TTL to expire
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let cleaned = manager.cleanup_expired().await;
        assert_eq!(cleaned, 1);

        assert!(manager.get_status("resource-1").await.is_none());
        assert!(manager.get_status("resource-2").await.is_some());
    }
}
