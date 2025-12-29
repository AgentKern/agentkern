//! Google A2A Protocol Adapter
//!
//! Implements Google's Agent-to-Agent Protocol (v0.3, July 2025)
//!
//! Protocol Spec:
//! - Transport: HTTP/HTTPS + SSE + gRPC
//! - Format: JSON-RPC 2.0
//! - Discovery: Agent Cards at /.well-known/agent.json
//! - Task lifecycle: submitted → working → completed/failed/canceled

use super::ProtocolAdapter;
use crate::error::NexusError;
use crate::types::{NexusMessage, Protocol, TaskStatus};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A2A Protocol adapter.
pub struct A2AAdapter {
    version: &'static str,
}

impl A2AAdapter {
    /// Create a new A2A adapter.
    pub fn new() -> Self {
        Self { version: "0.3" }
    }
}

impl Default for A2AAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProtocolAdapter for A2AAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::GoogleA2A
    }

    fn version(&self) -> &'static str {
        self.version
    }

    fn detect(&self, raw: &[u8]) -> bool {
        // A2A uses JSON-RPC 2.0 with specific methods
        if let Ok(text) = std::str::from_utf8(raw) {
            // Check for JSON-RPC 2.0 with A2A-specific methods
            if text.contains("\"jsonrpc\"") && text.contains("\"2.0\"") {
                // A2A methods start with known prefixes
                return text.contains("\"method\":\"tasks/")
                    || text.contains("\"method\":\"agents/")
                    || text.contains("\"method\":\"messages/");
            }
        }
        false
    }

    async fn parse(&self, raw: &[u8]) -> Result<NexusMessage, NexusError> {
        let text = std::str::from_utf8(raw).map_err(|e| NexusError::ParseError {
            message: e.to_string(),
        })?;

        let rpc: A2AJsonRpcMessage = serde_json::from_str(text)?;

        Ok(NexusMessage {
            id: rpc.id.unwrap_or_default(),
            method: rpc.method,
            params: rpc.params.unwrap_or(serde_json::Value::Null),
            source_protocol: Protocol::GoogleA2A,
            source_agent: None,
            target_agent: None,
            correlation_id: None,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
        let rpc = A2AJsonRpcMessage {
            jsonrpc: "2.0".into(),
            id: Some(msg.id.clone()),
            method: msg.method.clone(),
            params: Some(msg.params.clone()),
        };

        serde_json::to_vec(&rpc).map_err(|e| NexusError::SerializeError {
            message: e.to_string(),
        })
    }

    fn supports_streaming(&self) -> bool {
        true // A2A supports SSE
    }
}

/// A2A JSON-RPC message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct A2AJsonRpcMessage {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

/// A2A Task message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ATask {
    pub id: String,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    pub status: A2ATaskState,
    pub message: Option<A2AMessage>,
    pub artifacts: Vec<A2AArtifact>,
}

/// A2A Task state (matches spec).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum A2ATaskState {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Failed,
    Canceled,
}

impl From<A2ATaskState> for TaskStatus {
    fn from(state: A2ATaskState) -> Self {
        match state {
            A2ATaskState::Submitted => TaskStatus::Submitted,
            A2ATaskState::Working => TaskStatus::Working,
            A2ATaskState::InputRequired => TaskStatus::Working,
            A2ATaskState::Completed => TaskStatus::Completed,
            A2ATaskState::Failed => TaskStatus::Failed,
            A2ATaskState::Canceled => TaskStatus::Canceled,
        }
    }
}

/// A2A Message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub role: String,
    pub parts: Vec<A2APart>,
}

/// A2A Message part (multi-modal).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum A2APart {
    Text { text: String },
    File { file: A2AFileData },
    Data { data: serde_json::Value },
}

/// A2A File data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AFileData {
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

/// A2A Artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AArtifact {
    pub name: String,
    pub parts: Vec<A2APart>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a2a_detection() {
        let adapter = A2AAdapter::new();

        // Valid A2A message
        let valid = r#"{"jsonrpc":"2.0","id":"1","method":"tasks/send","params":{}}"#;
        assert!(adapter.detect(valid.as_bytes()));

        // Not A2A (different method)
        let invalid = r#"{"jsonrpc":"2.0","method":"other/method"}"#;
        assert!(!adapter.detect(invalid.as_bytes()));
    }

    #[tokio::test]
    async fn test_a2a_parse() {
        let adapter = A2AAdapter::new();

        let msg =
            r#"{"jsonrpc":"2.0","id":"req-1","method":"tasks/send","params":{"task":"hello"}}"#;
        let parsed = adapter.parse(msg.as_bytes()).await.unwrap();

        assert_eq!(parsed.method, "tasks/send");
        assert_eq!(parsed.id, "req-1");
        assert_eq!(parsed.source_protocol, Protocol::GoogleA2A);
    }

    #[tokio::test]
    async fn test_a2a_serialize() {
        let adapter = A2AAdapter::new();

        let msg = NexusMessage::new("tasks/create", serde_json::json!({"name": "test"}));
        let serialized = adapter.serialize(&msg).await.unwrap();

        let text = String::from_utf8(serialized).unwrap();
        assert!(text.contains("\"jsonrpc\":\"2.0\""));
        assert!(text.contains("tasks/create"));
    }

    #[test]
    fn test_task_state_conversion() {
        assert!(matches!(
            TaskStatus::from(A2ATaskState::Completed),
            TaskStatus::Completed
        ));
        assert!(matches!(
            TaskStatus::from(A2ATaskState::Failed),
            TaskStatus::Failed
        ));
    }
}
