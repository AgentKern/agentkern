#![cfg(feature = "anp")]
use agentkern_nexus::protocols::{ANPAdapter, ProtocolAdapter};
use agentkern_nexus::types::{NexusMessage, Protocol};
use serde_json::json;

#[tokio::test]
async fn test_anp_detection() {
    let adapter = ANPAdapter::new();

    let raw = json!({
        "type": "https://w3id.org/anp/1.0/propose",
        "id": "msg-123",
        "from": "did:example:alice",
        "body": {}
    })
    .to_string();

    assert!(adapter.detect(raw.as_bytes()));

    let invalid = json!({
        "type": "http://example.com/other",
    })
    .to_string();

    assert!(!adapter.detect(invalid.as_bytes()));
}

#[tokio::test]
async fn test_anp_parsing() {
    let adapter = ANPAdapter::new();

    let raw = json!({
        "type": "https://w3id.org/anp/1.0/propose",
        "id": "msg-123",
        "thread_id": "thread-456",
        "from": "did:example:alice",
        "to": "did:example:bob",
        "body": {
            "price": 100,
            "currency": "USD"
        }
    })
    .to_string();

    let msg = adapter
        .parse(raw.as_bytes())
        .await
        .expect("Failed to parse ANP");

    assert_eq!(msg.method, "anp.propose");
    assert_eq!(msg.source_protocol, Protocol::W3cANP);
    assert_eq!(msg.source_agent, Some("did:example:alice".into()));
    assert_eq!(msg.target_agent, Some("did:example:bob".into()));
    assert_eq!(msg.correlation_id, Some("thread-456".into()));

    let price = msg.params.get("price").and_then(|v: &serde_json::Value| v.as_i64()).unwrap();
    assert_eq!(price, 100);
}

#[tokio::test]
async fn test_anp_serialization() {
    let adapter = ANPAdapter::new();

    let msg = NexusMessage::new("anp.accept", json!({"status": "confirmed"}))
        .from_agent("did:example:bob")
        .to_agent("did:example:alice");

    // Simulate setting correlation ID via metadata or direct field if adapter supports it
    // The generic NexusMessage constructor puts correlation_id. ANPAdapter uses it.
    let mut msg = msg;
    msg.correlation_id = Some("thread-456".into());

    let bytes: Vec<u8> = adapter
        .serialize(&msg)
        .await
        .expect("Failed to serialize ANP");
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(json["type"], "https://w3id.org/anp/1.0/accept");
    assert_eq!(json["from"], "did:example:bob");
    assert_eq!(json["to"], "did:example:alice");
    assert_eq!(json["thread_id"], "thread-456");
    assert_eq!(json["body"]["status"], "confirmed");
}
