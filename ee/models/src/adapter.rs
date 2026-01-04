//! Frontier Model Adapter
//!
//! Generic trait for frontier AI models with cost controls
//! Supports: Nova, Claude, GPT, Gemini, Llama, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;

/// Frontier model trait - implement for each model family.
#[async_trait]
pub trait FrontierModel: Send + Sync {
    /// Model identifier.
    fn model_id(&self) -> &str;
    
    /// Model family (nova, claude, gpt, gemini, llama).
    fn family(&self) -> ModelFamily;
    
    /// Maximum context window (tokens).
    fn max_context(&self) -> usize;
    
    /// Run inference.
    async fn infer(&self, request: &InferenceRequest) -> Result<ModelResponse, ModelError>;
    
    /// Estimate cost before running.
    fn estimate_cost(&self, request: &InferenceRequest) -> CostEstimate;
    
    /// Check if model supports feature.
    fn supports(&self, capability: ModelCapability) -> bool;
}

/// Model family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelFamily {
    /// Amazon Nova (Lite, Pro, Omni, Sonic)
    Nova,
    /// Anthropic Claude
    Claude,
    /// OpenAI GPT
    Gpt,
    /// Google Gemini
    Gemini,
    /// Meta Llama
    Llama,
    /// Mistral
    Mistral,
    /// Custom/Self-hosted
    Custom,
}

/// Model capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelCapability {
    /// Text generation
    TextGeneration,
    /// Image understanding
    VisionInput,
    /// Image generation
    ImageGeneration,
    /// Audio/Speech input
    AudioInput,
    /// Speech output
    SpeechOutput,
    /// Video understanding
    VideoInput,
    /// Code execution
    CodeInterpreter,
    /// Tool/Function calling
    ToolUse,
    /// Adjustable reasoning depth
    ThinkingBudget,
    /// Long context (>100K tokens)
    LongContext,
}

/// Model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model ID (e.g., "nova-2-pro", "claude-3-opus")
    pub model_id: String,
    /// API endpoint
    pub endpoint: String,
    /// API key (or reference to secret)
    pub api_key_ref: String,
    /// Default temperature
    pub temperature: f32,
    /// Default max tokens
    pub max_tokens: u32,
    /// Cost per input token (USD)
    pub cost_per_input_token: f64,
    /// Cost per output token (USD)
    pub cost_per_output_token: f64,
    /// Rate limit (requests per minute)
    pub rate_limit_rpm: Option<u32>,
}

/// Inference request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// System prompt
    pub system: Option<String>,
    /// User messages
    pub messages: Vec<Message>,
    /// Temperature override
    pub temperature: Option<f32>,
    /// Max tokens override
    pub max_tokens: Option<u32>,
    /// Thinking budget (for models that support it)
    pub thinking_budget: Option<ThinkingLevel>,
    /// Tools available for the model
    pub tools: Vec<Tool>,
    /// Stop sequences
    pub stop: Vec<String>,
    /// Response format
    pub response_format: Option<ResponseFormat>,
}

/// Message in conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
}

/// Message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Multimodal(Vec<ContentPart>),
}

/// Content part for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text { text: String },
    Image { url: String, detail: Option<String> },
    Audio { url: String },
    Video { url: String },
}

/// Thinking level (for models with adjustable reasoning).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingLevel {
    /// Fast, less reasoning
    Low,
    /// Balanced
    Medium,
    /// Deep reasoning
    High,
    /// Maximum reasoning (more expensive)
    Maximum,
}

/// Tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Response format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseFormat {
    Text,
    Json,
    JsonSchema(serde_json::Value),
}

/// Model response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    /// Generated content
    pub content: String,
    /// Tool calls (if any)
    pub tool_calls: Vec<ToolCall>,
    /// Finish reason
    pub finish_reason: FinishReason,
    /// Usage statistics
    pub usage: Usage,
    /// Actual cost (USD)
    pub cost_usd: f64,
    /// Latency (ms)
    pub latency_ms: u64,
}

/// Tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Finish reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolUse,
    ContentFilter,
    Error,
}

/// Token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    /// Reasoning tokens (for thinking budget models)
    pub reasoning_tokens: Option<u32>,
}

/// Cost estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub input_tokens: u32,
    pub estimated_output_tokens: u32,
    pub estimated_cost_usd: f64,
    pub confidence: f32,
}

/// Model error.
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Context too long: {0} tokens")]
    ContextTooLong(usize),
    
    #[error("Content filtered")]
    ContentFiltered,
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Cost limit exceeded")]
    CostLimitExceeded,
    
    #[error("Capability not supported: {0:?}")]
    CapabilityNotSupported(ModelCapability),
    
    #[error("License error: {0}")]
    LicenseError(#[from] agentkern_connectors_ee::license::LicenseError),
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_family() {
        assert_eq!(ModelFamily::Nova, ModelFamily::Nova);
        assert_ne!(ModelFamily::Nova, ModelFamily::Claude);
    }

    #[test]
    fn test_thinking_level() {
        let level = ThinkingLevel::High;
        assert!(matches!(level, ThinkingLevel::High));
    }

    #[test]
    fn test_inference_request() {
        let request = InferenceRequest {
            system: Some("You are helpful".into()),
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text("Hello".into()),
            }],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            thinking_budget: None,
            tools: vec![],
            stop: vec![],
            response_format: None,
        };
        
        assert!(request.system.is_some());
    }
}
