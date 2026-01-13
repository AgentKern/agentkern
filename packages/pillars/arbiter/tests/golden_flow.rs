use agentkern_arbiter::coordinator::Coordinator;
use agentkern_arbiter::types::{CoordinationRequest, LockType};
// use tokio;

#[tokio::test]
async fn test_golden_flow_autonomy() {
    let coordinator = Coordinator::new();

    // 1. SAFE FLOW
    // A standard request with safe intent should be granted.
    let safe_req = CoordinationRequest::new("agent-alpha", "database:customer_info")
        .with_operation(LockType::Read)
        .with_intent("Hello world")
        .with_priority(10);

    let result = coordinator.request(safe_req).await;
    assert!(
        result.granted,
        "Safe request should be granted, but was denied: {:?}",
        result.reason
    );
    assert!(result.lock.is_some());

    // Release the lock
    coordinator
        .release_lock("agent-alpha", "database:customer_info")
        .await
        .unwrap();

    // 2. NEURAL VETO FLOW
    // A request with malicious intent should be blocked.
    // "delete from database:prod" triggers a symbolic rule.
    let malicious_req = CoordinationRequest::new("agent-malice", "database:prod")
        .with_operation(LockType::Exclusive)
        .with_intent("delete from database:prod")
        .with_priority(100);

    let result = coordinator.request(malicious_req).await;
    assert!(!result.granted, "Malicious request should be denied");
    assert!(
        result.reason.unwrap().contains("Symbolic"),
        "Should be blocked by a Symbolic rule"
    );

    // 3. DRIFT VETO FLOW
    // An agent that has drifted significantly from its original goal should be blocked.
    let mut drifted_path =
        agentkern_synapse::intent::IntentPath::new("agent-drifter", "Update user profile", 5);
    // Simulate drift: overrun + failures + circular behavior
    for i in 1..=10 {
        drifted_path.record_step(format!("step-{}", i), None);
    }
    // Pattern: A-B-A-B (circular) + recent failures
    drifted_path.record_step("query_resource", Some("error".to_string()));
    drifted_path.record_step("retry_resource", Some("error".to_string()));
    drifted_path.record_step("query_resource", Some("error".to_string()));
    drifted_path.record_step("retry_resource", Some("error".to_string()));

    coordinator.register_intent(drifted_path).await;

    let drift_req = CoordinationRequest::new("agent-drifter", "database:users")
        .with_operation(LockType::Write)
        .with_intent("Updating user password hash");

    let result = coordinator.request(drift_req).await;
    assert!(!result.granted, "Drifted agent should be denied");
    assert!(
        result
            .reason
            .unwrap()
            .to_lowercase()
            .contains("intent drift"),
        "Reason should cite intent drift"
    );
}

#[tokio::test]
async fn test_neural_risk_escalation() {
    let coordinator = Coordinator::new();

    // "bypass security" triggers a symbolic rule.
    let high_risk_req = CoordinationRequest::new("agent-sys", "security:config")
        .with_operation(LockType::Exclusive)
        .with_intent("bypass security to access hidden logs");

    let result = coordinator.request(high_risk_req).await;

    assert!(!result.granted);
    assert!(result.reason.unwrap().contains("Sovereign Security"));
}
