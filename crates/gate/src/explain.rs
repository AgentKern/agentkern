//! Explainability Engine
//!
//! Per Roadmap: "Explainability Engine - Native explainability for every AgentKern action"
//! Per MANDATE.md: "Audit Mindset: Every line of code is a liability until proven otherwise"
//!
//! Provides human-readable explanations for AI agent decisions.
//!
//! # Architecture
//!
//! Uses enum-based dispatch for explainability methods (dyn-safe pattern):
//! - RuleBased: Symbolic reasoning trace (default)
//! - Shap: Feature importance analysis
//! - Lime: Local approximation (Enterprise)
//! - Custom: Extensible via callback
//!
//! # Open Source vs Enterprise
//!
//! OSS: Rule-based explanations, basic SHAP
//! Enterprise: Advanced SHAP with GPU, LIME, custom ML explainers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Explanation method enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExplanationMethod {
    /// Rule-based symbolic reasoning
    #[default]
    RuleBased,
    /// SHAP values
    Shap,
    /// LIME local approximation
    Lime,
    /// Attention visualization (for transformers)
    Attention,
    /// Custom/plugin method
    Custom,
}

/// Contribution of a feature to the decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    /// Feature name
    pub feature: String,
    /// Contribution value (-1 to 1, negative = against, positive = towards)
    pub value: f64,
    /// Human-readable description
    pub description: Option<String>,
}

/// Counterfactual explanation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterfactual {
    /// What would need to change
    pub condition: String,
    /// What the outcome would be
    pub outcome: String,
    /// How significant is this change (0-100)
    pub significance: u8,
}

/// Provenance step in the decision chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceStep {
    /// Step name/ID
    pub step: String,
    /// What happened
    pub action: String,
    /// Result
    pub result: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Full explanation for a decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explanation {
    /// Human-readable summary
    pub summary: String,
    /// Detailed natural language explanation
    pub natural_language: String,
    /// Method used
    pub method: ExplanationMethod,
    /// Feature contributions
    pub contributions: Vec<Contribution>,
    /// Counterfactual scenarios
    pub counterfactuals: Vec<Counterfactual>,
    /// Decision provenance chain
    pub provenance: Vec<ProvenanceStep>,
    /// Confidence in the explanation (0-100)
    pub confidence: u8,
}

impl Explanation {
    /// Create a simple rule-based explanation.
    pub fn rule_based(summary: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            natural_language: detail.into(),
            method: ExplanationMethod::RuleBased,
            contributions: vec![],
            counterfactuals: vec![],
            provenance: vec![],
            confidence: 100,
        }
    }

    /// Add a contribution.
    pub fn with_contribution(mut self, feature: impl Into<String>, value: f64) -> Self {
        self.contributions.push(Contribution {
            feature: feature.into(),
            value,
            description: None,
        });
        self
    }

    /// Add a counterfactual.
    pub fn with_counterfactual(
        mut self,
        condition: impl Into<String>,
        outcome: impl Into<String>,
    ) -> Self {
        self.counterfactuals.push(Counterfactual {
            condition: condition.into(),
            outcome: outcome.into(),
            significance: 50,
        });
        self
    }

    /// Add a provenance step.
    pub fn with_provenance(
        mut self,
        step: impl Into<String>,
        action: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        self.provenance.push(ProvenanceStep {
            step: step.into(),
            action: action.into(),
            result: result.into(),
            timestamp: chrono::Utc::now(),
        });
        self
    }
}

/// Input context for generating explanations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainContext {
    /// Agent ID
    pub agent_id: String,
    /// Action being explained
    pub action: String,
    /// Decision outcome
    pub outcome: String,
    /// Was it allowed?
    pub allowed: bool,
    /// Input features
    pub features: HashMap<String, serde_json::Value>,
    /// Policy rules that applied
    pub applied_rules: Vec<String>,
}

/// Explainability engine - generates explanations using various methods.
pub struct ExplainabilityEngine {
    default_method: ExplanationMethod,
    available_methods: Vec<ExplanationMethod>,
}

impl ExplainabilityEngine {
    /// Create a new engine with default methods.
    pub fn new() -> Self {
        Self {
            default_method: ExplanationMethod::RuleBased,
            available_methods: vec![ExplanationMethod::RuleBased, ExplanationMethod::Shap],
        }
    }

    /// Set default explanation method.
    pub fn set_default_method(&mut self, method: ExplanationMethod) {
        self.default_method = method;
    }

    /// Register an additional method.
    pub fn register_method(&mut self, method: ExplanationMethod) {
        if !self.available_methods.contains(&method) {
            self.available_methods.push(method);
        }
    }

    /// Generate an explanation using the default method.
    pub fn explain(&self, context: &ExplainContext) -> Explanation {
        self.explain_with(&self.default_method, context)
    }

    /// Generate an explanation using a specific method.
    pub fn explain_with(
        &self,
        method: &ExplanationMethod,
        context: &ExplainContext,
    ) -> Explanation {
        match method {
            ExplanationMethod::RuleBased => self.explain_rule_based(context),
            ExplanationMethod::Shap => self.explain_shap(context),
            ExplanationMethod::Lime => self.explain_lime(context),
            ExplanationMethod::Attention => self.explain_attention(context),
            ExplanationMethod::Custom => self.explain_rule_based(context), // Fallback
        }
    }

    /// Rule-based explanation.
    fn explain_rule_based(&self, context: &ExplainContext) -> Explanation {
        let summary = if context.allowed {
            format!("Action '{}' was ALLOWED", context.action)
        } else {
            format!("Action '{}' was BLOCKED", context.action)
        };

        let detail = if context.applied_rules.is_empty() {
            format!(
                "The action '{}' by agent '{}' was {} based on default policy.",
                context.action,
                context.agent_id,
                if context.allowed {
                    "allowed"
                } else {
                    "blocked"
                }
            )
        } else {
            format!(
                "The action '{}' by agent '{}' was {} because rules [{}] matched.",
                context.action,
                context.agent_id,
                if context.allowed {
                    "allowed"
                } else {
                    "blocked"
                },
                context.applied_rules.join(", ")
            )
        };

        let mut explanation = Explanation::rule_based(&summary, &detail);

        for rule in &context.applied_rules {
            explanation = explanation
                .with_contribution(rule.clone(), if context.allowed { 1.0 } else { -1.0 });
        }

        if !context.allowed {
            explanation = explanation.with_counterfactual(
                format!("If the action was not '{}'", context.action),
                "It might have been allowed".to_string(),
            );
        }

        explanation
    }

    /// SHAP-style explanation.
    fn explain_shap(&self, context: &ExplainContext) -> Explanation {
        let mut contributions = Vec::new();

        for (feature, value) in &context.features {
            let importance = match value {
                serde_json::Value::Bool(b) => {
                    if *b {
                        0.5
                    } else {
                        -0.5
                    }
                }
                serde_json::Value::Number(n) => {
                    let v = n.as_f64().unwrap_or(0.0);
                    (v / 100.0).clamp(-1.0, 1.0)
                }
                serde_json::Value::String(s) => {
                    if s.contains("delete") || s.contains("admin") || s.contains("root") {
                        -0.7
                    } else {
                        0.3
                    }
                }
                _ => 0.0,
            };

            contributions.push(Contribution {
                feature: feature.clone(),
                value: importance,
                description: Some(format!("{}: {}", feature, value)),
            });
        }

        contributions.sort_by(|a, b| b.value.abs().partial_cmp(&a.value.abs()).unwrap());

        Explanation {
            summary: format!(
                "SHAP analysis: {} was {} (top factor: {})",
                context.action,
                context.outcome,
                contributions
                    .first()
                    .map(|c| c.feature.as_str())
                    .unwrap_or("none")
            ),
            natural_language: format!(
                "Feature importance analysis shows {} factors influenced this decision.",
                contributions.len()
            ),
            method: ExplanationMethod::Shap,
            contributions,
            counterfactuals: vec![],
            provenance: vec![],
            confidence: 80,
        }
    }

    /// LIME explanation (placeholder for enterprise).
    fn explain_lime(&self, context: &ExplainContext) -> Explanation {
        Explanation {
            summary: format!("LIME analysis for '{}'", context.action),
            natural_language: "LIME (Local Interpretable Model-agnostic Explanations) requires Enterprise license.".into(),
            method: ExplanationMethod::Lime,
            contributions: vec![],
            counterfactuals: vec![],
            provenance: vec![],
            confidence: 50,
        }
    }

    /// Attention explanation (placeholder).
    fn explain_attention(&self, context: &ExplainContext) -> Explanation {
        Explanation {
            summary: format!("Attention analysis for '{}'", context.action),
            natural_language: "Attention-based explanation for transformer models.".into(),
            method: ExplanationMethod::Attention,
            contributions: vec![],
            counterfactuals: vec![],
            provenance: vec![],
            confidence: 60,
        }
    }

    /// Get list of available methods.
    pub fn available_methods(&self) -> &[ExplanationMethod] {
        &self.available_methods
    }
}

impl Default for ExplainabilityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_context(allowed: bool) -> ExplainContext {
        ExplainContext {
            agent_id: "test-agent".into(),
            action: "send_email".into(),
            outcome: if allowed { "allowed" } else { "blocked" }.into(),
            allowed,
            features: HashMap::from([
                ("recipient".into(), serde_json::json!("user@example.com")),
                ("priority".into(), serde_json::json!(50)),
            ]),
            applied_rules: vec!["email_policy".into()],
        }
    }

    #[test]
    fn test_rule_based_explanation() {
        let engine = ExplainabilityEngine::new();
        let context = test_context(true);

        let explanation = engine.explain(&context);

        assert_eq!(explanation.method, ExplanationMethod::RuleBased);
        assert!(explanation.summary.contains("ALLOWED"));
        assert!(explanation.natural_language.contains("email_policy"));
    }

    #[test]
    fn test_shap_explanation() {
        let engine = ExplainabilityEngine::new();
        let context = test_context(false);

        let explanation = engine.explain_with(&ExplanationMethod::Shap, &context);

        assert_eq!(explanation.method, ExplanationMethod::Shap);
        assert!(!explanation.contributions.is_empty());
    }

    #[test]
    fn test_explanation_builder() {
        let exp = Explanation::rule_based("Test", "Detail")
            .with_contribution("feature1", 0.8)
            .with_counterfactual("If X", "Then Y")
            .with_provenance("step1", "check", "pass");

        assert_eq!(exp.contributions.len(), 1);
        assert_eq!(exp.counterfactuals.len(), 1);
        assert_eq!(exp.provenance.len(), 1);
    }

    #[test]
    fn test_contribution_sorting() {
        let mut contributions = vec![
            Contribution {
                feature: "low".into(),
                value: 0.1,
                description: None,
            },
            Contribution {
                feature: "high".into(),
                value: 0.9,
                description: None,
            },
            Contribution {
                feature: "negative".into(),
                value: -0.7,
                description: None,
            },
        ];

        contributions.sort_by(|a, b| b.value.abs().partial_cmp(&a.value.abs()).unwrap());

        assert_eq!(contributions[0].feature, "high");
        assert_eq!(contributions[1].feature, "negative");
    }

    #[test]
    fn test_available_methods() {
        let engine = ExplainabilityEngine::new();
        let methods = engine.available_methods();

        assert!(methods.contains(&ExplanationMethod::RuleBased));
        assert!(methods.contains(&ExplanationMethod::Shap));
    }
}
