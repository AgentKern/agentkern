use agentkern_arbiter::coordinator::Coordinator;
use agentkern_arbiter::types::{CoordinationRequest, LockType};
use madsim::{task::*, time::*};
use std::sync::Arc;
use std::time::Duration;

#[madsim::test]
async fn test_arbiter_deterministic_chaos() {
    let coord = Arc::new(Coordinator::new());
    let num_agents = 50;
    let resource = "global_lock";

    let mut handles: Vec<JoinHandle<()>> = vec![];

    for i in 0..num_agents {
        let coord = coord.clone();
        let agent_id = format!("agent-{}", i);

        handles.push(spawn(async move {
            let mut successes = 0;
            while successes < 5 {
                let mut req = CoordinationRequest::new(&agent_id, resource);
                req.priority = (i % 10) as i32; // Mixed priorities

                let result = coord.request(req).await;
                if result.granted {
                    // Simulate work
                    sleep(Duration::from_millis(5)).await;
                    if let Err(e) = coord.release_lock(&agent_id, resource).await {
                        // Preemption is expected in this chaos test due to mixed priorities
                        if !e.contains("not owned by") {
                            panic!("Failed to release: {}", e);
                        }
                    }
                    successes += 1;
                } else {
                    // Wait a bit and retry
                    sleep(Duration::from_millis(10)).await;
                }
            }
        }));
    }

    // Wait for all agents to finish or timeout
    for h in handles {
        h.await.unwrap();
    }

    // Final check: resource should be free
    let status = coord.get_lock_status(resource).await;
    assert!(
        status.is_none(),
        "Resource should be free at the end of simulation"
    );
}
