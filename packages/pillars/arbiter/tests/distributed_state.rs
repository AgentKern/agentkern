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

/// Clean up test data for a specific resource
async fn cleanup_resource(pool: &sqlx::PgPool, resource: &str) {
    sqlx::query("DELETE FROM arbiter_locks WHERE resource = $1")
        .bind(resource)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM arbiter_queue WHERE resource = $1")
        .bind(resource)
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_lock_persists_across_restart() {
    let pool = setup_pool().await;
    let resource = "test:lock_persistence_1";
    cleanup_resource(&pool, resource).await;
    let agent_id = "agent-persistence-test";

    // === Phase 1: Acquire lock with first coordinator ===
    {
        let coord1 = PgCoordinator::new(pool.clone());

        let request = CoordinationRequest::new(agent_id, resource)
            .with_priority(5)
            .with_duration_ms(60000)
            .with_intent("Safe lock acquisition for persistence test");

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
        let conflict_request = CoordinationRequest::new("agent-conflict", resource)
            .with_priority(5)
            .with_intent("Safe conflicting request for test");

        let conflict_result = coord2.request(conflict_request).await;
        assert!(
            !conflict_result.granted,
            "Conflicting lock should be denied"
        );
        assert!(conflict_result.queue_position.is_some(), "Should be queued");
    }

    cleanup_resource(&pool, resource).await;
}

#[tokio::test]
async fn test_queue_persists_across_restart() {
    let pool = setup_pool().await;
    let resource = "test:queue_persistence_2";
    cleanup_resource(&pool, resource).await;

    // === Phase 1: Create lock and enqueue waiters ===
    {
        let coord1 = PgCoordinator::new(pool.clone());

        // Agent 1 acquires lock
        let req1 = CoordinationRequest::new("agent-1", resource)
            .with_priority(5)
            .with_duration_ms(60000)
            .with_intent("Safe initial lock");
        let result1 = coord1.request(req1).await;
        assert!(result1.granted);

        // Agent 2 gets queued
        let req2 = CoordinationRequest::new("agent-2", resource)
            .with_priority(3)
            .with_intent("Safe waiter 1");
        let result2 = coord1.request(req2).await;
        assert!(!result2.granted);
        assert_eq!(result2.queue_position, Some(1));

        // Agent 3 gets queued (higher priority than agent-2)
        let req3 = CoordinationRequest::new("agent-3", resource)
            .with_priority(4)
            .with_intent("Safe waiter 2");
        let result3 = coord1.request(req3).await;
        assert!(!result3.granted);
    }

    // === Phase 2: Verify queue with NEW coordinator ===
    {
        let coord2 = PgCoordinator::new(pool.clone());

        // Verify lock exists before releasing (with retry)
        let mut attempts = 0;
        let mut pre_release_status = None;
        while attempts < 5 {
            pre_release_status = coord2.get_lock_status(resource).await;
            if pre_release_status.is_some() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            attempts += 1;
        }
        assert!(
            pre_release_status.is_some(),
            "Lock missing before release in Phase 2 after retries"
        );
        let lock = pre_release_status.unwrap();
        assert_eq!(
            lock.locked_by, "agent-1",
            "Lock owner mismatch before release"
        );

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

    cleanup_resource(&pool, resource).await;
}

#[tokio::test]
async fn test_lock_manager_direct() {
    let pool = setup_pool().await;
    let resource = "test:lock_manager_3";
    cleanup_resource(&pool, resource).await;

    let lock_manager = PgLockManager::new(pool.clone());

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

    cleanup_resource(&pool, resource).await;
}

#[tokio::test]
async fn test_queue_direct() {
    let pool = setup_pool().await;
    let resource = "test:queue_direct_4";
    cleanup_resource(&pool, resource).await;

    let queue = PgQueue::new(pool.clone());

    // Enqueue requests
    let req1 = CoordinationRequest::new("agent-1", resource)
        .with_priority(3)
        .with_intent("Safe direct queue 1");
    let pos1 = queue.enqueue(req1).await.expect("Enqueue should succeed");
    assert_eq!(pos1, 1);

    let req2 = CoordinationRequest::new("agent-2", resource)
        .with_priority(5)
        .with_intent("Safe direct queue 2");
    let _pos2 = queue.enqueue(req2).await.expect("Failed to enqueue 2");
    // Agent-2 has higher priority, so position depends on ordering logic

    // Verify queue length
    let len = queue.queue_length(resource).await;
    assert_eq!(len, 2, "Queue should have 2 items");

    // Pop should return higher priority first (with retry)
    let mut next = None;
    for _ in 0..5 {
        next = queue.pop(resource).await;
        if next.is_some() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    assert!(next.is_some());
    assert_eq!(next.unwrap().agent_id, "agent-2"); // Higher priority

    let next2 = queue.pop(resource).await;
    assert!(next2.is_some());
    assert_eq!(next2.unwrap().agent_id, "agent-1");

    // Queue should be empty now
    let next3 = queue.pop(resource).await;
    assert!(next3.is_none());

    cleanup_resource(&pool, resource).await;
}

#[tokio::test]
async fn test_priority_preemption() {
    let pool = setup_pool().await;
    let resource = "test:preemption_5";
    cleanup_resource(&pool, resource).await;

    let lock_manager = PgLockManager::new(pool.clone());

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

    cleanup_resource(&pool, resource).await;
}
