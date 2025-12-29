//! AgentKern Native Protocol Adapter
//!
//! The native AgentKern protocol for internal agent communication.
//! Optimized for performance and AgentKern-specific features.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::types::{Protocol, NexusMessage};
use crate::error::NexusError;
use super::ProtocolAdapter;

/// AgentKern native protocol adapter.
pub struct AgentKernAdapter {
    version: &'static str,
}

impl AgentKernAdapter {
    /// Create a new AgentKern adapter.
    pub fn new() -> Self {
        Self { version: "1.0" }
    }
}

impl Default for AgentKernAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProtocolAdapter for AgentKernAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::AgentKern
    }

    fn version(&self) -> &'static str {
        self.version
    }

    fn detect(&self, raw: &[u8]) -> bool {
        if let Ok(text) = std::str::from_utf8(raw) {
            // AgentKern native uses specific header
            text.contains("\"protocol\":\"agentkern\"")
                || text.contains("\"vm_version\"")
                || text.starts_with("{\"vm_")
        } else {
            false
        }
    }

    async fn parse(&self, raw: &[u8]) -> Result<NexusMessage, NexusError> {
        let text = std::str::from_utf8(raw)
            .map_err(|e| NexusError::ParseError { message: e.to_string() })?;
        
        // Native format is just NexusMessage serialized
        let msg: NexusMessage = serde_json::from_str(text)?;
        Ok(msg)
    }

    async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
        serde_json::to_vec(msg)
            .map_err(|e| NexusError::SerializeError { message: e.to_string() })
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

/// AgentKern-specific message extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentKernExtension {
    /// Trust score of source agent
    pub trust_score: Option<u8>,
    /// Policy verification result
    pub policy_verified: Option<bool>,
    /// Carbon footprint in grams CO2
    pub carbon_grams: Option<f64>,
    /// Explainability data
    pub explanation: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agentkern_detection() {
        let adapter = AgentKernAdapter::new();
        
        let valid = r#"{"protocol":"agentkern","id":"1","method":"test"}"#;
        assert!(adapter.detect(valid.as_bytes()));
        
        let a2a = r#"{"jsonrpc":"2.0","method":"tasks/send"}"#;
        assert!(!adapter.detect(a2a.as_bytes()));
    }

    #[tokio::test]
    async fn test_agentkern_roundtrip() {
        let adapter = AgentKernAdapter::new();
        
        let original = NexusMessage::new("gate/verify", serde_json::json!({"action": "test"}))
            .from_agent("agent-1");
        
        let serialized = adapter.serialize(&original).await.unwrap();
        let parsed = adapter.parse(&serialized).await.unwrap();
        
        assert_eq!(parsed.method, original.method);
        assert_eq!(parsed.source_agent, original.source_agent);
    }
}
