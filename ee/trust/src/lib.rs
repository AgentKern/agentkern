#![allow(unused)]
//! AgentKern Enterprise: Trust & Reputation System
//!
//! Per LICENSING_STRATEGY.md: "AgentKern-Trust (Reputation)"
//! The "Credit Bureau" of Agents.
//!
//! **License**: AgentKern Enterprise License
//!
//! Features:
//! - Global agent reputation scores
//! - Trust-based access control
//! - Behavioral scoring
//! - Cross-organization reputation sharing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

mod license {
    #[derive(Debug, thiserror::Error)]
    pub enum LicenseError {
        #[error("Enterprise license required for Trust")]
        LicenseRequired,
    }

    pub fn require(feature: &str) -> Result<(), LicenseError> {
        let key = std::env::var("AGENTKERN_LICENSE_KEY")
            .map_err(|_| LicenseError::LicenseRequired)?;
        
        if key.is_empty() {
            return Err(LicenseError::LicenseRequired);
        }
        
        tracing::debug!(feature = %feature, "Enterprise trust feature accessed");
        Ok(())
    }
}

/// Agent reputation score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    /// Overall score (0-1000)
    pub score: u16,
    /// Confidence level (0-100)
    pub confidence: u8,
    /// Trust tier
    pub tier: TrustTier,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// Score components
    pub components: ScoreComponents,
}

impl Default for ReputationScore {
    fn default() -> Self {
        Self {
            score: 500, // Neutral starting score
            confidence: 0,
            tier: TrustTier::Unknown,
            updated_at: Utc::now(),
            components: ScoreComponents::default(),
        }
    }
}

/// Trust tier levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustTier {
    /// Blacklisted (0-199)
    Blacklisted,
    /// Untrusted (200-399)
    Untrusted,
    /// Unknown/New (400-599)
    Unknown,
    /// Trusted (600-799)
    Trusted,
    /// Verified (800-899)
    Verified,
    /// Elite (900-1000)
    Elite,
}

impl TrustTier {
    /// Get tier from numeric score.
    pub fn from_score(score: u16) -> Self {
        match score {
            0..=199 => Self::Blacklisted,
            200..=399 => Self::Untrusted,
            400..=599 => Self::Unknown,
            600..=799 => Self::Trusted,
            800..=899 => Self::Verified,
            900..=1000 => Self::Elite,
            _ => Self::Elite, // Cap at Elite
        }
    }

    /// Get minimum score for this tier.
    pub fn min_score(&self) -> u16 {
        match self {
            Self::Blacklisted => 0,
            Self::Untrusted => 200,
            Self::Unknown => 400,
            Self::Trusted => 600,
            Self::Verified => 800,
            Self::Elite => 900,
        }
    }

    /// Check if tier allows high-risk actions.
    pub fn allows_high_risk(&self) -> bool {
        matches!(self, Self::Verified | Self::Elite)
    }
}

/// Score components breakdown.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreComponents {
    /// Compliance adherence (0-200)
    pub compliance: u16,
    /// Success rate (0-200)
    pub success_rate: u16,
    /// Response time quality (0-200)
    pub performance: u16,
    /// Security behavior (0-200)
    pub security: u16,
    /// Historical longevity (0-200)
    pub longevity: u16,
}

impl ScoreComponents {
    /// Calculate total score.
    pub fn total(&self) -> u16 {
        self.compliance + self.success_rate + self.performance + self.security + self.longevity
    }
}

/// Reputation event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationEvent {
    /// Successful action completed
    ActionSuccess { action: String, impact: i16 },
    /// Action failed
    ActionFailed { action: String, impact: i16 },
    /// Policy violation
    PolicyViolation { policy_id: String, impact: i16 },
    /// Positive attestation from another agent
    PositiveAttestation { from_agent: String, impact: i16 },
    /// Negative attestation from another agent
    NegativeAttestation { from_agent: String, impact: i16 },
    /// Verification completed
    VerificationComplete { impact: i16 },
    /// Trust decay over time
    TimeDecay { impact: i16 },
}

impl ReputationEvent {
    /// Get the impact value.
    pub fn impact(&self) -> i16 {
        match self {
            Self::ActionSuccess { impact, .. } => *impact,
            Self::ActionFailed { impact, .. } => *impact,
            Self::PolicyViolation { impact, .. } => *impact,
            Self::PositiveAttestation { impact, .. } => *impact,
            Self::NegativeAttestation { impact, .. } => *impact,
            Self::VerificationComplete { impact } => *impact,
            Self::TimeDecay { impact } => *impact,
        }
    }
}

/// Agent record in the reputation system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    /// Agent ID
    pub agent_id: String,
    /// Organization ID
    pub org_id: String,
    /// Current reputation
    pub reputation: ReputationScore,
    /// Total actions
    pub total_actions: u64,
    /// Successful actions
    pub successful_actions: u64,
    /// Policy violations
    pub violations: u32,
    /// First seen
    pub first_seen: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// Is blacklisted
    pub blacklisted: bool,
    /// Blacklist reason
    pub blacklist_reason: Option<String>,
}

/// Trust network for agent-to-agent reputation.
#[derive(Debug)]
pub struct TrustNetwork {
    /// Agent records
    agents: HashMap<String, AgentRecord>,
    /// Trust relationships (agent -> agents they trust)
    trust_graph: HashMap<String, Vec<String>>,
}

impl TrustNetwork {
    /// Create a new trust network (requires enterprise license).
    pub fn new() -> Result<Self, license::LicenseError> {
        license::require("TRUST_NETWORK")?;
        
        Ok(Self {
            agents: HashMap::new(),
            trust_graph: HashMap::new(),
        })
    }

    /// Register a new agent.
    pub fn register_agent(&mut self, agent_id: &str, org_id: &str) -> &AgentRecord {
        let now = Utc::now();
        
        self.agents.entry(agent_id.to_string()).or_insert_with(|| {
            AgentRecord {
                agent_id: agent_id.to_string(),
                org_id: org_id.to_string(),
                reputation: ReputationScore::default(),
                total_actions: 0,
                successful_actions: 0,
                violations: 0,
                first_seen: now,
                last_activity: now,
                blacklisted: false,
                blacklist_reason: None,
            }
        })
    }

    /// Get agent reputation.
    pub fn get_reputation(&self, agent_id: &str) -> Option<&ReputationScore> {
        self.agents.get(agent_id).map(|r| &r.reputation)
    }

    /// Get agent trust tier.
    pub fn get_trust_tier(&self, agent_id: &str) -> TrustTier {
        self.agents
            .get(agent_id)
            .map(|r| r.reputation.tier)
            .unwrap_or(TrustTier::Unknown)
    }

    /// Record an event for an agent.
    pub fn record_event(&mut self, agent_id: &str, event: ReputationEvent) {
        if let Some(record) = self.agents.get_mut(agent_id) {
            let impact = event.impact();
            let new_score = (record.reputation.score as i32 + impact as i32)
                .max(0)
                .min(1000) as u16;
            
            record.reputation.score = new_score;
            record.reputation.tier = TrustTier::from_score(new_score);
            record.reputation.updated_at = Utc::now();
            record.last_activity = Utc::now();
            record.total_actions += 1;
            
            // Update counters based on event type
            match &event {
                ReputationEvent::ActionSuccess { .. } => {
                    record.successful_actions += 1;
                }
                ReputationEvent::PolicyViolation { .. } => {
                    record.violations += 1;
                    
                    // Auto-blacklist after too many violations
                    if record.violations >= 10 {
                        record.blacklisted = true;
                        record.blacklist_reason = Some("Too many policy violations".to_string());
                        record.reputation.tier = TrustTier::Blacklisted;
                    }
                }
                _ => {}
            }
            
            // Update confidence based on activity
            let confidence = ((record.total_actions as f64).log10() * 30.0).min(100.0) as u8;
            record.reputation.confidence = confidence;
            
            tracing::info!(
                agent_id = %agent_id,
                score = new_score,
                tier = ?record.reputation.tier,
                "Reputation updated"
            );
        }
    }

    /// Check if an agent can perform a high-risk action.
    pub fn can_perform_high_risk(&self, agent_id: &str) -> bool {
        self.agents
            .get(agent_id)
            .map(|r| !r.blacklisted && r.reputation.tier.allows_high_risk())
            .unwrap_or(false)
    }

    /// Blacklist an agent.
    pub fn blacklist(&mut self, agent_id: &str, reason: &str) {
        if let Some(record) = self.agents.get_mut(agent_id) {
            record.blacklisted = true;
            record.blacklist_reason = Some(reason.to_string());
            record.reputation.score = 0;
            record.reputation.tier = TrustTier::Blacklisted;
            
            tracing::warn!(
                agent_id = %agent_id,
                reason = %reason,
                "Agent blacklisted"
            );
        }
    }

    /// Add a trust relationship.
    pub fn add_trust(&mut self, from_agent: &str, to_agent: &str) {
        self.trust_graph
            .entry(from_agent.to_string())
            .or_insert_with(Vec::new)
            .push(to_agent.to_string());
    }

    /// Get agents trusted by a given agent.
    pub fn get_trusted_by(&self, agent_id: &str) -> Vec<&str> {
        self.trust_graph
            .get(agent_id)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get global statistics.
    pub fn get_stats(&self) -> TrustNetworkStats {
        let total = self.agents.len();
        let blacklisted = self.agents.values().filter(|a| a.blacklisted).count();
        let trusted = self.agents.values().filter(|a| a.reputation.tier >= TrustTier::Trusted).count();
        let avg_score = if total > 0 {
            self.agents.values().map(|a| a.reputation.score as u64).sum::<u64>() / total as u64
        } else {
            0
        };
        
        TrustNetworkStats {
            total_agents: total,
            blacklisted_agents: blacklisted,
            trusted_agents: trusted,
            average_score: avg_score as u16,
        }
    }
}

/// Trust network statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustNetworkStats {
    pub total_agents: usize,
    pub blacklisted_agents: usize,
    pub trusted_agents: usize,
    pub average_score: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_tier_from_score() {
        assert_eq!(TrustTier::from_score(0), TrustTier::Blacklisted);
        assert_eq!(TrustTier::from_score(500), TrustTier::Unknown);
        assert_eq!(TrustTier::from_score(750), TrustTier::Trusted);
        assert_eq!(TrustTier::from_score(950), TrustTier::Elite);
    }

    #[test]
    fn test_high_risk_permissions() {
        assert!(TrustTier::Elite.allows_high_risk());
        assert!(TrustTier::Verified.allows_high_risk());
        assert!(!TrustTier::Trusted.allows_high_risk());
        assert!(!TrustTier::Unknown.allows_high_risk());
    }

    #[test]
    fn test_trust_network_requires_license() {
        unsafe { std::env::remove_var("AGENTKERN_LICENSE_KEY"); }
        let result = TrustNetwork::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_reputation_events() {
        unsafe { std::env::set_var("AGENTKERN_LICENSE_KEY", "test-license"); }
        
        let mut network = TrustNetwork::new().unwrap();
        network.register_agent("agent-1", "org-1");
        
        // Start at 500 (Unknown tier)
        assert_eq!(network.get_trust_tier("agent-1"), TrustTier::Unknown);
        
        // Success events should increase score
        network.record_event("agent-1", ReputationEvent::ActionSuccess {
            action: "test".to_string(),
            impact: 50,
        });
        
        assert!(network.get_reputation("agent-1").unwrap().score > 500);
        
        unsafe { std::env::remove_var("AGENTKERN_LICENSE_KEY"); }
    }

    #[test]
    fn test_blacklisting() {
        unsafe { std::env::set_var("AGENTKERN_LICENSE_KEY", "test-license"); }
        
        let mut network = TrustNetwork::new().unwrap();
        network.register_agent("bad-agent", "org-1");
        
        network.blacklist("bad-agent", "Malicious behavior");
        
        assert!(!network.can_perform_high_risk("bad-agent"));
        assert_eq!(network.get_trust_tier("bad-agent"), TrustTier::Blacklisted);
        
        unsafe { std::env::remove_var("AGENTKERN_LICENSE_KEY"); }
    }
}
