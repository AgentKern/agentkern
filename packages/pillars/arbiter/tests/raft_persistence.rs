use agentkern_arbiter::raft_manager::{NodeId, RaftLockManager};
use std::time::Duration;

#[tokio::test]
async fn test_raft_lock_persistence_across_restarts() {
    let temp_dir = std::env::temp_dir().join("raft_persistence_test");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();
    let path = temp_dir.to_str().unwrap().to_string();

    let node_id: NodeId = 1;
    let agent_id = "test-agent-1";
    let resource = "resource-1";

    // === Phase 1: Initialize Raft and acquire lock ===
    {
        let manager =
            RaftLockManager::new(node_id, "127.0.0.1:9001".to_string(), path.clone()).await
                .expect("Failed to create RaftLockManager");

        // Initialize single-node cluster
        let nodes = std::collections::BTreeMap::from([(node_id, ())]);
        manager
            .raft
            .initialize(nodes)
            .await
            .expect("Failed to initialize raft");

        // Wait for leader election
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Acquire lock
        let success = manager
            .acquire_lock(agent_id, resource, 10, 60000)
            .await
            .expect("Failed to acquire lock");
        assert!(success, "Lock should be granted");

        // Verify it exists in memory
        let metrics = manager.raft.metrics().borrow().clone();
        assert!(metrics.current_term >= 1);

        // "Shutdown" manager gracefully to release sled lock
        manager.raft.shutdown().await.expect("Raft shutdown failed");
        drop(manager);
    }

    // === Phase 2: Create new manager pointing to same data and verify lock ===
    {
        // Re-open with same path
        let manager =
            RaftLockManager::new(node_id, "127.0.0.1:9001".to_string(), path.clone()).await
                .expect("Failed to create RaftLockManager on restart");

        // Wait for Raft to stabilize and elect leader (it should be us)
        let mut leader_ready = false;
        for i in 0..40 {
            let metrics = manager.raft.metrics().borrow().clone();
            println!(
                "Attempt {}: State={:?}, Term={}, LastLogIdx={:?}, Applied={:?}",
                i,
                metrics.state,
                metrics.current_term,
                metrics.last_log_index,
                metrics.last_applied
            );
            if metrics.state == agentkern_arbiter::raft_manager::ServerState::Leader {
                leader_ready = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        assert!(leader_ready, "Node failed to become leader after restart");

        // The state machine should have the lock loaded from disk!
        // We can check this by trying to acquire the same lock with lower priority - it should return false.
        let result = manager
            .acquire_lock("another-agent", resource, 5, 20000)
            .await
            .expect("Acquire should not error when leader");
        assert!(
            !result,
            "Lock should have been DENIED because it exists on disk"
        );

        // Let's try to heartbeat it. If it exists, it should return Ok(true).
        let hb_result = manager
            .heartbeat(agent_id, resource)
            .await
            .expect("Heartbeat should not error");
        assert!(hb_result, "Heartbeat should succeed if lock was persisted");
    }

    // Clean up
    let _ = std::fs::remove_dir_all(temp_dir);
}
