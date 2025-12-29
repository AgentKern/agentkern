//! {{PROJECT_NAME}} - WASM Policy Module
//!
//! This policy module runs inside AgentKern Gate and enforces
//! custom rules on agent actions.
//!
//! Build with: `cargo build --target wasm32-unknown-unknown --release`

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Policy decision result
#[derive(Serialize)]
pub struct PolicyDecision {
    pub allowed: bool,
    pub reason: Option<String>,
    pub risk_score: f64,
}

/// Action context passed from Gate
#[derive(Deserialize)]
pub struct ActionContext {
    pub agent_id: String,
    pub action: String,
    pub target: String,
    pub metadata: serde_json::Value,
}

/// Main policy evaluation function
/// 
/// This is called by Gate for every agent action.
/// Return a PolicyDecision to allow or deny the action.
#[wasm_bindgen]
pub fn evaluate(context_json: &str) -> String {
    let context: ActionContext = match serde_json::from_str(context_json) {
        Ok(ctx) => ctx,
        Err(e) => {
            return serde_json::to_string(&PolicyDecision {
                allowed: false,
                reason: Some(format!("Invalid context: {}", e)),
                risk_score: 1.0,
            })
            .unwrap_or_default();
        }
    };

    // Example policy rules
    let decision = evaluate_action(&context);

    serde_json::to_string(&decision).unwrap_or_default()
}

fn evaluate_action(ctx: &ActionContext) -> PolicyDecision {
    // Rule 1: Block dangerous actions
    let dangerous_actions = ["delete_all", "sudo", "rm_rf"];
    if dangerous_actions.contains(&ctx.action.as_str()) {
        return PolicyDecision {
            allowed: false,
            reason: Some(format!("Action '{}' is blocked by policy", ctx.action)),
            risk_score: 1.0,
        };
    }

    // Rule 2: Check target domains
    if ctx.action == "http_request" {
        let blocked_domains = ["malware.com", "phishing.net"];
        for domain in blocked_domains {
            if ctx.target.contains(domain) {
                return PolicyDecision {
                    allowed: false,
                    reason: Some(format!("Domain '{}' is blocked", domain)),
                    risk_score: 0.9,
                };
            }
        }
    }

    // Rule 3: Rate-based risk scoring
    let risk_score = calculate_risk_score(ctx);
    if risk_score > 0.8 {
        return PolicyDecision {
            allowed: false,
            reason: Some("Risk score too high".to_string()),
            risk_score,
        };
    }

    // Default: Allow with calculated risk
    PolicyDecision {
        allowed: true,
        reason: None,
        risk_score,
    }
}

fn calculate_risk_score(ctx: &ActionContext) -> f64 {
    let mut score = 0.0;

    // Higher risk for write operations
    if ctx.action.contains("write") || ctx.action.contains("delete") {
        score += 0.3;
    }

    // Higher risk for external targets
    if ctx.target.starts_with("http://") {
        score += 0.2; // HTTP (not HTTPS) is riskier
    }

    // Higher risk for unknown agents
    if ctx.agent_id.contains("anonymous") {
        score += 0.4;
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_safe_action() {
        let ctx = ActionContext {
            agent_id: "agent-123".to_string(),
            action: "read".to_string(),
            target: "https://api.example.com".to_string(),
            metadata: serde_json::json!({}),
        };
        let decision = evaluate_action(&ctx);
        assert!(decision.allowed);
    }

    #[test]
    fn test_block_dangerous_action() {
        let ctx = ActionContext {
            agent_id: "agent-123".to_string(),
            action: "delete_all".to_string(),
            target: "database".to_string(),
            metadata: serde_json::json!({}),
        };
        let decision = evaluate_action(&ctx);
        assert!(!decision.allowed);
    }
}
