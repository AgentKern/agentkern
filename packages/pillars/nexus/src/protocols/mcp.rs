//! Nexus Adapter for Anthropic's Model Context Protocol (MCP)
//!
//! MCP is a JSON-RPC 2.0 based protocol for connecting AI models to external tools and context.
//! Spec: https://modelcontextprotocol.io/

use super::ProtocolAdapter;
use crate::types::{NexusMessage, Protocol};
use crate::error::NexusError;
use serde_json::Value;

pub struct McpAdapter;

impl McpAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for McpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ProtocolAdapter for McpAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::AnthropicMCP
    }

    fn detect(&self, data: &[u8]) -> bool {
        // MCP is JSON-RPC 2.0. We look for specific MCP methods.
        if let Ok(text) = std::str::from_utf8(data) {
            text.contains("jsonrpc") && 
            (text.contains("mcp.") || text.contains("prompts/") || text.contains("resources/"))
        } else {
            false
        }
    }

    async fn parse(&self, data: &[u8]) -> Result<NexusMessage, NexusError> {
        let v: Value = serde_json::from_slice(data)?;
        
        let method = v.get("method").and_then(|m| m.as_str()).unwrap_or("unknown").to_string();
        let params = v.get("params").cloned().unwrap_or(Value::Null);
        let id_val = v.get("id");
        
        // Construct NexusMessage
        let mut msg = NexusMessage::new(method, params);
        msg.source_protocol = Protocol::AnthropicMCP;

        // If ID present, treat as correlation ID
        if let Some(id) = id_val {
            if id.is_string() {
                 msg.correlation_id = id.as_str().map(|s| s.to_string());
            } else if id.is_number() {
                 msg.correlation_id = Some(id.to_string());
            }
        }
        
        Ok(msg)
    }

    async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
        // Convert back to JSON-RPC format
        let json_rpc = serde_json::json!({
            "jsonrpc": "2.0",
            "method": msg.method,
            "params": msg.params,
            "id": msg.correlation_id
        });
        
        serde_json::to_vec(&json_rpc).map_err(|e| NexusError::SerializeError { message: e.to_string() })
    }
}
