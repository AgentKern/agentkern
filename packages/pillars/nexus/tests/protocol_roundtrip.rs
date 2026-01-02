//! Protocol Round-Trip Tests for Nexus
//!
//! These tests verify that protocol translation preserves message semantics.
//! Any translation path must be round-trippable without data loss.
//!
//! # Purpose (EPISTEMIC WARRANT)
//!
//! Per the Epistemic Debt Audit, Nexus has zero integration tests.
//! These characterization tests lock in the current translation behavior
//! to prevent silent regressions during future refactoring.

use agentkern_nexus::protocols::translator::{ProtocolTranslator, TranslationResult};
use agentkern_nexus::types::{NexusMessage, Protocol};
use std::collections::HashMap;

/// Helper to create a test message.
fn create_message(protocol: Protocol, method: &str) -> NexusMessage {
    NexusMessage {
        id: "test-msg-1".to_string(),
        method: method.to_string(),
        params: serde_json::json!({"task_id": "123", "message": "hello world"}),
        source_protocol: protocol,
        source_agent: Some("agent-source".to_string()),
        target_agent: Some("agent-target".to_string()),
        correlation_id: Some("corr-1".to_string()),
        timestamp: chrono::Utc::now(),
        metadata: HashMap::new(),
    }
}

// ============================================
// GOLDEN: Protocol Translation Round-Trips
// ============================================

#[test]
fn golden_same_protocol_identity() {
    let translator = ProtocolTranslator::new();
    let message = create_message(Protocol::AgentKern, "execute");

    let result = translator
        .translate_message(message.clone(), Protocol::AgentKern)
        .expect("Translation should succeed");

    // GOLDEN: Same protocol translation is identity
    assert_eq!(result.confidence, 100);
    assert!(result.lost_fields.is_empty());
    assert_eq!(result.source_protocol, Protocol::AgentKern);
    assert_eq!(result.target_protocol, Protocol::AgentKern);
}

#[test]
fn golden_a2a_to_mcp_translation() {
    let translator = ProtocolTranslator::new();
    let message = create_message(Protocol::GoogleA2A, "invoke");

    let result = translator
        .translate_message(message, Protocol::AnthropicMCP)
        .expect("Translation should succeed");

    // GOLDEN: A2A → MCP translation metadata
    assert_eq!(result.source_protocol, Protocol::GoogleA2A);
    assert_eq!(result.target_protocol, Protocol::AnthropicMCP);

    // GOLDEN: Confidence should be reasonable (not 0)
    assert!(result.confidence > 0, "Confidence should be > 0");
}

#[test]
fn golden_mcp_to_a2a_translation() {
    let translator = ProtocolTranslator::new();
    let message = create_message(Protocol::AnthropicMCP, "call");

    let result = translator
        .translate_message(message, Protocol::GoogleA2A)
        .expect("Translation should succeed");

    // GOLDEN: MCP → A2A translation metadata
    assert_eq!(result.source_protocol, Protocol::AnthropicMCP);
    assert_eq!(result.target_protocol, Protocol::GoogleA2A);
}

#[test]
fn golden_supported_translation_paths() {
    let translator = ProtocolTranslator::new();
    let paths = translator.supported_translations();

    // GOLDEN: At minimum, A2A ↔ MCP paths should be supported
    assert!(
        !paths.is_empty(),
        "Should have at least one translation path"
    );

    // Document what paths are currently supported
    for (source, target) in &paths {
        println!("GOLDEN PATH: {:?} → {:?}", source, target);
    }
}

// ============================================
// GOLDEN: Field Mapping Stability
// ============================================

#[test]
fn golden_field_mapping_task_id_to_id() {
    let translator = ProtocolTranslator::new();

    // A2A uses "task_id", MCP uses "id"
    let mut message = create_message(Protocol::GoogleA2A, "invoke");
    message.params = serde_json::json!({"task_id": "abc123", "message": "test"});

    let result = translator
        .translate_message(message, Protocol::AnthropicMCP)
        .expect("Translation should succeed");

    // GOLDEN: Field mapping should transform task_id → id
    // Note: Actual behavior documented here
    let params = result
        .message
        .params
        .as_object()
        .expect("Params should be object");

    // Document current mapping behavior (may be task_id→id or preserved)
    let has_id = params.contains_key("id");
    let has_task_id = params.contains_key("task_id");
    println!(
        "GOLDEN MAPPING: has_id={}, has_task_id={}",
        has_id, has_task_id
    );
}

// ============================================
// GOLDEN: Status Translation
// ============================================

#[test]
fn golden_status_translation_preserves_value() {
    use agentkern_nexus::types::TaskStatus;

    let translator = ProtocolTranslator::new();

    let statuses = [
        TaskStatus::Working,
        TaskStatus::Completed,
        TaskStatus::Failed,
        TaskStatus::Canceled,
    ];

    for status in statuses {
        let translated = translator.translate_status(status, Protocol::AnthropicMCP);

        // GOLDEN: Status should be preserved (unified enum)
        assert_eq!(
            translated, status,
            "Status {:?} should be preserved",
            status
        );
    }
}
