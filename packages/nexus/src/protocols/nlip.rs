//! ECMA NLIP Protocol Adapter
//!
//! Implements ECMA-430 Natural Language Interaction Protocol (Dec 2025)
//!
//! Per ECMA-430 (1st Edition, Dec 10, 2025):
//! - Multimodal message format (text, structured data, binary, location)
//! - Envelope protocol for universal AI agent communication
//! - Transport bindings: HTTP (ECMA-431), WebSocket (ECMA-432), AMQP (ECMA-433)
//! - Security profiles: ECMA-434
//!
//! This makes NLIP the third stable protocol after A2A and MCP.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::types::{Protocol, NexusMessage};
use crate::error::NexusError;
use super::ProtocolAdapter;

/// NLIP Protocol adapter (ECMA-430).
pub struct NLIPAdapter {
    version: &'static str,
}

impl NLIPAdapter {
    /// Create a new NLIP adapter.
    pub fn new() -> Self {
        Self { version: "1.0" }
    }
}

impl Default for NLIPAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProtocolAdapter for NLIPAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::EcmaNLIP
    }

    fn version(&self) -> &'static str {
        self.version
    }

    fn detect(&self, raw: &[u8]) -> bool {
        // NLIP uses JSON with specific envelope structure
        if let Ok(text) = std::str::from_utf8(raw) {
            // ECMA-430: Messages contain "nlip" version field
            // and use envelope with header/payload structure
            if text.contains("\"nlip\"") || text.contains("\"nlipVersion\"") {
                return true;
            }
            // Alternative: Check for NLIP-specific content types
            if text.contains("\"contentType\"") && 
               (text.contains("\"text/natural-language\"") ||
                text.contains("\"application/nlip\"")) {
                return true;
            }
            // Check for NLIP envelope structure
            if text.contains("\"envelope\"") && text.contains("\"header\"") &&
               text.contains("\"payload\"") {
                return true;
            }
        }
        false
    }

    async fn parse(&self, raw: &[u8]) -> Result<NexusMessage, NexusError> {
        let text = std::str::from_utf8(raw)
            .map_err(|e| NexusError::ParseError { message: e.to_string() })?;
        
        let envelope: NLIPEnvelope = serde_json::from_str(text)?;
        
        // Extract method from intent or use default
        let method = envelope.header.intent
            .unwrap_or_else(|| "nlip/message".to_string());
        
        // Build params from payload
        let params = serde_json::json!({
            "conversationId": envelope.header.conversation_id,
            "messageId": envelope.header.message_id,
            "timestamp": envelope.header.timestamp,
            "content": envelope.payload.content,
            "modality": envelope.payload.modality,
            "context": envelope.payload.context,
        });
        
        Ok(NexusMessage {
            id: envelope.header.message_id.unwrap_or_default(),
            method,
            params,
            source_protocol: Protocol::EcmaNLIP,
            source_agent: envelope.header.sender,
            target_agent: envelope.header.recipient,
            correlation_id: envelope.header.conversation_id,
            timestamp: chrono::Utc::now(),
            metadata: envelope.header.metadata.unwrap_or_default(),
        })
    }

    async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
        // Extract content from params
        let content = msg.params.get("content")
            .cloned()
            .or_else(|| Some(serde_json::json!([{
                "type": "text",
                "value": msg.params.get("text").unwrap_or(&serde_json::Value::Null)
            }])))
            .unwrap_or(serde_json::Value::Array(vec![]));
        
        let envelope = NLIPEnvelope {
            nlip_version: "1.0".to_string(),
            header: NLIPHeader {
                message_id: Some(msg.id.clone()),
                conversation_id: msg.correlation_id.clone(),
                timestamp: Some(msg.timestamp.to_rfc3339()),
                sender: msg.source_agent.clone(),
                recipient: msg.target_agent.clone(),
                intent: Some(msg.method.clone()),
                metadata: Some(msg.metadata.clone()),
            },
            payload: NLIPPayload {
                content,
                modality: Some("text".to_string()),
                context: None,
            },
        };
        
        serde_json::to_vec(&envelope)
            .map_err(|e| NexusError::SerializeError { message: e.to_string() })
    }

    fn supports_streaming(&self) -> bool {
        true // NLIP supports WebSocket streaming (ECMA-432)
    }
}

// ============================================================================
// ECMA-430 MESSAGE TYPES
// ============================================================================

/// NLIP Envelope (ECMA-430 core structure).
///
/// The envelope wraps all NLIP messages with header and payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLIPEnvelope {
    /// NLIP version (currently "1.0")
    #[serde(rename = "nlipVersion", default = "default_version")]
    pub nlip_version: String,
    /// Message header (routing, metadata)
    pub header: NLIPHeader,
    /// Message payload (content)
    pub payload: NLIPPayload,
}

fn default_version() -> String { "1.0".to_string() }

/// NLIP Header (routing and metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLIPHeader {
    /// Unique message identifier
    #[serde(rename = "messageId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Conversation/thread ID for multi-turn
    #[serde(rename = "conversationId", skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    /// ISO 8601 timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Sender agent/user ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
    /// Recipient agent/user ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    /// Intent/action (maps to method)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// NLIP Payload (multimodal content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLIPPayload {
    /// Multimodal content parts
    pub content: serde_json::Value,
    /// Primary modality (text, audio, video, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    /// Conversation context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<NLIPContext>,
}

/// NLIP Context (conversation state).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLIPContext {
    /// Previous messages in conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<serde_json::Value>>,
    /// System instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Variables/slots
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// NLIP Content Part (multimodal).
///
/// Per ECMA-430: supports text, binary, location, structured data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NLIPContent {
    /// Natural language text
    Text {
        value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        language: Option<String>,
    },
    /// Binary content (base64 encoded)
    Binary {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// Geolocation data
    Location {
        latitude: f64,
        longitude: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        altitude: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        accuracy: Option<f64>,
    },
    /// Structured data (JSON)
    Data {
        schema: Option<String>,
        value: serde_json::Value,
    },
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nlip_detection() {
        let adapter = NLIPAdapter::new();
        
        // Valid NLIP message with version
        let valid = r#"{"nlipVersion":"1.0","header":{},"payload":{"content":[]}}"#;
        assert!(adapter.detect(valid.as_bytes()));
        
        // Valid NLIP with envelope
        let envelope = r#"{"envelope":{"header":{},"payload":{}}}"#;
        assert!(adapter.detect(envelope.as_bytes()));
        
        // Not NLIP (A2A message)
        let a2a = r#"{"jsonrpc":"2.0","method":"tasks/send"}"#;
        assert!(!adapter.detect(a2a.as_bytes()));
    }

    #[tokio::test]
    async fn test_nlip_parse() {
        let adapter = NLIPAdapter::new();
        
        let msg = r#"{
            "nlipVersion": "1.0",
            "header": {
                "messageId": "msg-123",
                "conversationId": "conv-456",
                "sender": "agent-a",
                "recipient": "agent-b",
                "intent": "query"
            },
            "payload": {
                "content": [{"type": "text", "value": "Hello, agent!"}],
                "modality": "text"
            }
        }"#;
        
        let parsed = adapter.parse(msg.as_bytes()).await.unwrap();
        
        assert_eq!(parsed.id, "msg-123");
        assert_eq!(parsed.method, "query");
        assert_eq!(parsed.source_protocol, Protocol::EcmaNLIP);
        assert_eq!(parsed.source_agent, Some("agent-a".to_string()));
        assert_eq!(parsed.target_agent, Some("agent-b".to_string()));
    }

    #[tokio::test]
    async fn test_nlip_serialize() {
        let adapter = NLIPAdapter::new();
        
        let mut msg = NexusMessage::new("nlip/respond", serde_json::json!({
            "content": [{"type": "text", "value": "Response text"}]
        }));
        msg.source_agent = Some("bot-1".to_string());
        
        let serialized = adapter.serialize(&msg).await.unwrap();
        let text = String::from_utf8(serialized).unwrap();
        
        assert!(text.contains("nlipVersion"));
        assert!(text.contains("header"));
        assert!(text.contains("payload"));
    }

    #[test]
    fn test_nlip_content_types() {
        // Text content
        let text = NLIPContent::Text {
            value: "Hello".to_string(),
            language: Some("en".to_string()),
        };
        let json = serde_json::to_string(&text).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        
        // Location content
        let loc = NLIPContent::Location {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: None,
            accuracy: Some(10.0),
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(json.contains("\"type\":\"location\""));
    }

    #[test]
    fn test_adapter_version() {
        let adapter = NLIPAdapter::new();
        assert_eq!(adapter.version(), "1.0");
        assert!(adapter.supports_streaming());
    }
}
