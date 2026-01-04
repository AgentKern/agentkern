//! Cost Optimizer for Frontier Models
//!
//! Thinking budget controls and cost optimization
//! Works with models that support adjustable reasoning depth

use serde::{Deserialize, Serialize};
use super::adapter::{ThinkingLevel, ModelFamily, InferenceRequest, CostEstimate};

/// Thinking budget configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBudget {
    /// Default thinking level
    pub default_level: ThinkingLevel,
    /// Max cost per request (USD)
    pub max_cost_per_request: f64,
    /// Auto-adjust based on task complexity
    pub auto_adjust: bool,
    /// Complexity detection keywords
    pub high_complexity_keywords: Vec<String>,
}

impl Default for ThinkingBudget {
    fn default() -> Self {
        Self {
            default_level: ThinkingLevel::Medium,
            max_cost_per_request: 0.10,
            auto_adjust: true,
            high_complexity_keywords: vec![
                "analyze".into(),
                "plan".into(),
                "debug".into(),
                "optimize".into(),
                "security".into(),
            ],
        }
    }
}

/// Cost optimizer for model routing and budget control.
pub struct CostOptimizer {
    budget: ThinkingBudget,
    /// Model costs by family (input, output per 1K tokens)
    model_costs: std::collections::HashMap<ModelFamily, (f64, f64)>,
}

impl CostOptimizer {
    /// Create new optimizer with budget.
    pub fn new(budget: ThinkingBudget) -> Self {
        let mut model_costs = std::collections::HashMap::new();
        
        // Approximate costs per 1K tokens (USD)
        model_costs.insert(ModelFamily::Nova, (0.0008, 0.0032));      // Nova 2 Lite
        model_costs.insert(ModelFamily::Claude, (0.003, 0.015));       // Claude 3 Sonnet
        model_costs.insert(ModelFamily::Gpt, (0.005, 0.015));          // GPT-4
        model_costs.insert(ModelFamily::Gemini, (0.00025, 0.0005));    // Gemini Pro
        model_costs.insert(ModelFamily::Llama, (0.0001, 0.0001));      // Self-hosted
        model_costs.insert(ModelFamily::Mistral, (0.0002, 0.0006));    // Mistral Medium
        model_costs.insert(ModelFamily::Custom, (0.001, 0.001));       // Default
        
        Self { budget, model_costs }
    }
    
    /// Determine optimal thinking level for request.
    pub fn recommend_thinking_level(&self, request: &InferenceRequest) -> ThinkingLevel {
        if !self.budget.auto_adjust {
            return self.budget.default_level;
        }
        
        // Analyze task complexity
        let complexity = self.analyze_complexity(request);
        
        match complexity {
            0..=2 => ThinkingLevel::Low,
            3..=5 => ThinkingLevel::Medium,
            6..=8 => ThinkingLevel::High,
            _ => ThinkingLevel::Maximum,
        }
    }
    
    /// Analyze task complexity (0-10 scale).
    fn analyze_complexity(&self, request: &InferenceRequest) -> u32 {
        let mut score = 0u32;
        
        // Check message content for complexity keywords
        for msg in &request.messages {
            let text = match &msg.content {
                super::adapter::MessageContent::Text(t) => t.to_lowercase(),
                super::adapter::MessageContent::Multimodal(_) => {
                    score += 2; // Multimodal is more complex
                    continue;
                }
            };
            
            for keyword in &self.budget.high_complexity_keywords {
                if text.contains(keyword) {
                    score += 1;
                }
            }
        }
        
        // Tools increase complexity
        score += request.tools.len() as u32;
        
        // Long context increases complexity
        let message_count = request.messages.len();
        if message_count > 10 {
            score += 2;
        }
        
        score.min(10)
    }
    
    /// Estimate cost for request with given model.
    pub fn estimate_cost(&self, family: ModelFamily, request: &InferenceRequest) -> CostEstimate {
        let (input_cost, output_cost) = self.model_costs
            .get(&family)
            .copied()
            .unwrap_or((0.001, 0.001));
        
        // Estimate token counts
        let input_tokens = self.estimate_input_tokens(request);
        let output_tokens = request.max_tokens.unwrap_or(500);
        
        let cost = (input_tokens as f64 * input_cost / 1000.0) 
                 + (output_tokens as f64 * output_cost / 1000.0);
        
        // Apply thinking budget multiplier
        let multiplier = match request.thinking_budget.unwrap_or(ThinkingLevel::Medium) {
            ThinkingLevel::Low => 0.5,
            ThinkingLevel::Medium => 1.0,
            ThinkingLevel::High => 2.0,
            ThinkingLevel::Maximum => 4.0,
        };
        
        CostEstimate {
            input_tokens,
            estimated_output_tokens: output_tokens,
            estimated_cost_usd: cost * multiplier,
            confidence: 0.8,
        }
    }
    
    fn estimate_input_tokens(&self, request: &InferenceRequest) -> u32 {
        let mut tokens = 0u32;
        
        if let Some(sys) = &request.system {
            tokens += (sys.len() / 4) as u32;
        }
        
        for msg in &request.messages {
            match &msg.content {
                super::adapter::MessageContent::Text(t) => {
                    tokens += (t.len() / 4) as u32;
                }
                super::adapter::MessageContent::Multimodal(parts) => {
                    tokens += 500; // Base for multimodal
                    tokens += parts.len() as u32 * 100;
                }
            }
        }
        
        tokens
    }
    
    /// Should request be allowed (under budget)?
    pub fn within_budget(&self, estimate: &CostEstimate) -> bool {
        estimate.estimated_cost_usd <= self.budget.max_cost_per_request
    }
    
    /// Recommend cheapest model for request.
    pub fn recommend_cheapest(&self, request: &InferenceRequest) -> ModelFamily {
        let mut cheapest = ModelFamily::Custom;
        let mut min_cost = f64::MAX;
        
        for family in [ModelFamily::Llama, ModelFamily::Mistral, ModelFamily::Gemini, 
                       ModelFamily::Nova, ModelFamily::Claude, ModelFamily::Gpt] {
            let estimate = self.estimate_cost(family, request);
            if estimate.estimated_cost_usd < min_cost {
                min_cost = estimate.estimated_cost_usd;
                cheapest = family;
            }
        }
        
        cheapest
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::adapter::{Message, MessageRole, MessageContent};

    #[test]
    fn test_thinking_budget_default() {
        let budget = ThinkingBudget::default();
        assert_eq!(budget.default_level, ThinkingLevel::Medium);
        assert!(budget.auto_adjust);
    }

    #[test]
    fn test_cost_optimizer() {
        let optimizer = CostOptimizer::new(ThinkingBudget::default());
        
        let request = InferenceRequest {
            system: None,
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text("Hello".into()),
            }],
            temperature: None,
            max_tokens: Some(100),
            thinking_budget: None,
            tools: vec![],
            stop: vec![],
            response_format: None,
        };
        
        let estimate = optimizer.estimate_cost(ModelFamily::Nova, &request);
        assert!(estimate.estimated_cost_usd > 0.0);
    }

    #[test]
    fn test_complexity_analysis() {
        let optimizer = CostOptimizer::new(ThinkingBudget::default());
        
        // Simple request
        let simple = InferenceRequest {
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
        
        // Complex request
        let complex = InferenceRequest {
            system: None,
            messages: vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text("Please analyze and optimize this security plan".into()),
            }],
            temperature: None,
            max_tokens: None,
            thinking_budget: None,
            tools: vec![super::super::adapter::Tool {
                name: "code_review".into(),
                description: "Review code".into(),
                parameters: serde_json::json!({}),
            }],
            stop: vec![],
            response_format: None,
        };
        
        let simple_level = optimizer.recommend_thinking_level(&simple);
        let complex_level = optimizer.recommend_thinking_level(&complex);
        
        assert!(matches!(simple_level, ThinkingLevel::Low | ThinkingLevel::Medium));
        assert!(matches!(complex_level, ThinkingLevel::Medium | ThinkingLevel::High | ThinkingLevel::Maximum));
    }
}
