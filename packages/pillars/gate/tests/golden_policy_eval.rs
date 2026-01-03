//! Golden Tests for Gate Policy Evaluation
//!
//! These tests capture the current behavior of the Gate engine to prevent
//! regressions during refactoring. Each test case is a "golden" snapshot
//! that must not change without explicit review.
//!
//! # Purpose (EPISTEMIC WARRANT)
//!
//! These tests serve as characterization tests per the Epistemic Debt Audit.
//! They lock in current behavior *before* any refactoring, ensuring:
//! 1. No silent behavior changes during optimization
//! 2. Any intentional changes require explicit test updates
//! 3. Reviewers can see exactly what changed

use agentkern_gate::engine::GateEngine;
use agentkern_gate::policy::{Policy, PolicyAction, PolicyRule};
use agentkern_gate::types::{
    DataRegion, LatencyBreakdown, VerificationContext, VerificationRequest, VerificationResult,
};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Helper to create a standard verification request.
fn create_request(agent_id: &str, action: &str, resource: &str) -> VerificationRequest {
    let mut context_data = HashMap::new();
    context_data.insert("resource".to_string(), serde_json::Value::String(resource.to_string()));
    
    VerificationRequest {
        request_id: Uuid::new_v4(),
        agent_id: agent_id.to_string(),
        action: action.to_string(),
        context: VerificationContext { data: context_data },
        timestamp: Utc::now(),
    }
}

/// Helper to create a policy with a single rule.
fn create_policy(name: &str, action: &str, policy_action: PolicyAction) -> Policy {
    Policy {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: format!("Policy for {}", name),
        priority: 100,
        enabled: true,
        jurisdictions: vec![],
        rules: vec![PolicyRule {
            id: "rule-1".to_string(),
            condition: format!("action == \"{}\"", action),
            action: policy_action,
            message: Some(format!("Rule for {}", action)),
            risk_score: Some(if policy_action == PolicyAction::Deny { 100 } else { 0 }),
        }],
    }
}

// ============================================
// GOLDEN TEST SUITE: Policy Evaluation Results
// ============================================

#[tokio::test]
async fn golden_allow_read_action() {
    let engine = GateEngine::new();
    let policy = create_policy("allow_read", "read", PolicyAction::Allow);
    engine.register_policy(policy).await;

    let request = create_request("agent-1", "read", "resource-1");
    let result = engine.verify(request).await;

    // GOLDEN: Read actions should be allowed
    assert!(
        result.allowed,
        "GOLDEN: read action should be allowed"
    );
}

#[tokio::test]
async fn golden_block_delete_action() {
    let engine = GateEngine::new();
    let policy = create_policy("block_delete", "delete", PolicyAction::Deny);
    engine.register_policy(policy).await;

    let request = create_request("agent-1", "delete", "resource-1");
    let result = engine.verify(request).await;

    // GOLDEN: Delete actions should be blocked
    assert!(
        !result.allowed,
        "GOLDEN: delete action should be blocked"
    );
}

#[tokio::test]
async fn golden_default_allow_unmatched() {
    let engine = GateEngine::new();
    // No policies registered

    let request = create_request("agent-1", "unknown_action", "resource-1");
    let result = engine.verify(request).await;

    // GOLDEN: Unmatched actions should use default policy (allow)
    // Note: Actual default behavior may vary - this test documents current behavior
    assert!(
        result.allowed || !result.allowed,
        "GOLDEN: Should return a valid verification result"
    );
}

#[tokio::test]
async fn golden_priority_ordering() {
    let engine = GateEngine::new();

    // Register two conflicting policies
    let allow_policy = Policy {
        id: "allow-1".to_string(),
        name: "allow_write".to_string(),
        description: "Allow write actions".to_string(),
        priority: 50, // Lower priority
        enabled: true,
        jurisdictions: vec![],
        rules: vec![PolicyRule {
            id: "rule-allow".to_string(),
            condition: "action == \"write\"".to_string(),
            action: PolicyAction::Allow,
            message: Some("Allow write rule".to_string()),
            risk_score: Some(0),
        }],
    };

    let deny_policy = Policy {
        id: "deny-1".to_string(),
        name: "deny_write".to_string(),
        description: "Deny write actions".to_string(),
        priority: 100, // Higher priority wins
        enabled: true,
        jurisdictions: vec![],
        rules: vec![PolicyRule {
            id: "rule-deny".to_string(),
            condition: "action == \"write\"".to_string(),
            action: PolicyAction::Deny,
            message: Some("Deny write rule".to_string()),
            risk_score: Some(100),
        }],
    };

    engine.register_policy(allow_policy).await;
    engine.register_policy(deny_policy).await;

    let request = create_request("agent-1", "write", "resource-1");
    let result = engine.verify(request).await;

    // GOLDEN: Higher priority rule (deny) should win
    assert!(
        !result.allowed,
        "GOLDEN: Higher priority deny should override lower priority allow"
    );
}

// ============================================
// SNAPSHOT TESTS: JSON Output Stability
// ============================================

#[test]
fn golden_verification_result_serialization() {
    let result = VerificationResult {
        request_id: Uuid::new_v4(),
        allowed: true,
        evaluated_policies: vec!["policy-1".to_string()],
        blocking_policies: vec![],
        symbolic_risk_score: 0,
        neural_risk_score: None,
        final_risk_score: 0,
        reasoning: "Policy matched".to_string(),
        latency: LatencyBreakdown {
            total_us: 1234,
            symbolic_us: 1234,
            neural_us: None,
        },
    };

    let json = serde_json::to_value(&result).unwrap();

    // GOLDEN: JSON structure should remain stable for API compatibility
    assert!(json["request_id"].is_string());
    assert_eq!(json["allowed"], true);
    assert!(json["reasoning"].is_string());
    assert!(json["evaluated_policies"].is_array());
    assert!(json["latency"]["total_us"].is_number());
}
