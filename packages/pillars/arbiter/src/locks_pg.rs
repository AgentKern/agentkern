//! AgentKern-Arbiter: Postgres-Backed Lock Manager
//!
//! Per Phase 16 Plan: Replaces in-memory HashMap with Postgres persistence.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{BusinessLock, LockType};

/// Postgres-backed lock manager for distributed lock state.
#[derive(Clone)]
pub struct PgLockManager {
    pool: PgPool,
    default_ttl_seconds: i64,
}

impl PgLockManager {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            default_ttl_seconds: 30,
        }
    }

    pub fn with_ttl(mut self, seconds: i64) -> Self {
        self.default_ttl_seconds = seconds;
        self
    }

    /// Try to acquire a lock on a resource (via Postgres row-level locking).
    pub async fn acquire(
        &self,
        agent_id: &str,
        resource: &str,
        priority: i32,
        lock_type: LockType,
        duration_ms: Option<u64>,
    ) -> Result<BusinessLock, LockError> {
        let ttl_ms = duration_ms.unwrap_or(self.default_ttl_seconds as u64 * 1000);
        let expires_at = Utc::now() + Duration::milliseconds(ttl_ms as i64);
        let lock_type_str = format!("{:?}", lock_type);

        // Use transaction with row-level locking
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| LockError::Database(e.to_string()))?;

        // Clean up expired locks first
        sqlx::query("DELETE FROM arbiter_locks WHERE expires_at < NOW()")
            .execute(&mut *tx)
            .await
            .ok();

        // Check for existing lock (with FOR UPDATE to block concurrent access)
        let existing: Option<ExistingLock> = sqlx::query_as(
            r#"
            SELECT id, resource, locked_by, priority, expires_at
            FROM arbiter_locks 
            WHERE resource = $1
            FOR UPDATE
            "#,
        )
        .bind(resource)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| LockError::Database(e.to_string()))?;

        if let Some(existing) = existing {
            // Lock exists and is not expired
            if existing.locked_by == agent_id {
                // Extend the lock (same owner)
                sqlx::query("UPDATE arbiter_locks SET expires_at = $1 WHERE id = $2")
                    .bind(expires_at)
                    .bind(existing.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| LockError::Database(e.to_string()))?;

                tx.commit()
                    .await
                    .map_err(|e| LockError::Database(e.to_string()))?;

                return Ok(BusinessLock {
                    id: existing.id,
                    resource: resource.to_string(),
                    locked_by: agent_id.to_string(),
                    acquired_at: Utc::now(),
                    expires_at,
                    priority,
                    lock_type,
                });
            }

            // Check priority for preemption
            if priority > existing.priority {
                tracing::info!(
                    "Agent {} preempting lock on {} from {} (priority {} > {})",
                    agent_id,
                    resource,
                    existing.locked_by,
                    priority,
                    existing.priority
                );
                // Delete existing and fall through to create new
                sqlx::query("DELETE FROM arbiter_locks WHERE id = $1")
                    .bind(existing.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| LockError::Database(e.to_string()))?;
            } else {
                tx.rollback().await.ok();
                return Err(LockError::ResourceLocked {
                    resource: resource.to_string(),
                    locked_by: existing.locked_by,
                    remaining_seconds: (existing.expires_at - Utc::now()).num_seconds(),
                });
            }
        }

        // Create new lock
        let new_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO arbiter_locks (id, resource, locked_by, priority, lock_type, acquired_at, expires_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), $6)
            "#
        )
        .bind(new_id)
        .bind(resource)
        .bind(agent_id)
        .bind(priority)
        .bind(&lock_type_str)
        .bind(expires_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| LockError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| LockError::Database(e.to_string()))?;

        Ok(BusinessLock {
            id: new_id,
            resource: resource.to_string(),
            locked_by: agent_id.to_string(),
            acquired_at: Utc::now(),
            expires_at,
            priority,
            lock_type,
        })
    }

    /// Release a lock on a resource.
    pub async fn release(&self, agent_id: &str, resource: &str) -> Result<(), LockError> {
        let result =
            sqlx::query("DELETE FROM arbiter_locks WHERE resource = $1 AND locked_by = $2")
                .bind(resource)
                .bind(agent_id)
                .execute(&self.pool)
                .await
                .map_err(|e| LockError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            // Check if lock exists but owned by someone else
            let lock: Option<(String,)> =
                sqlx::query_as("SELECT locked_by FROM arbiter_locks WHERE resource = $1")
                    .bind(resource)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| LockError::Database(e.to_string()))?;

            if let Some((owner,)) = lock {
                return Err(LockError::NotOwner {
                    resource: resource.to_string(),
                    owner,
                    requester: agent_id.to_string(),
                });
            }
            return Err(LockError::NotFound {
                resource: resource.to_string(),
            });
        }

        Ok(())
    }

    /// Get the current lock status for a resource.
    pub async fn get_status(&self, resource: &str) -> Option<BusinessLock> {
        let row: Option<LockRow> = sqlx::query_as(
            r#"
            SELECT id, resource, locked_by, priority, lock_type, acquired_at, expires_at
            FROM arbiter_locks
            WHERE resource = $1 AND expires_at > NOW()
            "#,
        )
        .bind(resource)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(|r| BusinessLock {
            id: r.id,
            resource: r.resource,
            locked_by: r.locked_by,
            acquired_at: r.acquired_at,
            expires_at: r.expires_at,
            priority: r.priority,
            lock_type: match r.lock_type.as_str() {
                "Read" => LockType::Read,
                _ => LockType::Write,
            },
        })
    }

    /// Clean up expired locks.
    pub async fn cleanup_expired(&self) -> usize {
        sqlx::query("DELETE FROM arbiter_locks WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected() as usize)
            .unwrap_or(0)
    }
}

// SQLx row types
#[derive(sqlx::FromRow)]
struct ExistingLock {
    id: Uuid,
    #[allow(dead_code)] // Fix CI warning
    resource: String,
    locked_by: String,
    priority: i32,
    expires_at: chrono::DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct LockRow {
    id: Uuid,
    resource: String,
    locked_by: String,
    priority: i32,
    lock_type: String,
    acquired_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
}

/// Lock operation errors (matching original interface).
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

    #[error("Database error: {0}")]
    Database(String),
}
