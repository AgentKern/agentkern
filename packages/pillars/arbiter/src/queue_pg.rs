//! AgentKern-Arbiter: Postgres-Backed Priority Queue
//!
//! Per Phase 16 Plan: Replaces in-memory Vec with Postgres persistence.

// use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{CoordinationRequest, LockType};

/// Postgres-backed priority queue for distributed state.
#[derive(Clone)]
pub struct PgQueue {
    pool: PgPool,
}

impl PgQueue {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Enqueue a coordination request.
    pub async fn enqueue(&self, request: CoordinationRequest) -> Result<usize, sqlx::Error> {
        let operation_str = format!("{:?}", request.operation);

        // Upsert: Update priority if already queued
        sqlx::query(
            r#"
            INSERT INTO arbiter_queue (id, agent_id, resource, priority, operation, expected_duration_ms, intent)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (agent_id, resource) 
            DO UPDATE SET 
                priority = GREATEST(arbiter_queue.priority, EXCLUDED.priority),
                enqueued_at = NOW()
            "#
        )
        .bind(Uuid::new_v4())
        .bind(&request.agent_id)
        .bind(&request.resource)
        .bind(request.priority)
        .bind(&operation_str)
        .bind(request.expected_duration_ms as i64)
        .bind(&request.intent)
        .execute(&self.pool)
        .await?;

        // Return position in queue
        self.get_position(&request.agent_id, &request.resource)
            .await
    }

    /// Get queue position for an agent's request.
    pub async fn get_position(&self, agent_id: &str, resource: &str) -> Result<usize, sqlx::Error> {
        // Count how many higher-priority items are ahead
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM arbiter_queue q1
            WHERE q1.resource = $1
            AND (q1.priority > (SELECT priority FROM arbiter_queue WHERE agent_id = $2 AND resource = $1)
                 OR (q1.priority = (SELECT priority FROM arbiter_queue WHERE agent_id = $2 AND resource = $1)
                     AND q1.enqueued_at < (SELECT enqueued_at FROM arbiter_queue WHERE agent_id = $2 AND resource = $1)))
            "#
        )
        .bind(resource)
        .bind(agent_id)
        .fetch_one(&self.pool)
        .await?;

        Ok((count.0 + 1) as usize)
    }

    /// Dequeue an agent's request (remove from queue).
    pub async fn dequeue(&self, agent_id: &str, resource: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM arbiter_queue WHERE agent_id = $1 AND resource = $2")
            .bind(agent_id)
            .bind(resource)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Pop the next highest-priority request for a resource (atomic via SKIP LOCKED).
    pub async fn pop(&self, resource: &str) -> Option<CoordinationRequest> {
        let row: Option<QueueRow> = sqlx::query_as(
            r#"
            DELETE FROM arbiter_queue
            WHERE id = (
                SELECT id FROM arbiter_queue
                WHERE resource = $1
                ORDER BY priority DESC, enqueued_at ASC
                FOR UPDATE SKIP LOCKED
                LIMIT 1
            )
            RETURNING id, agent_id, resource, priority, operation, expected_duration_ms, intent
            "#,
        )
        .bind(resource)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        row.map(|r| CoordinationRequest {
            agent_id: r.agent_id,
            resource: r.resource,
            priority: r.priority,
            operation: match r.operation.as_str() {
                "Read" => LockType::Read,
                _ => LockType::Write,
            },
            expected_duration_ms: r.expected_duration_ms as u64,
            intent: r.intent,
            requested_at: chrono::Utc::now(), // Restored from DB, set to now for simplicity
        })
    }

    /// Estimate wait time based on queue position.
    pub fn estimate_wait_ms(&self, position: usize, avg_lock_duration_ms: u64) -> u64 {
        position as u64 * avg_lock_duration_ms
    }

    /// Get total queue length for a resource.
    pub async fn queue_length(&self, resource: &str) -> usize {
        let count: Option<(i64,)> =
            sqlx::query_as("SELECT COUNT(*) FROM arbiter_queue WHERE resource = $1")
                .bind(resource)
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten();

        count.map(|c| c.0 as usize).unwrap_or(0)
    }
}

// SQLx row type
#[derive(sqlx::FromRow)]
struct QueueRow {
    #[allow(dead_code)]
    id: Uuid,
    agent_id: String,
    resource: String,
    priority: i32,
    operation: String,
    expected_duration_ms: i64,
    intent: Option<String>,
}
