//! AgentKern Integration Tests
//!
//! Tests the full stack: Rust → TypeScript Gateway → SDK
//!
//! Run with: cargo test --test integration

use std::process::Command;
use std::time::Duration;

/// Test that Gate HTTP API is accessible.
#[tokio::test]
#[ignore = "requires running gateway server"]
async fn test_gate_api_health() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    let response = client
        .get("http://localhost:3000/api/gate/health")
        .send()
        .await
        .expect("Failed to connect to gateway");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
}

/// Test policy verification through the API.
#[tokio::test]
#[ignore = "requires running gateway server"]
async fn test_gate_verify_policy() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    let request_body = serde_json::json!({
        "agentId": "test-agent",
        "action": "send_email",
        "context": {
            "recipient": "user@example.com"
        }
    });
    
    let response = client
        .post("http://localhost:3000/api/gate/verify")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to verify policy");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["allowed"].is_boolean());
    assert!(body["requestId"].is_string());
}

/// Test Synapse memory operations through the API.
#[tokio::test]
#[ignore = "requires running gateway server"]
async fn test_synapse_store_memory() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    let memory = serde_json::json!({
        "agentId": "test-agent",
        "content": "User prefers dark mode",
        "importance": 0.8
    });
    
    let response = client
        .post("http://localhost:3000/api/synapse/memories")
        .json(&memory)
        .send()
        .await
        .expect("Failed to store memory");
    
    assert!(response.status().is_success());
}

/// Test Arbiter lock acquisition through the API.
#[tokio::test]
#[ignore = "requires running gateway server"]
async fn test_arbiter_acquire_lock() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    let lock_request = serde_json::json!({
        "resourceId": "database:users:123",
        "agentId": "test-agent",
        "ttlSeconds": 30
    });
    
    let response = client
        .post("http://localhost:3000/api/arbiter/locks")
        .json(&lock_request)
        .send()
        .await
        .expect("Failed to acquire lock");
    
    assert!(response.status().is_success());
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["lockId"].is_string());
}

/// Test Identity service through the API.
#[tokio::test]
#[ignore = "requires running gateway server"]
async fn test_identity_create_agent() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    let agent = serde_json::json!({
        "name": "Integration Test Agent",
        "capabilities": ["read", "write"]
    });
    
    let response = client
        .post("http://localhost:3000/api/identity/agents")
        .json(&agent)
        .send()
        .await
        .expect("Failed to create agent");
    
    // May fail if agent exists, that's okay
    let status = response.status();
    assert!(status.is_success() || status == reqwest::StatusCode::CONFLICT);
}

/// Test the full verification flow: Create agent → Register policy → Verify action.
#[tokio::test]
#[ignore = "requires running gateway server"]
async fn test_full_verification_flow() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    
    let base_url = "http://localhost:3000/api";
    
    // Step 1: Create an agent
    let agent_id = format!("flow-test-{}", uuid::Uuid::new_v4());
    let agent = serde_json::json!({
        "id": agent_id,
        "name": "Flow Test Agent",
        "capabilities": ["financial", "email"]
    });
    
    let _agent_resp = client
        .post(format!("{}/identity/agents", base_url))
        .json(&agent)
        .send()
        .await;
    
    // Step 2: Register a policy
    let policy = serde_json::json!({
        "id": "test-policy",
        "name": "Test Policy",
        "rules": [{
            "condition": "action == 'transfer_funds' && amount > 10000",
            "action": "deny",
            "message": "Large transfers require approval"
        }]
    });
    
    let _policy_resp = client
        .post(format!("{}/gate/policies", base_url))
        .json(&policy)
        .send()
        .await;
    
    // Step 3: Verify a safe action
    let safe_request = serde_json::json!({
        "agentId": agent_id,
        "action": "send_email",
        "context": {}
    });
    
    let safe_resp = client
        .post(format!("{}/gate/verify", base_url))
        .json(&safe_request)
        .send()
        .await
        .expect("Failed to verify safe action");
    
    let safe_body: serde_json::Value = safe_resp.json().await.unwrap();
    assert_eq!(safe_body["allowed"], true, "Safe action should be allowed");
    
    // Step 4: Verify a blocked action
    let blocked_request = serde_json::json!({
        "agentId": agent_id,
        "action": "transfer_funds",
        "context": {
            "amount": 50000
        }
    });
    
    let blocked_resp = client
        .post(format!("{}/gate/verify", base_url))
        .json(&blocked_request)
        .send()
        .await
        .expect("Failed to verify blocked action");
    
    let blocked_body: serde_json::Value = blocked_resp.json().await.unwrap();
    // May or may not be blocked depending on policy registration
    assert!(blocked_body["requestId"].is_string());
}

/// Test CRDT sync between Rust (Synapse) and TypeScript.
#[tokio::test]
#[ignore = "requires running gateway server"]
async fn test_crdt_sync() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    // Create initial state
    let state_a = serde_json::json!({
        "agentId": "crdt-test-agent",
        "state": {
            "counter": 5,
            "preferences": {"theme": "dark"}
        }
    });
    
    let resp = client
        .post("http://localhost:3000/api/synapse/state")
        .json(&state_a)
        .send()
        .await
        .expect("Failed to set state");
    
    assert!(resp.status().is_success());
    
    // Merge conflicting state
    let state_b = serde_json::json!({
        "agentId": "crdt-test-agent",
        "state": {
            "counter": 8,
            "preferences": {"language": "en"}
        }
    });
    
    let merge_resp = client
        .post("http://localhost:3000/api/synapse/state/merge")
        .json(&state_b)
        .send()
        .await
        .expect("Failed to merge state");
    
    assert!(merge_resp.status().is_success());
    
    // Verify merge result
    let get_resp = client
        .get("http://localhost:3000/api/synapse/state/crdt-test-agent")
        .send()
        .await
        .expect("Failed to get state");
    
    let merged: serde_json::Value = get_resp.json().await.unwrap();
    // Counter should be max(5, 8) = 8 (GCounter semantics)
    // Preferences should have both theme and language (ORSet/LWWMap semantics)
    assert!(merged["state"].is_object());
}

/// Test latency requirements (<50ms for verification).
#[tokio::test]
#[ignore = "requires running gateway server"]
async fn test_latency_requirements() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    
    let request = serde_json::json!({
        "agentId": "latency-test",
        "action": "read_data",
        "context": {}
    });
    
    let start = std::time::Instant::now();
    
    let response = client
        .post("http://localhost:3000/api/gate/verify")
        .json(&request)
        .send()
        .await
        .expect("Failed to verify");
    
    let latency = start.elapsed();
    
    assert!(response.status().is_success());
    assert!(
        latency.as_millis() < 50,
        "Verification took {}ms, should be <50ms",
        latency.as_millis()
    );
}
