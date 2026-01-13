//! AgentKern-Gate: Core Verification Engine
//!
//! The heart of the Neuro-Symbolic verification system.
//!
//! Per ENGINEERING_STANDARD.md:
//! - Fast Path (Symbolic): <1ms
//! - Safety Path (Neural): <20ms (only when risk > threshold)

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::carbon::CarbonVeto;
use crate::dsl::{evaluate, EvalContext};
use crate::neural::NeuralScorer;
use crate::policy::{Policy, PolicyAction};
use crate::types::{
    DataRegion, LatencyBreakdown, VerificationContext, VerificationRequest, VerificationResult,
};
use agentkern_treasury::carbon::ComputeType;

/// The AgentKern Gate Engine.
///
/// Evaluates agent actions against registered policies using a
/// two-phase Neuro-Symbolic approach.
pub struct GateEngine {
    /// Registered policies
    policies: Arc<RwLock<HashMap<String, Policy>>>,
    /// Neural scorer for semantic analysis
    neural_scorer: NeuralScorer,
    /// Threshold for triggering neural path
    /// neural threshold
    neural_threshold: u8,
    /// Current jurisdiction
    jurisdiction: DataRegion,
    /// Carbon policy veto (optional)
    carbon_veto: Option<Arc<CarbonVeto>>,
    /// Prompt Injection Guard (Phase 12)
    prompt_guard: crate::prompt_guard::PromptGuard,
    /// Agent Budgets (Phase 12)
    budgets: Arc<RwLock<HashMap<String, crate::budget::AgentBudget>>>,
    /// Explainability Engine (Phase 12)
    explainability: crate::explain::ExplainabilityEngine,
}

impl Default for GateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl GateEngine {
    /// Create a new Gate Engine.
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            neural_scorer: NeuralScorer::new(),
            // Threshold 50: Medium-risk actions trigger neural evaluation
            neural_threshold: 50,
            jurisdiction: DataRegion::Global,
            carbon_veto: None,
            prompt_guard: crate::prompt_guard::PromptGuard::new(),
            budgets: Arc::new(RwLock::new(HashMap::new())),
            explainability: crate::explain::ExplainabilityEngine::new(),
        }
    }

    /// Set a budget for an agent.
    pub async fn set_budget(
        &self,
        agent_id: impl Into<String>,
        budget: crate::budget::AgentBudget,
    ) {
        let mut budgets = self.budgets.write().await;
        budgets.insert(agent_id.into(), budget);
    }

    /// Set the jurisdiction for policy filtering.
    pub fn with_jurisdiction(mut self, jurisdiction: DataRegion) -> Self {
        self.jurisdiction = jurisdiction;
        self
    }

    /// Set the threshold for triggering neural evaluation.
    pub fn with_neural_threshold(mut self, threshold: u8) -> Self {
        self.neural_threshold = threshold;
        self.neural_scorer = NeuralScorer::new().with_threshold(threshold);
        self
    }

    /// Set the carbon veto controller.
    pub fn with_carbon_veto(mut self, veto: CarbonVeto) -> Self {
        self.carbon_veto = Some(Arc::new(veto));
        self
    }

    /// Register a policy.
    pub async fn register_policy(&self, policy: Policy) {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy);
    }

    /// Remove a policy.
    pub async fn remove_policy(&self, policy_id: &str) -> Option<Policy> {
        let mut policies = self.policies.write().await;
        policies.remove(policy_id)
    }

    /// Get all registered policies.
    pub async fn get_policies(&self) -> Vec<Policy> {
        let policies = self.policies.read().await;
        policies.values().cloned().collect()
    }

    /// Explain a verification decision.
    pub fn explain(
        &self,
        request: &VerificationRequest,
        result: &VerificationResult,
    ) -> crate::explain::Explanation {
        let context = crate::explain::ExplainContext {
            agent_id: request.agent_id.clone(),
            action: request.action.clone(),
            outcome: if result.allowed {
                "allowed".to_string()
            } else {
                "blocked".to_string()
            },
            allowed: result.allowed,
            features: request.context.data.clone(),
            applied_rules: result.blocking_policies.clone(),
        };

        self.explainability.explain(&context)
    }

    /// Verify an action against all applicable policies.
    pub async fn verify(&self, request: VerificationRequest) -> VerificationResult {
        let start = Instant::now();

        // === PROMPT GUARD (Fast Security Check) ===
        // Phase 12: AI-Native Defense
        // Check for prompt injection attacks explicitly
        let prompt_start = Instant::now();
        let prompt_analysis = self.prompt_guard.analyze(&request.action);
        let prompt_us = prompt_start.elapsed().as_micros() as u64;

        if prompt_analysis.threat_level >= crate::prompt_guard::ThreatLevel::High {
            tracing::warn!(
                request_id = %request.request_id,
                agent_id = %request.agent_id,
                action = %request.action,
                attacks = ?prompt_analysis.attacks,
                "Prompt Injection Detected"
            );

            return VerificationResult {
                request_id: request.request_id,
                allowed: false,
                evaluated_policies: vec![],
                blocking_policies: vec!["prompt-guard".to_string()],
                symbolic_risk_score: 100,
                neural_risk_score: Some(100),
                final_risk_score: 100,
                reasoning: format!("Blocked by Prompt Guard: {:?}", prompt_analysis.attacks),
                latency: LatencyBreakdown {
                    total_us: start.elapsed().as_micros() as u64,
                    symbolic_us: prompt_us, // Count guard as symbolic/fast
                    neural_us: None,
                },
            };
        }

        // === AGENT BUDGET (Resource Limit Check) ===
        // Phase 12: AI-Native Defense
        // Enforce token/API limits
        {
            let mut budgets = self.budgets.write().await;
            if let Some(budget) = budgets.get_mut(&request.agent_id) {
                // Consume 1 API call for the verification itself
                if let Err(e) = budget.consume_api_call() {
                    return VerificationResult {
                        request_id: request.request_id,
                        allowed: false,
                        evaluated_policies: vec![],
                        blocking_policies: vec!["budget-limit".to_string()],
                        symbolic_risk_score: 100,
                        neural_risk_score: None,
                        final_risk_score: 100,
                        reasoning: format!("Blocked by Budget: {}", e),
                        latency: LatencyBreakdown {
                            total_us: start.elapsed().as_micros() as u64,
                            symbolic_us: prompt_us,
                            neural_us: None,
                        },
                    };
                }

                // If context has "tokens", consume them
                if let Some(tokens) = request.context.data.get("tokens").and_then(|t| t.as_u64()) {
                    if let Err(e) = budget.consume_tokens(tokens) {
                        return VerificationResult {
                            request_id: request.request_id,
                            allowed: false,
                            evaluated_policies: vec![],
                            blocking_policies: vec!["budget-limit".to_string()],
                            symbolic_risk_score: 100,
                            neural_risk_score: None,
                            final_risk_score: 100,
                            reasoning: format!("Blocked by Budget: {}", e),
                            latency: LatencyBreakdown {
                                total_us: start.elapsed().as_micros() as u64,
                                symbolic_us: prompt_us,
                                neural_us: None,
                            },
                        };
                    }
                }
            }
        }

        // === SYMBOLIC PATH (Fast) ===
        let symbolic_start = Instant::now();
        let (evaluated, blocking, symbolic_risk) = self.evaluate_symbolic(&request).await;
        let symbolic_us = symbolic_start.elapsed().as_micros() as u64;

        // === NEURAL PATH (If needed) ===
        let neural_result = if symbolic_risk >= self.neural_threshold {
            let neural_start = Instant::now();
            let score = self
                .neural_scorer
                .score(&request.action, &request.context)
                .await;
            Some((score, neural_start.elapsed().as_micros() as u64))
        } else {
            None
        };

        // === CARBON PATH (ESG Veto) ===
        let carbon_result = if let Some(veto) = &self.carbon_veto {
            // In a real request, these would come from the context or a header
            let compute_type = match request.context.data.get("compute_type") {
                Some(v) => match v.as_str() {
                    Some("gpu") => ComputeType::Gpu,
                    Some("tpu") => ComputeType::Tpu,
                    _ => ComputeType::Cpu,
                },
                None => ComputeType::Cpu,
            };

            let duration_ms = request
                .context
                .data
                .get("duration_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            Some(veto.evaluate(
                &request.agent_id,
                &request.action,
                compute_type,
                duration_ms,
            ))
        } else {
            None
        };

        let total_us = start.elapsed().as_micros() as u64;

        // Calculate final risk score
        let final_risk = if let Some((neural_risk, _)) = neural_result {
            // Combine symbolic and neural scores (weighted average)
            ((symbolic_risk as u16 + neural_risk as u16) / 2) as u8
        } else {
            symbolic_risk
        };

        // Determine if action is allowed
        let carbon_allowed = carbon_result.as_ref().map(|r| r.allowed).unwrap_or(true);

        // BLOCKING THRESHOLD: 80
        const BLOCKING_THRESHOLD: u8 = 80;
        let allowed = blocking.is_empty() && final_risk < BLOCKING_THRESHOLD && carbon_allowed;

        let reasoning = if !carbon_allowed {
            carbon_result
                .as_ref()
                .and_then(|r| r.message.clone())
                .unwrap_or_else(|| "Blocked by carbon budget".to_string())
        } else if !blocking.is_empty() {
            format!("Blocked by policies: {}", blocking.join(", "))
        } else if final_risk >= 80 {
            "Action blocked due to high risk score".to_string()
        } else {
            "All policies passed".to_string()
        };

        let result = VerificationResult {
            request_id: request.request_id,
            allowed,
            evaluated_policies: evaluated,
            blocking_policies: blocking,
            symbolic_risk_score: symbolic_risk,
            neural_risk_score: neural_result.map(|(score, _)| score),
            final_risk_score: final_risk,
            reasoning,
            latency: LatencyBreakdown {
                total_us,
                symbolic_us,
                neural_us: neural_result.map(|(_, us)| us),
            },
        };

        // P1 Fix: ISO 42001 Ready Structured Audit Logging
        tracing::info!(
            request_id = %result.request_id,
            agent_id = %request.agent_id,
            action = %request.action,
            allowed = result.allowed,
            final_risk = result.final_risk_score,
            symbolic_risk = result.symbolic_risk_score,
            neural_risk = ?result.neural_risk_score,
            latency_us = result.latency.total_us,
            "Verification complete"
        );

        result
    }

    /// Evaluate policies using the symbolic (deterministic) path.
    async fn evaluate_symbolic(
        &self,
        request: &VerificationRequest,
    ) -> (Vec<String>, Vec<String>, u8) {
        let policies = self.policies.read().await;

        let mut evaluated = Vec::new();
        let mut blocking = Vec::new();
        let mut max_risk = 0u8;

        // Build evaluation context
        let eval_ctx = EvalContext {
            action: request.action.clone(),
            agent_id: request.agent_id.clone(),
            context: request.context.data.clone(),
        };

        // Sort policies by priority (higher first)
        let mut sorted_policies: Vec<_> = policies
            .values()
            .filter(|p| {
                p.enabled
                    && p.applies_to_jurisdiction(self.jurisdiction)
                    && p.applies_to_namespace(&request.namespace)
            })
            .collect();
        sorted_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for policy in sorted_policies {
            evaluated.push(policy.id.clone());

            for rule in &policy.rules {
                if evaluate(&rule.condition, &eval_ctx) {
                    // Rule matched
                    if let Some(risk) = rule.risk_score {
                        max_risk = max_risk.max(risk);
                    }

                    match rule.action {
                        PolicyAction::Deny => {
                            blocking.push(policy.id.clone());
                            max_risk = max_risk.max(100);
                        }
                        PolicyAction::Review => {
                            // Flag for review but don't block
                            max_risk = max_risk.max(60);
                        }
                        PolicyAction::Audit => {
                            // Just log, no action needed
                        }
                        PolicyAction::Allow => {
                            // Explicitly allow
                        }
                    }
                }
            }
        }

        (evaluated, blocking, max_risk)
    }
}

/// Builder for creating verification requests.
pub struct VerificationRequestBuilder {
    agent_id: String,
    action: String,
    namespace: String,
    context: HashMap<String, serde_json::Value>,
}

impl VerificationRequestBuilder {
    pub fn new(agent_id: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            action: action.into(),
            namespace: "default".to_string(),
            context: HashMap::new(),
        }
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn context(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> VerificationRequest {
        VerificationRequest {
            request_id: Uuid::new_v4(),
            agent_id: self.agent_id,
            action: self.action,
            namespace: self.namespace,
            context: VerificationContext { data: self.context },
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::PolicyRule;

    #[tokio::test]
    async fn test_engine_allows_safe_action() {
        let engine = GateEngine::new();

        let request = VerificationRequestBuilder::new("agent-1", "send_email")
            .context("to", "user@example.com")
            .build();

        let result = engine.verify(request).await;
        assert!(result.allowed);
        assert_eq!(result.blocking_policies.len(), 0);
    }

    #[tokio::test]
    async fn test_engine_blocks_by_policy() {
        let engine = GateEngine::new();

        // Register a blocking policy
        let policy = Policy {
            id: "no-transfers".to_string(),
            name: "No Transfers".to_string(),
            description: String::new(),
            priority: 100,
            enabled: true,
            jurisdictions: vec![],
            namespace: "global".to_string(),
            rules: vec![PolicyRule {
                id: "block-transfer".to_string(),
                condition: "action == 'transfer_funds'".to_string(),
                action: PolicyAction::Deny,
                message: Some("Transfers are blocked".to_string()),
                risk_score: Some(100),
            }],
        };
        engine.register_policy(policy).await;

        let request = VerificationRequestBuilder::new("agent-1", "transfer_funds")
            .context("amount", 5000)
            .build();

        let result = engine.verify(request).await;
        assert!(!result.allowed);
        assert!(result
            .blocking_policies
            .contains(&"no-transfers".to_string()));
    }

    #[tokio::test]
    async fn test_latency_breakdown() {
        let engine = GateEngine::new();

        let request = VerificationRequestBuilder::new("agent-1", "read_data").build();

        let result = engine.verify(request).await;

        // Symbolic path should be very fast
        assert!(result.latency.symbolic_us < 1000); // <1ms
        assert!(result.latency.total_us >= result.latency.symbolic_us);
    }

    #[tokio::test]
    async fn test_carbon_veto_blocks_action() {
        use agentkern_treasury::carbon::{CarbonBudget, CarbonLedger};
        use rust_decimal_macros::dec;

        let ledger = CarbonLedger::new();
        let agent_id = "agent-carbon".to_string();

        // Set a tiny budget
        ledger.set_budget(
            CarbonBudget::new(agent_id.clone())
                .with_daily_limit(dec!(0.1))
                .block_on_exceed(),
        );

        let veto = CarbonVeto::new(ledger);
        let engine = GateEngine::new().with_carbon_veto(veto);

        let request = VerificationRequestBuilder::new(agent_id, "heavy_op")
            .context("compute_type", "gpu")
            .context("duration_ms", 60_000) // 1 minute @ GPU will exceed 0.1g
            .build();

        let result = engine.verify(request).await;

        assert!(!result.allowed);
        assert!(result.reasoning.contains("Carbon budget exceeded"));
    }

    #[tokio::test]
    async fn test_namespace_isolation() {
        let engine = GateEngine::new();

        // Policy in "namespace-A"
        let policy_a = Policy {
            id: "policy-a".to_string(),
            name: "Policy A".to_string(),
            description: String::new(),
            priority: 100,
            enabled: true,
            jurisdictions: vec![],
            namespace: "namespace-A".to_string(),
            rules: vec![PolicyRule {
                id: "rule-a".to_string(),
                condition: "action == 'test'".to_string(),
                action: PolicyAction::Deny,
                message: Some("Blocked A".to_string()),
                risk_score: Some(100),
            }],
        };
        engine.register_policy(policy_a).await;

        // Policy in "namespace-B"
        let policy_b = Policy {
            id: "policy-b".to_string(),
            name: "Policy B".to_string(),
            description: String::new(),
            priority: 100,
            enabled: true,
            jurisdictions: vec![],
            namespace: "namespace-B".to_string(),
            rules: vec![PolicyRule {
                id: "rule-b".to_string(),
                condition: "action == 'test'".to_string(),
                action: PolicyAction::Deny,
                message: Some("Blocked B".to_string()),
                risk_score: Some(100),
            }],
        };
        engine.register_policy(policy_b).await;

        // Request in namespace-A should be blocked by policy-a, but NOT policy-b
        let req_a = VerificationRequestBuilder::new("agent-1", "test")
            .namespace("namespace-A")
            .build();
        let res_a = engine.verify(req_a).await;
        assert!(!res_a.allowed);
        assert!(res_a.blocking_policies.contains(&"policy-a".to_string()));
        assert!(!res_a.blocking_policies.contains(&"policy-b".to_string()));

        // Request in namespace-C should NOT be blocked by either (no global policy)
        let req_c = VerificationRequestBuilder::new("agent-1", "test")
            .namespace("namespace-C")
            .build();
        let res_c = engine.verify(req_c).await;
        assert!(res_c.allowed);
    }
}
