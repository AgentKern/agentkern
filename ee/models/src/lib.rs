//! Frontier Model Adapters
//!
//! Unified interface for frontier AI models (Nova, Claude, GPT, Gemini)
//! Technology-focused, vendor-neutral design
//! 
//! Graceful Degradation: Works with credentials, demo mode without

pub mod adapter;
pub mod cost_optimizer;
pub mod demo;

pub use adapter::{FrontierModel, ModelConfig, ModelResponse, InferenceRequest, ModelFamily};
pub use cost_optimizer::{ThinkingBudget, CostOptimizer};
pub use demo::{DemoModel, ModelFactory};

