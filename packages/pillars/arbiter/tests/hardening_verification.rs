#![cfg(feature = "full")]
use agentkern_arbiter::antifragile::{AntifragileEngine, Failure};
use std::time::Instant;

#[tokio::test]
async fn test_antifragile_velocity_performance() {
    let engine = AntifragileEngine::new();
    let resource = "high-throughput-resource";

    // Warm up
    for _ in 0..10 {
        let failure = Failure::new(resource, "warmup error");
        engine.handle_failure(failure).await;
    }

    let start = Instant::now();
    let iterations = 1000;

    for i in 0..iterations {
        let failure = Failure::new(resource, format!("error-{}", i));
        engine.handle_failure(failure).await;
    }

    let duration = start.elapsed();
    let avg_ms = duration.as_secs_f64() * 1000.0 / iterations as f64;

    println!("Avg failure handling time: {}ms", avg_ms);

    // We expect O(1) performance to be extremely fast (<< 1ms per failure)
    assert!(avg_ms < 0.5, "Failure handling too slow: {}ms", avg_ms);
}

#[tokio::test]
async fn test_circuit_breaker_prediction_trigger() {
    // Test that the predictive circuit breaker opens on high velocity
    // Default threshold is 10 failures/min.
    let engine = AntifragileEngine::new();
    let resource = "predictive-resource";

    // Flood with failures to trigger velocity protection
    for _ in 0..15 {
        let failure = Failure::new(resource, "flood error");
        engine.handle_failure(failure).await;
    }

    // Velocity threshold should have been trippped
    assert!(
        !engine.is_service_available(resource).await,
        "Circuit should be predictively OPEN"
    );
}
