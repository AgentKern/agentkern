//! Arbiter Distributed State Integration Tests
//!
//! Tests that locks and queue state survive across "restarts" (simulated by
//! creating new PgCoordinator instances with the same database).
//!
//! Run with: cargo test -p agentkern-arbiter --test distributed_state -- --test-threads=1

use agentkern_arbiter::{
    coordinator_pg::PgCoordinator,
    locks_pg::PgLockManager,
    queue_pg::PgQueue,
    types::{CoordinationRequest, LockType},
};
use sqlx::postgres::PgPoolOptions;
// use std::sync::Arc;

/// Get database URL from environment (use TEST_DATABASE_URL for isolated testing)
fn get_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("DATABASE_URL or TEST_DATABASE_URL must be set")
}

/// Helper to run migrations (assumes migrations are applied)
async fn setup_pool() -> sqlx::PgPool {
    let database_url = get_database_url();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

/// Clean up test data before each test
async fn cleanup(pool: &sqlx::PgPool) {
    sqlx::query("DELETE FROM arbiter_locks WHERE resource LIKE 'test:%'")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM arbiter_queue WHERE resource LIKE 'test:%'")
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_lock_persists_across_restart() {
    let pool = setup_pool().await;
    cleanup(&pool).await;

    let resource = "test:lock_persistence_1";
    let agent_id = "agent-persistence-test";

    // === Phase 1: Acquire lock with first coordinator ===
    {
        let coord1 = PgCoordinator::new(pool.clone());

        let request = CoordinationRequest::new(agent_id, resource)
            .with_priority(5)
            .with_duration_ms(60000); // 60 second TTL

        let result = coord1.request(request).await;
        assert!(result.granted, "Lock should be granted");
        assert!(result.lock.is_some(), "Lock object should be returned");

        let lock = result.lock.unwrap();
        assert_eq!(lock.locked_by, agent_id);
        assert_eq!(lock.resource, resource);

        // Coordinator goes out of scope (simulating restart)
    }

    // === Phase 2: Verify lock exists with NEW coordinator ===
    {
        let coord2 = PgCoordinator::new(pool.clone());

        // Lock should still exist
        let status = coord2.get_lock_status(resource).await;
        assert!(status.is_some(), "Lock should persist across restart");

        let lock = status.unwrap();
        assert_eq!(lock.locked_by, agent_id, "Lock owner should be preserved");

        // Another agent should NOT be able to acquire (same priority)
        let conflict_request =
            CoordinationRequest::new("agent-conflict", resource).with_priority(5);

        let conflict_result = coord2.request(conflict_request).await;
        assert!(
            !conflict_result.granted,
            "Conflicting lock should be denied"
        );
        assert!(conflict_result.queue_position.is_some(), "Should be queued");
    }

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_queue_persists_across_restart() {
    let pool = setup_pool().await;
    cleanup(&pool).await;

    let resource = "test:queue_persistence_2";

    // === Phase 1: Create lock and enqueue waiters ===
    {
        let coord1 = PgCoordinator::new(pool.clone());

        // Agent 1 acquires lock
        let req1 = CoordinationRequest::new("agent-1", resource).with_priority(5);
        let result1 = coord1.request(req1).await;
        assert!(result1.granted);

        // Agent 2 gets queued
        let req2 = CoordinationRequest::new("agent-2", resource).with_priority(3);
        let result2 = coord1.request(req2).await;
        assert!(!result2.granted);
        assert_eq!(result2.queue_position, Some(1));

        // Agent 3 gets queued (higher priority than agent-2)
        let req3 = CoordinationRequest::new("agent-3", resource).with_priority(4);
        let result3 = coord1.request(req3).await;
        assert!(!result3.granted);
    }

    // === Phase 2: Verify queue with NEW coordinator ===
    {
        let coord2 = PgCoordinator::new(pool.clone());

        // Release agent-1's lock
        coord2
            .release_lock("agent-1", resource)
            .await
            .expect("Release should succeed");

        // After release, the queue should grant to higher priority waiter (agent-3)
        // Note: In the current implementation, pop() returns the highest priority first

        // Verify lock is now held by next in queue
        let status = coord2.get_lock_status(resource).await;
        assert!(status.is_some(), "Lock should be granted to next in queue");

        let lock = status.unwrap();
        // Agent 3 has priority 4 > Agent 2 priority 3, so agent-3 should get lock
        assert_eq!(
            lock.locked_by, "agent-3",
            "Higher priority agent should get lock"
        );
    }

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_lock_manager_direct() {
    let pool = setup_pool().await;
    cleanup(&pool).await;

    let lock_manager = PgLockManager::new(pool.clone());
    let resource = "test:lock_manager_3";

    // Acquire
    let lock = lock_manager
        .acquire("agent-1", resource, 5, LockType::Write, Some(30000))
        .await
        .expect("Acquire should succeed");

    assert_eq!(lock.locked_by, "agent-1");

    // Get status
    let status = lock_manager.get_status(resource).await;
    assert!(status.is_some());

    // Release
    lock_manager
        .release("agent-1", resource)
        .await
        .expect("Release should succeed");

    // Verify released
    let status = lock_manager.get_status(resource).await;
    assert!(status.is_none(), "Lock should be gone after release");

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_queue_direct() {
    let pool = setup_pool().await;
    cleanup(&pool).await;

    let queue = PgQueue::new(pool.clone());
    let resource = "test:queue_direct_4";

    // Enqueue requests
    let req1 = CoordinationRequest::new("agent-1", resource).with_priority(3);
    let pos1 = queue.enqueue(req1).await.expect("Enqueue should succeed");
    assert_eq!(pos1, 1);

    let req2 = CoordinationRequest::new("agent-2", resource).with_priority(5);
    let _pos2 = queue.enqueue(req2).await.expect("Failed to enqueue 2");
    // Agent-2 has higher priority, so position depends on ordering logic

    // Pop should return higher priority first
    let next = queue.pop(resource).await;
    assert!(next.is_some());
    assert_eq!(next.unwrap().agent_id, "agent-2"); // Higher priority

    let next2 = queue.pop(resource).await;
    assert!(next2.is_some());
    assert_eq!(next2.unwrap().agent_id, "agent-1");

    // Queue should be empty now
    let next3 = queue.pop(resource).await;
    assert!(next3.is_none());

    cleanup(&pool).await;
}

#[tokio::test]
async fn test_priority_preemption() {
    let pool = setup_pool().await;
    cleanup(&pool).await;

    let lock_manager = PgLockManager::new(pool.clone());
    let resource = "test:preemption_5";

    // Low priority acquires
    lock_manager
        .acquire("low-agent", resource, 1, LockType::Write, Some(60000))
        .await
        .expect("Low priority acquire should succeed");

    // High priority preempts
    let preempt_lock = lock_manager
        .acquire("high-agent", resource, 10, LockType::Write, Some(60000))
        .await
        .expect("High priority should preempt");

    assert_eq!(preempt_lock.locked_by, "high-agent");

    // Verify preemption
    let status = lock_manager.get_status(resource).await.unwrap();
    assert_eq!(status.locked_by, "high-agent");

    cleanup(&pool).await;
}
