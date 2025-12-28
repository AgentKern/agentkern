//! AgentKern-Gate: Policy Definition
//!
//! YAML-based policy DSL for defining guardrails.
//!
//! # Example Policy (YAML)
//!
//! ```yaml
//! id: spending-limits
//! name: Spending Limits Policy
//! description: Prevent excessive spending by agents
//! priority: 100
//! enabled: true
//! jurisdictions: [us, eu, global]
//!
//! rules:
//!   - id: max-transaction
//!     condition: "action == 'transfer_funds' && context.amount > 10000"
//!     action: deny
//!     message: "Transaction exceeds maximum allowed amount"
//!     
//!   - id: require-approval
//!     condition: "action == 'transfer_funds' && context.amount > 1000"
//!     action: review
//!     message: "Transaction requires human approval"
//!     
//!   - id: audit-all-transfers
//!     condition: "action == 'transfer_funds'"
//!     action: audit
//! ```

use serde::{Deserialize, Serialize};
use crate::types::DataRegion;

/// A AgentKern policy definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique policy identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Policy description
    #[serde(default)]
    pub description: String,
    /// Priority (higher = evaluated first)
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Is this policy active?
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Jurisdictions where this policy applies
    #[serde(default)]
    pub jurisdictions: Vec<DataRegion>,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
}

fn default_priority() -> i32 { 0 }
fn default_enabled() -> bool { true }

/// Individual policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule identifier
    pub id: String,
    /// Condition expression (DSL)
    pub condition: String,
    /// Action to take if condition matches
    pub action: PolicyAction,
    /// Optional message for denials/reviews
    #[serde(default)]
    pub message: Option<String>,
    /// Risk score to assign if matched (0-100)
    #[serde(default)]
    pub risk_score: Option<u8>,
}

/// Action to take when a policy rule matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyAction {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Flag for human review
    Review,
    /// Log for audit purposes
    Audit,
}

impl Policy {
    /// Parse a policy from YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }

    /// Serialize policy to YAML string.
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Check if this policy applies to a given jurisdiction.
    pub fn applies_to_jurisdiction(&self, region: DataRegion) -> bool {
        if self.jurisdictions.is_empty() {
            return true; // Applies globally if no jurisdictions specified
        }
        self.jurisdictions.contains(&region) || self.jurisdictions.contains(&DataRegion::Global)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_from_yaml() {
        let yaml = r#"
id: test-policy
name: Test Policy
description: A test policy
priority: 100
enabled: true
jurisdictions: [us, eu]
rules:
  - id: deny-high-risk
    condition: "action == 'delete_all'"
    action: deny
    message: "This action is too dangerous"
    risk_score: 100
"#;

        let policy = Policy::from_yaml(yaml).unwrap();
        assert_eq!(policy.id, "test-policy");
        assert_eq!(policy.name, "Test Policy");
        assert_eq!(policy.priority, 100);
        assert!(policy.enabled);
        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.rules[0].action, PolicyAction::Deny);
    }

    #[test]
    fn test_jurisdiction_matching() {
        let policy = Policy {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            priority: 0,
            enabled: true,
            jurisdictions: vec![DataRegion::Eu, DataRegion::Us],
            rules: vec![],
        };

        assert!(policy.applies_to_jurisdiction(DataRegion::Eu));
        assert!(policy.applies_to_jurisdiction(DataRegion::Us));
        assert!(!policy.applies_to_jurisdiction(DataRegion::Cn));
    }
}
