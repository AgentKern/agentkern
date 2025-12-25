//! VeriMantle-Arbiter: Core Types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A business lock on a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessLock {
    /// Lock ID
    pub id: Uuid,
    /// Resource being locked (e.g., "customer:12345", "database:accounts")
    pub resource: String,
    /// Agent holding the lock
    pub locked_by: String,
    /// When the lock was acquired
    pub acquired_at: DateTime<Utc>,
    /// When the lock expires
    pub expires_at: DateTime<Utc>,
    /// Lock priority (higher = more important)
    pub priority: i32,
    /// Lock type
    pub lock_type: LockType,
}

/// Type of lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LockType {
    /// Shared read lock (multiple readers allowed)
    Read,
    /// Exclusive write lock (single writer only)
    Write,
    /// Exclusive lock (no other access)
    Exclusive,
}

impl BusinessLock {
    /// Check if the lock is expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Remaining time until expiration in seconds.
    pub fn remaining_seconds(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

/// Request for coordination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationRequest {
    /// Requesting agent
    pub agent_id: String,
    /// Resource to coordinate
    pub resource: String,
    /// Type of operation
    pub operation: LockType,
    /// Expected duration in milliseconds
    pub expected_duration_ms: u64,
    /// Priority level (higher = more important)
    pub priority: i32,
    /// Request timestamp
    pub requested_at: DateTime<Utc>,
}

impl CoordinationRequest {
    pub fn new(agent_id: impl Into<String>, resource: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            resource: resource.into(),
            operation: LockType::Write,
            expected_duration_ms: 30000, // 30 seconds default
            priority: 0,
            requested_at: Utc::now(),
        }
    }

    pub fn with_operation(mut self, op: LockType) -> Self {
        self.operation = op;
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_duration_ms(mut self, ms: u64) -> Self {
        self.expected_duration_ms = ms;
        self
    }
}

/// Result of a coordination request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationResult {
    /// Was coordination granted?
    pub granted: bool,
    /// Lock if granted
    pub lock: Option<BusinessLock>,
    /// Position in queue if waiting
    pub queue_position: Option<u32>,
    /// Estimated wait time in milliseconds
    pub estimated_wait_ms: Option<u64>,
    /// Reason if denied
    pub reason: Option<String>,
}

impl CoordinationResult {
    pub fn granted(lock: BusinessLock) -> Self {
        Self {
            granted: true,
            lock: Some(lock),
            queue_position: None,
            estimated_wait_ms: None,
            reason: None,
        }
    }

    pub fn queued(position: u32, estimated_wait_ms: u64) -> Self {
        Self {
            granted: false,
            lock: None,
            queue_position: Some(position),
            estimated_wait_ms: Some(estimated_wait_ms),
            reason: Some("Resource is locked, request queued".to_string()),
        }
    }

    pub fn denied(reason: impl Into<String>) -> Self {
        Self {
            granted: false,
            lock: None,
            queue_position: None,
            estimated_wait_ms: None,
            reason: Some(reason.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_business_lock_expiration() {
        let lock = BusinessLock {
            id: Uuid::new_v4(),
            resource: "test".to_string(),
            locked_by: "agent-1".to_string(),
            acquired_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(30),
            priority: 0,
            lock_type: LockType::Write,
        };

        assert!(!lock.is_expired());
        assert!(lock.remaining_seconds() > 0);
    }

    #[test]
    fn test_coordination_request_builder() {
        let req = CoordinationRequest::new("agent-1", "database:accounts")
            .with_operation(LockType::Exclusive)
            .with_priority(10)
            .with_duration_ms(60000);

        assert_eq!(req.agent_id, "agent-1");
        assert_eq!(req.resource, "database:accounts");
        assert_eq!(req.operation, LockType::Exclusive);
        assert_eq!(req.priority, 10);
        assert_eq!(req.expected_duration_ms, 60000);
    }
}
