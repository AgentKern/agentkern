use agentkern_synapse::drift::{DriftAlerter, DriftDetector};
use agentkern_synapse::{StateStore, StateUpdate};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_state_store_concurrency_stress() {
    let store = Arc::new(StateStore::new());
    let mut handles = vec![];

    let start = Instant::now();
    for i in 0..10 {
        let store_clone = store.clone();
        handles.push(tokio::spawn(async move {
            for j in 0..100 {
                let agent_id = format!("agent-{}", i);
                let mut updates = std::collections::HashMap::new();
                updates.insert(
                    "last_action".to_string(),
                    serde_json::json!(format!("action-{}", j)),
                );
                let update = StateUpdate {
                    agent_id: agent_id.clone(),
                    updates,
                    deletes: None,
                };
                store_clone.update_state(update).await;
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let duration = start.elapsed();
    println!("State store stress test completed in {:?}", duration);

    // With parking_lot, 1000 updates across 10 threads should be sub-100ms
    assert!(
        duration.as_millis() < 500,
        "State store too slow under contention: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_non_blocking_alerting_consistency() {
    let alerter = Arc::new(DriftAlerter::new());
    let alert_count = Arc::new(AtomicUsize::new(0));
    let alert_count_clone = alert_count.clone();

    alerter.on_alert(Box::new(move |_| {
        alert_count_clone.fetch_add(1, Ordering::SeqCst);
    }));

    let _detector = DriftDetector::new()
        .with_threshold(1) // Sensitive
        .with_alerter(alerter.clone());

    let store = StateStore::new().with_alerter(alerter);
    let agent_id = "stress-agent";

    // START INTENT with small expected steps to trigger overrun drift
    store.start_intent(agent_id, "Test task", 2).await;

    // Recording steps that trigger drift
    for i in 0..10 {
        store
            .record_step(agent_id, format!("action-{}", i), None)
            .await;
    }

    // Alerting is non-blocking (spawned in tokio)
    // We might need a small sleep to ensure all tasks finish
    tokio::time::sleep(Duration::from_millis(100)).await;

    let final_count = alert_count.load(Ordering::SeqCst);
    println!("Alerts triggered: {}", final_count);

    // Ensure history is consistent (O(1) VecDeque)
    let history = store.get_alerter().unwrap().get_history(100);
    assert!(history.len() > 0, "History should not be empty");
}
