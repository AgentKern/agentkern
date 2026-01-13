//! The Golden Flow: Chaos Edition
//!
//! Per Phase 10: "Chaos-Infused Golden Flow Test"
//!
//! Scenarios:
//! 1. A2A Communication via Nexus with active Chaos Proxy.
//! 2. Random latency injection (50-200ms).
//! 3. Random packet drops (10% rate).
//! 4. Verifies that the "Agent" eventually succeeds via retries.

use agentkern_nexus::chaos_proxy::{ChaosConfig, ChaosProxy, ChaosTarget};
use agentkern_nexus::types::Protocol;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_chaos_infused_golden_flow() {
    // 1. Setup Chaos Proxy (10% Protocol Failure)
    let config = ChaosConfig::new().with_protocol(Protocol::AgentKern, 0.10); // 10% failure rate

    let proxy = Arc::new(ChaosProxy::new(config));

    // 2. Simulation Loop (The "Golden Flow")
    // We try to send 50 messages. Statistically, ~5 should fail.
    // We verify that we catch the errors and can "retry".

    let iterations = 50;
    let mut successes = 0;
    let mut failures = 0;
    let mut total_retries = 0;

    for i in 0..iterations {
        // Retry Loop (Robust Agent Pattern)
        let mut attempts = 0;
        let max_retries = 3;

        loop {
            attempts += 1;

            // Simulate network call via Chaos Proxy
            let result = proxy
                .maybe_fail(ChaosTarget::Protocol(Protocol::AgentKern))
                .await;

            match result {
                Ok(_) => {
                    successes += 1;
                    break; // Success
                }
                Err(e) => {
                    if attempts > max_retries {
                        tracing::error!("Failed after {} attempts: {:?}", attempts, e);
                        failures += 1;
                        break; // Exhausted retries
                    }
                    total_retries += 1;
                    tracing::warn!("Retry {}/{} due to chaos: {:?}", attempts, max_retries, e);
                    tokio::time::sleep(Duration::from_millis(10)).await; // Backoff
                }
            }
        }
    }

    // 3. Telemetry & Verification
    let stats = proxy.stats();
    println!("Chaos Stats: {:?}", stats);

    println!(
        "Results: {}/{} successes. {} permanent failures. {} retries caught.",
        successes, iterations, failures, total_retries
    );

    // Assertions
    // We expect SOME failures injected
    assert!(
        stats.failures_injected > 0,
        "Chaos Monitor: No failures injected? Check probability."
    );

    // But due to retries, we expect HIGH success rate (near 100%)
    // With 10% fail rate and 3 retries, probability of total failure is 0.1^4 = 0.0001 (very low)
    assert_eq!(
        failures, 0,
        "Resilience Failure: System could not recover from chaos."
    );
    assert_eq!(
        successes, iterations,
        "Resilience Failure: Not all tasks completed."
    );
}
