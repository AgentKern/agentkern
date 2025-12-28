//! Geo-Fence Policy Engine
//!
//! Enforces data residency rules during mesh sync.
//! Per GLOBAL_GAPS.md: "AgentKern-Sovereign"

use super::DataRegion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transfer policy for cross-region data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferPolicy {
    /// Block all transfers out of region
    Block,
    /// Allow with anonymization
    AllowAnonymized,
    /// Allow with explicit consent
    AllowWithConsent,
    /// Allow freely
    Allow,
}

/// Data residency rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyRule {
    /// Data pattern (e.g., "pii:*", "user:*")
    pub pattern: String,
    /// Policy for this pattern
    pub policy: TransferPolicy,
    /// Allowed target regions (if not Block)
    pub allowed_regions: Vec<DataRegion>,
}

/// Geo-fence controller.
pub struct GeoFence {
    /// Local region
    local_region: DataRegion,
    /// Residency rules
    rules: Vec<ResidencyRule>,
    /// Default policy
    default_policy: TransferPolicy,
}

impl GeoFence {
    /// Create a new geo-fence for a region.
    pub fn new(region: DataRegion) -> Self {
        let mut fence = Self {
            local_region: region,
            rules: Vec::new(),
            default_policy: TransferPolicy::Allow,
        };

        // Apply default rules based on region
        fence.apply_regional_defaults();
        fence
    }

    /// Apply regional defaults.
    fn apply_regional_defaults(&mut self) {
        match self.local_region {
            DataRegion::EuFrankfurt | DataRegion::EuIreland => {
                // GDPR: Block PII by default
                self.rules.push(ResidencyRule {
                    pattern: "pii:*".to_string(),
                    policy: TransferPolicy::Block,
                    allowed_regions: vec![DataRegion::EuFrankfurt, DataRegion::EuIreland],
                });
                self.default_policy = TransferPolicy::AllowWithConsent;
            }
            DataRegion::MenaRiyadh | DataRegion::MenaDubai => {
                // PDPL: Block all by default
                self.rules.push(ResidencyRule {
                    pattern: "*".to_string(),
                    policy: TransferPolicy::Block,
                    allowed_regions: vec![DataRegion::MenaRiyadh, DataRegion::MenaDubai],
                });
            }
            DataRegion::IndiaMumbai => {
                // DPDP: PII stays in India
                self.rules.push(ResidencyRule {
                    pattern: "pii:*".to_string(),
                    policy: TransferPolicy::Block,
                    allowed_regions: vec![DataRegion::IndiaMumbai],
                });
            }
            _ => {
                // Liberal default
                self.default_policy = TransferPolicy::Allow;
            }
        }
    }

    /// Add a custom residency rule.
    pub fn add_rule(&mut self, rule: ResidencyRule) {
        self.rules.push(rule);
    }

    /// Check if data can be transferred to target region.
    pub fn can_transfer(&self, target: DataRegion, data_id: &str) -> bool {
        // Same region is always allowed
        if target == self.local_region {
            return true;
        }

        // Check rules in order
        for rule in &self.rules {
            if self.matches_pattern(&rule.pattern, data_id) {
                return match rule.policy {
                    TransferPolicy::Block => rule.allowed_regions.contains(&target),
                    TransferPolicy::AllowAnonymized => true, // Caller must anonymize
                    TransferPolicy::AllowWithConsent => true, // Caller must verify consent
                    TransferPolicy::Allow => true,
                };
            }
        }

        // Use default policy
        self.default_policy != TransferPolicy::Block
    }

    /// Get the transfer policy for data.
    pub fn get_policy(&self, data_id: &str, target: DataRegion) -> TransferPolicy {
        if target == self.local_region {
            return TransferPolicy::Allow;
        }

        for rule in &self.rules {
            if self.matches_pattern(&rule.pattern, data_id) {
                if rule.allowed_regions.contains(&target) {
                    return TransferPolicy::Allow;
                }
                return rule.policy;
            }
        }

        self.default_policy
    }

    /// Simple pattern matching (supports * wildcard).
    fn matches_pattern(&self, pattern: &str, data_id: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return data_id.starts_with(prefix);
        }
        pattern == data_id
    }

    /// Get local region.
    pub fn local_region(&self) -> DataRegion {
        self.local_region
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eu_gdpr_blocks_pii() {
        let fence = GeoFence::new(DataRegion::EuFrankfurt);

        // PII should be blocked from leaving EU
        assert!(!fence.can_transfer(DataRegion::UsEast, "pii:user:123"));

        // PII within EU is allowed
        assert!(fence.can_transfer(DataRegion::EuIreland, "pii:user:123"));
    }

    #[test]
    fn test_mena_blocks_all() {
        let fence = GeoFence::new(DataRegion::MenaRiyadh);

        // Everything blocked from leaving
        assert!(!fence.can_transfer(DataRegion::UsEast, "any:data"));

        // Within MENA is allowed
        assert!(fence.can_transfer(DataRegion::MenaDubai, "any:data"));
    }

    #[test]
    fn test_us_liberal() {
        let fence = GeoFence::new(DataRegion::UsEast);

        // No default restrictions
        assert!(fence.can_transfer(DataRegion::EuFrankfurt, "user:123"));
        assert!(fence.can_transfer(DataRegion::MenaRiyadh, "data:abc"));
    }

    #[test]
    fn test_same_region() {
        let fence = GeoFence::new(DataRegion::EuFrankfurt);

        // Same region always allowed
        assert!(fence.can_transfer(DataRegion::EuFrankfurt, "pii:secret"));
    }

    #[test]
    fn test_custom_rule() {
        let mut fence = GeoFence::new(DataRegion::UsEast);
        fence.add_rule(ResidencyRule {
            pattern: "health:*".to_string(),
            policy: TransferPolicy::Block,
            allowed_regions: vec![DataRegion::UsEast, DataRegion::UsWest],
        });

        assert!(!fence.can_transfer(DataRegion::EuFrankfurt, "health:record:123"));
        assert!(fence.can_transfer(DataRegion::UsWest, "health:record:123"));
    }
}
