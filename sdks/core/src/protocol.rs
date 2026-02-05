//! A2A Protocol Module
//!
//! Agent-to-Agent (A2A) message encoding and parsing.
//! Designed for interoperability with the emerging A2A standard.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{SdkError, SdkResult};
use crate::proof::LiabilityProof;

/// A2A Message Types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Request an action
    Request,
    /// Response to a request
    Response,
    /// Notification (no response expected)
    Notification,
    /// Error message
    Error,
    /// Heartbeat/ping
    Ping,
    /// Heartbeat response
    Pong,
    /// Capability advertisement
    Capabilities,
}

/// A2A Message - Standard agent-to-agent communication format.
///
/// Compatible with the emerging A2A protocol specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    /// Protocol version
    pub version: String,
    /// Message type
    #[serde(rename = "type")]
    pub msg_type: MessageType,
    /// Unique message ID
    pub id: String,
    /// Sender agent ID (DID)
    pub from: String,
    /// Recipient agent ID (DID)
    pub to: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Liability proof (for authorized actions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof: Option<String>,
    /// Message payload (action-specific)
    pub payload: serde_json::Value,
    /// Thread ID for conversation tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    /// Reference to previous message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
}

impl A2AMessage {
    /// Create a new request message.
    pub fn request(from: &str, to: &str, payload: serde_json::Value) -> Self {
        Self {
            version: "1.0".to_string(),
            msg_type: MessageType::Request,
            id: Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: to.to_string(),
            timestamp: Utc::now(),
            proof: None,
            payload,
            thread_id: None,
            in_reply_to: None,
        }
    }

    /// Create a response to an existing message.
    pub fn response(request: &A2AMessage, from: &str, payload: serde_json::Value) -> Self {
        Self {
            version: "1.0".to_string(),
            msg_type: MessageType::Response,
            id: Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: request.from.clone(),
            timestamp: Utc::now(),
            proof: None,
            payload,
            thread_id: request
                .thread_id
                .clone()
                .or_else(|| Some(request.id.clone())),
            in_reply_to: Some(request.id.clone()),
        }
    }

    /// Create an error response.
    pub fn error(request: &A2AMessage, from: &str, code: &str, message: &str) -> Self {
        let payload = serde_json::json!({
            "error": {
                "code": code,
                "message": message,
            }
        });

        Self {
            version: "1.0".to_string(),
            msg_type: MessageType::Error,
            id: Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: request.from.clone(),
            timestamp: Utc::now(),
            proof: None,
            payload,
            thread_id: request
                .thread_id
                .clone()
                .or_else(|| Some(request.id.clone())),
            in_reply_to: Some(request.id.clone()),
        }
    }

    /// Create a notification (no response expected).
    pub fn notification(from: &str, to: &str, payload: serde_json::Value) -> Self {
        Self {
            version: "1.0".to_string(),
            msg_type: MessageType::Notification,
            id: Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: to.to_string(),
            timestamp: Utc::now(),
            proof: None,
            payload,
            thread_id: None,
            in_reply_to: None,
        }
    }

    /// Attach a liability proof to this message.
    pub fn with_proof(mut self, proof: &LiabilityProof) -> Self {
        self.proof = Some(proof.to_jwt().to_string());
        self
    }

    /// Start a new conversation thread.
    pub fn with_thread(mut self) -> Self {
        self.thread_id = Some(Uuid::new_v4().to_string());
        self
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> SdkResult<String> {
        serde_json::to_string(self).map_err(SdkError::from)
    }

    /// Serialize to pretty JSON string.
    pub fn to_json_pretty(&self) -> SdkResult<String> {
        serde_json::to_string_pretty(self).map_err(SdkError::from)
    }

    /// Parse from JSON string.
    pub fn from_json(json: &str) -> SdkResult<Self> {
        serde_json::from_str(json).map_err(SdkError::from)
    }

    /// Check if this message has a liability proof.
    pub fn has_proof(&self) -> bool {
        self.proof.is_some()
    }

    /// Extract and parse the liability proof (if present).
    pub fn extract_proof(&self) -> SdkResult<Option<LiabilityProof>> {
        match &self.proof {
            Some(jwt) => Ok(Some(LiabilityProof::from_jwt(jwt)?)),
            None => Ok(None),
        }
    }

    /// Check if this is a response to another message.
    pub fn is_response(&self) -> bool {
        self.in_reply_to.is_some()
    }

    /// Check if this message belongs to a thread.
    pub fn is_threaded(&self) -> bool {
        self.thread_id.is_some()
    }
}

/// A2A Capability Advertisement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Agent ID
    pub agent_id: String,
    /// Agent name
    pub name: String,
    /// Supported actions
    pub actions: Vec<String>,
    /// Protocol versions supported
    pub protocols: Vec<String>,
    /// Endpoint URL
    pub endpoint: Option<String>,
    /// Public key (for verification)
    pub public_key: String,
}

impl AgentCapabilities {
    /// Create a capabilities message.
    pub fn to_message(&self, from: &str, to: &str) -> A2AMessage {
        A2AMessage {
            version: "1.0".to_string(),
            msg_type: MessageType::Capabilities,
            id: Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: to.to_string(),
            timestamp: Utc::now(),
            proof: None,
            payload: serde_json::to_value(self).unwrap_or_default(),
            thread_id: None,
            in_reply_to: None,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_message() {
        let msg = A2AMessage::request(
            "did:key:zSender",
            "did:key:zRecipient",
            serde_json::json!({"action": "transfer", "amount": 100}),
        );

        assert_eq!(msg.msg_type, MessageType::Request);
        assert_eq!(msg.from, "did:key:zSender");
        assert_eq!(msg.to, "did:key:zRecipient");
        assert!(!msg.id.is_empty());
    }

    #[test]
    fn test_response_message() {
        let request = A2AMessage::request(
            "did:key:zSender",
            "did:key:zRecipient",
            serde_json::json!({}),
        );

        let response = A2AMessage::response(
            &request,
            "did:key:zRecipient",
            serde_json::json!({"status": "ok"}),
        );

        assert_eq!(response.msg_type, MessageType::Response);
        assert_eq!(response.in_reply_to, Some(request.id.clone()));
        assert_eq!(response.to, request.from);
    }

    #[test]
    fn test_json_roundtrip() {
        let msg = A2AMessage::request(
            "did:key:zFrom",
            "did:key:zTo",
            serde_json::json!({"test": true}),
        );

        let json = msg.to_json().unwrap();
        let parsed = A2AMessage::from_json(&json).unwrap();

        assert_eq!(parsed.id, msg.id);
        assert_eq!(parsed.msg_type, msg.msg_type);
    }

    #[test]
    fn test_thread() {
        let msg = A2AMessage::request("did:key:zFrom", "did:key:zTo", serde_json::json!({}))
            .with_thread();

        assert!(msg.is_threaded());
        assert!(msg.thread_id.is_some());
    }

    #[test]
    fn test_error_message() {
        let request = A2AMessage::request(
            "did:key:zSender",
            "did:key:zRecipient",
            serde_json::json!({}),
        );

        let error = A2AMessage::error(
            &request,
            "did:key:zRecipient",
            "UNAUTHORIZED",
            "Missing proof",
        );

        assert_eq!(error.msg_type, MessageType::Error);
        assert!(error.payload["error"]["code"].as_str() == Some("UNAUTHORIZED"));
    }
}
