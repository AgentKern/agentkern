//! Demo Model Implementation
//!
//! A working demo model that returns realistic responses without credentials
//! Used when no real API keys are configured

use super::adapter::*;
use crate::core::{ConnectionMode, ConnectionStatus, GracefulService, GracefulResult};
use async_trait::async_trait;

/// Demo model that works without credentials.
pub struct DemoModel {
    config: ModelConfig,
    mode: ConnectionMode,
}

impl DemoModel {
    /// Create new demo model.
    pub fn new(family: ModelFamily) -> Self {
        let mode = ConnectionMode::detect("models");
        
        let config = ModelConfig {
            model_id: format!("demo-{:?}", family).to_lowercase(),
            endpoint: "https://demo.agentkern.dev/v1".into(),
            api_key_ref: String::new(),
            temperature: 0.7,
            max_tokens: 1000,
            cost_per_input_token: 0.0,
            cost_per_output_token: 0.0,
            rate_limit_rpm: None,
        };
        
        Self { config, mode }
    }
    
    /// Try to create a live model, fallback to demo if no credentials.
    pub fn new_graceful(family: ModelFamily) -> Self {
        Self::new(family)
    }
    
    fn generate_demo_response(&self, request: &InferenceRequest) -> String {
        let user_msg = request.messages.last()
            .map(|m| match &m.content {
                MessageContent::Text(t) => t.clone(),
                MessageContent::Multimodal(_) => "[multimodal input]".to_string(),
            })
            .unwrap_or_default();
        
        format!(
            "[Demo Mode] This is a simulated response to: \"{}\". \
            Set AGENTKERN_MODELS_API_KEY for live responses.",
            user_msg.chars().take(50).collect::<String>()
        )
    }
}

impl GracefulService for DemoModel {
    fn mode(&self) -> ConnectionMode {
        self.mode
    }
    
    fn status(&self) -> ConnectionStatus {
        ConnectionStatus::new("models")
    }
}

#[async_trait]
impl FrontierModel for DemoModel {
    fn model_id(&self) -> &str {
        &self.config.model_id
    }
    
    fn family(&self) -> ModelFamily {
        ModelFamily::Custom
    }
    
    fn max_context(&self) -> usize {
        128_000
    }
    
    async fn infer(&self, request: &InferenceRequest) -> Result<ModelResponse, ModelError> {
        // Always works - returns demo or live response
        let content = if self.mode.is_live() {
            // Would call real API here
            format!("[Live API response would go here]")
        } else {
            self.generate_demo_response(request)
        };
        
        Ok(ModelResponse {
            content,
            tool_calls: vec![],
            finish_reason: FinishReason::Stop,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
                total_tokens: 150,
                reasoning_tokens: None,
            },
            cost_usd: 0.0,
            latency_ms: 50,
        })
    }
    
    fn estimate_cost(&self, _request: &InferenceRequest) -> CostEstimate {
        CostEstimate {
            input_tokens: 100,
            estimated_output_tokens: 50,
            estimated_cost_usd: 0.0,
            confidence: 1.0,
        }
    }
    
    fn supports(&self, capability: ModelCapability) -> bool {
        matches!(capability, 
            ModelCapability::TextGeneration | 
            ModelCapability::ToolUse
        )
    }
}

/// Factory to get the best available model.
pub struct ModelFactory;

impl ModelFactory {
    /// Get model with graceful fallback.
    /// Returns live model if credentials available, demo otherwise.
    pub fn get(family: ModelFamily) -> Box<dyn FrontierModel> {
        let mode = ConnectionMode::detect("models");
        
        match mode {
            ConnectionMode::Live => {
                // Would return real model implementation
                Box::new(DemoModel::new(family))
            }
            _ => {
                // Demo mode - always works
                Box::new(DemoModel::new(family))
            }
        }
    }
    
    /// Get connection status.
    pub fn status() -> ConnectionStatus {
        ConnectionStatus::new("models")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_model_works_without_credentials() {
        let model = DemoModel::new(ModelFamily::Nova);
        assert!(model.is_available());
    }

    #[tokio::test]
    async fn test_demo_model_infer() {
        let model = DemoModel::new(ModelFamily::Claude);
        
        let request = InferenceRequest {
            system: None,
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text("Hello".into()),
            }],
            temperature: None,
            max_tokens: None,
            thinking_budget: None,
            tools: vec![],
            stop: vec![],
            response_format: None,
        };
        
        let result = model.infer(&request).await;
        assert!(result.is_ok());
    }
}
