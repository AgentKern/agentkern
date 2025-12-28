//! Anthropic MCP Protocol Adapter
//!
//! Implements Anthropic's Model Context Protocol (June 2025 spec)
//!
//! Protocol Spec:
//! - Transport: stdio, HTTP, WebSocket
//! - Format: JSON-RPC 2.0
//! - Auth: OAuth 2.1
//! - Focus: LLM ↔ Tools connection

use super::ProtocolAdapter;
use crate::error::NexusError;
use crate::types::{NexusMessage, Protocol};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// MCP Protocol adapter.
pub struct MCPAdapter {
    version: &'static str,
}

impl MCPAdapter {
    /// Create a new MCP adapter.
    pub fn new() -> Self {
        Self {
            version: "2025-06-18",
        }
    }
}

impl Default for MCPAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProtocolAdapter for MCPAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::AnthropicMCP
    }

    fn version(&self) -> &'static str {
        self.version
    }

    fn detect(&self, raw: &[u8]) -> bool {
        if let Ok(text) = std::str::from_utf8(raw) {
            // MCP uses JSON-RPC 2.0 with specific methods
            if text.contains("\"jsonrpc\"") && text.contains("\"2.0\"") {
                // MCP-specific methods
                return text.contains("\"method\":\"tools/")
                    || text.contains("\"method\":\"resources/")
                    || text.contains("\"method\":\"prompts/")
                    || text.contains("\"method\":\"sampling/")
                    || text.contains("\"method\":\"initialize\"")
                    || text.contains("\"method\":\"ping\"");
            }
        }
        false
    }

    async fn parse(&self, raw: &[u8]) -> Result<NexusMessage, NexusError> {
        let text = std::str::from_utf8(raw).map_err(|e| NexusError::ParseError {
            message: e.to_string(),
        })?;

        let rpc: MCPJsonRpcMessage = serde_json::from_str(text)?;

        Ok(NexusMessage {
            id: rpc.id.map(|id| id.to_string()).unwrap_or_default(),
            method: rpc.method.unwrap_or_default(),
            params: rpc.params.unwrap_or(serde_json::Value::Null),
            source_protocol: Protocol::AnthropicMCP,
            source_agent: None,
            target_agent: None,
            correlation_id: None,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
        let rpc = MCPJsonRpcMessage {
            jsonrpc: "2.0".into(),
            id: Some(MCPId::String(msg.id.clone())),
            method: Some(msg.method.clone()),
            params: Some(msg.params.clone()),
            result: None,
            error: None,
        };

        serde_json::to_vec(&rpc).map_err(|e| NexusError::SerializeError {
            message: e.to_string(),
        })
    }

    fn supports_streaming(&self) -> bool {
        true // MCP supports streaming via WebSocket
    }
}

/// MCP JSON-RPC message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MCPJsonRpcMessage {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<MCPId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<MCPError>,
}

/// MCP ID can be string or number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum MCPId {
    String(String),
    Number(i64),
}

impl ToString for MCPId {
    fn to_string(&self) -> String {
        match self {
            MCPId::String(s) => s.clone(),
            MCPId::Number(n) => n.to_string(),
        }
    }
}

/// MCP Error.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MCPError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// MCP Tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP Resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// MCP Server capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<MCPToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<MCPResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<MCPPromptsCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolsCapability {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResourcesCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPPromptsCapability {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_detection() {
        let adapter = MCPAdapter::new();

        // Valid MCP message
        let valid = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        assert!(adapter.detect(valid.as_bytes()));

        // Initialize is also MCP
        let init = r#"{"jsonrpc":"2.0","id":"init","method":"initialize"}"#;
        assert!(adapter.detect(init.as_bytes()));

        // A2A message should not match
        let a2a = r#"{"jsonrpc":"2.0","method":"tasks/send"}"#;
        assert!(!adapter.detect(a2a.as_bytes()));
    }

    #[tokio::test]
    async fn test_mcp_parse() {
        let adapter = MCPAdapter::new();

        let msg = r#"{"jsonrpc":"2.0","id":"req-1","method":"tools/call","params":{"name":"get_weather"}}"#;
        let parsed = adapter.parse(msg.as_bytes()).await.unwrap();

        assert_eq!(parsed.method, "tools/call");
        assert_eq!(parsed.source_protocol, Protocol::AnthropicMCP);
    }

    #[tokio::test]
    async fn test_mcp_serialize() {
        let adapter = MCPAdapter::new();

        let msg = NexusMessage::new("tools/list", serde_json::json!({}));
        let serialized = adapter.serialize(&msg).await.unwrap();

        let text = String::from_utf8(serialized).unwrap();
        assert!(text.contains("tools/list"));
    }

    #[test]
    fn test_mcp_tool_schema() {
        let tool = MCPTool {
            name: "get_weather".into(),
            description: "Get current weather".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("get_weather"));
        assert!(json.contains("inputSchema"));
    }
}
