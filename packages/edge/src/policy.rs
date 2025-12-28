//! Edge Policy Engine
//!
//! Lightweight policy evaluation for edge devices

use serde::{Deserialize, Serialize};

#[cfg(feature = "embedded")]
use alloc::string::String;

/// Edge policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgePolicy {
    /// Policy name
    pub name: String,
    /// Rules
    pub rules: Vec<PolicyRule>,
}

/// Policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule ID
    pub id: String,
    /// Action pattern to match
    pub pattern: String,
    /// Action to take
    pub action: PolicyAction,
    /// Priority (lower = higher priority)
    pub priority: u32,
}

impl PolicyRule {
    /// Check if action matches pattern.
    pub fn matches(&self, action: &str) -> bool {
        if self.pattern == "*" {
            return true;
        }

        if self.pattern.ends_with('*') {
            let prefix = &self.pattern[..self.pattern.len() - 1];
            return action.starts_with(prefix);
        }

        action == self.pattern
    }
}

/// Policy action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyAction {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Queue for sync (offline mode)
    Queue,
    /// Escalate to cloud
    Escalate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_match_exact() {
        let rule = PolicyRule {
            id: "r1".into(),
            pattern: "read_sensor".into(),
            action: PolicyAction::Allow,
            priority: 1,
        };

        assert!(rule.matches("read_sensor"));
        assert!(!rule.matches("write_sensor"));
    }

    #[test]
    fn test_policy_match_wildcard() {
        let rule = PolicyRule {
            id: "r1".into(),
            pattern: "sensor.*".into(),
            action: PolicyAction::Allow,
            priority: 1,
        };

        assert!(rule.matches("sensor.read"));
        assert!(rule.matches("sensor.write"));
        assert!(!rule.matches("actuator.write"));
    }

    #[test]
    fn test_policy_match_all() {
        let rule = PolicyRule {
            id: "r1".into(),
            pattern: "*".into(),
            action: PolicyAction::Queue,
            priority: 100,
        };

        assert!(rule.matches("anything"));
    }
}
