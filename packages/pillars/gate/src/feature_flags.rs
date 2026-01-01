//! Feature Flags - Privacy-First Feature Management
//!
//! Per DevEx Roadmap: "Platform-Native Feature Flags"
//! Enables canary rollouts and dark launching of new features.
//!
//! # Features
//! - In-memory flag storage (default)
//! - Redis-backed for distributed deployments
//! - Percentage-based rollouts
//! - Agent-specific targeting
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::feature_flags::{FeatureFlags, Flag, Rollout};
//!
//! let flags = FeatureFlags::new();
//! flags.set("new_llm_model", Flag::percentage(10)); // 10% rollout
//!
//! if flags.is_enabled("new_llm_model", "agent-123") {
//!     // Use new model
//! }
//! ```

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Feature flag value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlagValue {
    /// Flag is on/off
    Boolean(bool),
    /// Percentage rollout (0-100)
    Percentage(u8),
    /// Specific agent IDs enabled
    AllowList(Vec<String>),
    /// Specific agent IDs disabled
    DenyList(Vec<String>),
    /// JSON value for complex configs
    Json(serde_json::Value),
}

/// Feature flag with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    /// Flag value
    pub value: FlagValue,
    /// Human-readable description
    pub description: Option<String>,
    /// Flag owner (team/person)
    pub owner: Option<String>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Expiration date (for temporary flags)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Flag {
    /// Create a boolean flag.
    pub fn boolean(enabled: bool) -> Self {
        Self {
            value: FlagValue::Boolean(enabled),
            description: None,
            owner: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
        }
    }

    /// Create a percentage rollout flag.
    pub fn percentage(pct: u8) -> Self {
        Self {
            value: FlagValue::Percentage(pct.min(100)),
            description: None,
            owner: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
        }
    }

    /// Create an allow-list flag.
    pub fn allow_list(agents: Vec<String>) -> Self {
        Self {
            value: FlagValue::AllowList(agents),
            description: None,
            owner: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
        }
    }

    /// Add description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add owner.
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    /// Set expiration.
    pub fn expires_in(mut self, duration: chrono::Duration) -> Self {
        self.expires_at = Some(chrono::Utc::now() + duration);
        self
    }
}

/// Feature flag evaluation context.
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// Agent ID
    pub agent_id: String,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

impl EvalContext {
    /// Create context for an agent.
    pub fn for_agent(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute.
    pub fn with_attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Feature flag manager.
pub struct FeatureFlags {
    flags: RwLock<HashMap<String, Flag>>,
}

impl FeatureFlags {
    /// Create a new feature flags manager.
    pub fn new() -> Self {
        Self {
            flags: RwLock::new(HashMap::new()),
        }
    }

    /// Set a flag.
    pub fn set(&self, name: impl Into<String>, flag: Flag) {
        let name = name.into();
        tracing::info!(flag = %name, "Feature flag set");
        self.flags.write().insert(name, flag);
    }

    /// Remove a flag.
    pub fn remove(&self, name: &str) -> Option<Flag> {
        tracing::info!(flag = %name, "Feature flag removed");
        self.flags.write().remove(name)
    }

    /// Check if a flag is enabled for a given context.
    pub fn is_enabled(&self, name: &str, ctx: &EvalContext) -> bool {
        let flags = self.flags.read();
        let Some(flag) = flags.get(name) else {
            return false; // Unknown flags are disabled
        };

        // Check expiration
        if let Some(expires) = flag.expires_at {
            if chrono::Utc::now() > expires {
                return false;
            }
        }

        match &flag.value {
            FlagValue::Boolean(enabled) => *enabled,
            FlagValue::Percentage(pct) => {
                // Consistent hashing based on agent ID
                let bucket = self.hash_to_bucket(&ctx.agent_id);
                bucket < *pct as u32
            }
            FlagValue::AllowList(agents) => agents.contains(&ctx.agent_id),
            FlagValue::DenyList(agents) => !agents.contains(&ctx.agent_id),
            FlagValue::Json(_) => true, // JSON flags are "enabled" if present
        }
    }

    /// Get flag value (for complex configs).
    pub fn get_value(&self, name: &str) -> Option<FlagValue> {
        self.flags.read().get(name).map(|f| f.value.clone())
    }

    /// Get all flag names.
    pub fn list_flags(&self) -> Vec<String> {
        self.flags.read().keys().cloned().collect()
    }

    /// Hash agent ID to a bucket (0-99) for percentage rollouts.
    fn hash_to_bucket(&self, agent_id: &str) -> u32 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        agent_id.hash(&mut hasher);
        (hasher.finish() % 100) as u32
    }

    /// Convenience: check with just agent ID string.
    pub fn is_enabled_for(&self, name: &str, agent_id: &str) -> bool {
        self.is_enabled(name, &EvalContext::for_agent(agent_id))
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Presets for common feature flag patterns.
pub mod presets {
    use super::*;

    /// Create a canary rollout (5% of agents).
    pub fn canary() -> Flag {
        Flag::percentage(5).with_description("Canary rollout - 5% of agents")
    }

    /// Create a beta flag (25% of agents).
    pub fn beta() -> Flag {
        Flag::percentage(25).with_description("Beta rollout - 25% of agents")
    }

    /// Create a dark launch flag (disabled by default).
    pub fn dark_launch() -> Flag {
        Flag::boolean(false).with_description("Dark launch - feature hidden")
    }

    /// Create a GA flag (enabled for everyone).
    pub fn general_availability() -> Flag {
        Flag::boolean(true).with_description("General availability")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_flag() {
        let flags = FeatureFlags::new();
        flags.set("new_feature", Flag::boolean(true));

        let ctx = EvalContext::for_agent("agent-1");
        assert!(flags.is_enabled("new_feature", &ctx));
    }

    #[test]
    fn test_percentage_rollout() {
        let flags = FeatureFlags::new();
        flags.set("experiment", Flag::percentage(50));

        // Test multiple agents - roughly 50% should be enabled
        let mut enabled_count = 0;
        for i in 0..100 {
            let ctx = EvalContext::for_agent(format!("agent-{}", i));
            if flags.is_enabled("experiment", &ctx) {
                enabled_count += 1;
            }
        }
        // Should be roughly 50% (allow some variance due to hashing)
        assert!(enabled_count > 30 && enabled_count < 70);
    }

    #[test]
    fn test_allow_list() {
        let flags = FeatureFlags::new();
        flags.set(
            "private_beta",
            Flag::allow_list(vec!["agent-vip".to_string()]),
        );

        assert!(flags.is_enabled_for("private_beta", "agent-vip"));
        assert!(!flags.is_enabled_for("private_beta", "agent-regular"));
    }

    #[test]
    fn test_unknown_flag() {
        let flags = FeatureFlags::new();
        let ctx = EvalContext::for_agent("agent-1");

        // Unknown flags default to disabled
        assert!(!flags.is_enabled("nonexistent", &ctx));
    }

    #[test]
    fn test_presets() {
        let canary = presets::canary();
        assert!(matches!(canary.value, FlagValue::Percentage(5)));
    }
}
