//! Nexus Core Types
//!
//! Unified message format that can represent any agent protocol message.
//! Designed for extensibility - new protocols just need to implement translation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Supported agent protocols.
/// 
/// # Extensibility
/// New protocols can be added here. The `Custom` variant allows for
/// runtime-registered protocols without recompilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// AgentKern native protocol
    AgentKern,
    /// Google A2A Protocol (Agent-to-Agent)
    #[serde(rename = "a2a")]
    GoogleA2A,
    /// Anthropic Model Context Protocol
    #[serde(rename = "mcp")]
    AnthropicMCP,
    /// IBM Agent Communication Protocol
    #[serde(rename = "acp")]
    IbmACP,
    /// W3C Agent Network Protocol (DID-based)
    #[serde(rename = "anp")]
    W3cANP,
    /// ECMA Natural Language Interaction Protocol
    #[serde(rename = "nlip")]
    EcmaNLIP,
    /// NEAR Agent Interaction and Transaction Protocol
    #[serde(rename = "aitp")]
    NearAITP,
    /// Custom/future protocol (identified by string)
    Custom(u32),
}

impl Protocol {
    /// Get human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::AgentKern => "AgentKern Native",
            Self::GoogleA2A => "Google A2A",
            Self::AnthropicMCP => "Anthropic MCP",
            Self::IbmACP => "IBM ACP",
            Self::W3cANP => "W3C ANP",
            Self::EcmaNLIP => "ECMA NLIP",
            Self::NearAITP => "NEAR AITP",
            Self::Custom(_) => "Custom",
        }
    }

    /// Check if protocol supports transactions.
    pub fn supports_transactions(&self) -> bool {
        matches!(self, Self::NearAITP)
    }

    /// Check if protocol uses DID.
    pub fn uses_did(&self) -> bool {
        matches!(self, Self::W3cANP)
    }
}

/// Unified message format for cross-protocol communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusMessage {
    /// Unique message ID
    pub id: String,
    /// JSON-RPC style method
    pub method: String,
    /// Parameters (protocol-specific, normalized)
    pub params: serde_json::Value,
    /// Source protocol
    pub source_protocol: Protocol,
    /// Source agent ID
    pub source_agent: Option<String>,
    /// Target agent ID (if known)
    pub target_agent: Option<String>,
    /// Correlation ID for request/response matching
    pub correlation_id: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Metadata (headers, auth tokens, etc.)
    pub metadata: HashMap<String, serde_json::Value>,
}

impl NexusMessage {
    /// Create a new message.
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            method: method.into(),
            params,
            source_protocol: Protocol::AgentKern,
            source_agent: None,
            target_agent: None,
            correlation_id: None,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set source agent.
    pub fn from_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.source_agent = Some(agent_id.into());
        self
    }

    /// Set target agent.
    pub fn to_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.target_agent = Some(agent_id.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Create a response to this message.
    pub fn respond(&self, result: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            method: format!("{}/response", self.method),
            params: result,
            source_protocol: self.source_protocol,
            source_agent: self.target_agent.clone(),
            target_agent: self.source_agent.clone(),
            correlation_id: Some(self.id.clone()),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for NexusMessage {
    fn default() -> Self {
        Self::new("ping", serde_json::Value::Null)
    }
}

/// Task representation for routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task type/category
    pub task_type: String,
    /// Required skills
    pub required_skills: Vec<String>,
    /// Task parameters
    pub params: serde_json::Value,
    /// Priority (0-100)
    pub priority: u8,
    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Status
    pub status: TaskStatus,
}

impl Task {
    /// Create a new task.
    pub fn new(task_type: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type: task_type.into(),
            required_skills: vec![],
            params,
            priority: 50,
            timeout_secs: None,
            created_at: Utc::now(),
            status: TaskStatus::Pending,
        }
    }

    /// Require specific skills.
    pub fn require_skills(mut self, skills: Vec<String>) -> Self {
        self.required_skills = skills;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(100);
        self
    }
}

/// Task status (A2A compatible).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Waiting to be assigned
    Pending,
    /// Submitted to an agent
    Submitted,
    /// Agent is working on it
    Working,
    /// Successfully completed
    Completed,
    /// Failed
    Failed,
    /// Canceled
    Canceled,
}

/// Modality for multi-modal agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Text,
    Audio,
    Video,
    Image,
    Code,
    File,
    Form,
}

/// Skill capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Skill ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Tags for matching
    pub tags: Vec<String>,
    /// Input schema (JSON Schema)
    pub input_schema: Option<serde_json::Value>,
    /// Output schema (JSON Schema)
    pub output_schema: Option<serde_json::Value>,
}

/// Agent capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Supported modalities
    pub input_modes: Vec<Modality>,
    /// Output modalities
    pub output_modes: Vec<Modality>,
    /// Rate limit (requests per minute)
    pub rate_limit: Option<u32>,
}

/// Skill category for agent capabilities.
/// Merged from arbiter/marketplace.rs during consolidation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    /// Text processing
    TextProcessing,
    /// Image processing
    ImageProcessing,
    /// Data analysis
    DataAnalysis,
    /// Code generation
    CodeGeneration,
    /// Research
    Research,
    /// Translation
    Translation,
    /// Summarization
    Summarization,
    /// Audio processing
    AudioProcessing,
    /// Video processing  
    VideoProcessing,
    /// Custom category
    Custom(String),
}

impl SkillCategory {
    /// Get human-readable name.
    pub fn name(&self) -> &str {
        match self {
            Self::TextProcessing => "Text Processing",
            Self::ImageProcessing => "Image Processing",
            Self::DataAnalysis => "Data Analysis",
            Self::CodeGeneration => "Code Generation",
            Self::Research => "Research",
            Self::Translation => "Translation",
            Self::Summarization => "Summarization",
            Self::AudioProcessing => "Audio Processing",
            Self::VideoProcessing => "Video Processing",
            Self::Custom(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_names() {
        assert_eq!(Protocol::GoogleA2A.name(), "Google A2A");
        assert_eq!(Protocol::AnthropicMCP.name(), "Anthropic MCP");
        assert!(Protocol::NearAITP.supports_transactions());
        assert!(Protocol::W3cANP.uses_did());
    }

    #[test]
    fn test_nexus_message() {
        let msg = NexusMessage::new("tasks/create", serde_json::json!({"name": "test"}))
            .from_agent("agent-A")
            .to_agent("agent-B")
            .with_metadata("auth", serde_json::json!("Bearer token"));
        
        assert_eq!(msg.method, "tasks/create");
        assert_eq!(msg.source_agent, Some("agent-A".into()));
        assert!(msg.metadata.contains_key("auth"));
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new("summarize", serde_json::json!({"text": "hello"}))
            .require_skills(vec!["nlp".into()])
            .with_priority(80);
        
        assert_eq!(task.priority, 80);
        assert_eq!(task.required_skills, vec!["nlp"]);
        assert_eq!(task.status, TaskStatus::Pending);
    }

    #[test]
    fn test_message_response() {
        let request = NexusMessage::new("echo", serde_json::json!({"msg": "hello"}))
            .from_agent("client");
        
        let response = request.respond(serde_json::json!({"msg": "hello back"}));
        
        assert_eq!(response.correlation_id, Some(request.id.clone()));
        assert_eq!(response.target_agent, request.source_agent);
    }
}
