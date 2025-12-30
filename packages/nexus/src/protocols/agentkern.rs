//! AgentKern Protocol Adapter - Native A2A/MCP translation
//!
//! This adapter handles the core AgentKern protocol for direct agent communication.

use super::adapter::ProtocolAdapter;
use crate::types::{NexusMessage, Protocol};
use crate::NexusError;

/// AgentKern native protocol adapter.
///
/// Handles direct A2A protocol messages without translation.
pub struct AgentKernAdapter;

impl AgentKernAdapter {
    /// Create a new AgentKern adapter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentKernAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ProtocolAdapter for AgentKernAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::AgentKern
    }

    fn version(&self) -> &'static str {
        "1.0"
    }

    fn detect(&self, data: &[u8]) -> bool {
        // Check for JSON-RPC 2.0 with A2A methods
        if let Ok(text) = std::str::from_utf8(data) {
            text.contains("jsonrpc") && text.contains("2.0")
        } else {
            false
        }
    }

    async fn parse(&self, data: &[u8]) -> Result<NexusMessage, NexusError> {
        // Parse A2A JSON-RPC message
        serde_json::from_slice(data).map_err(|e| NexusError::ParseError {
            message: e.to_string(),
        })
    }

    async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
        serde_json::to_vec(msg).map_err(|e| NexusError::SerializeError {
            message: e.to_string(),
        })
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}
