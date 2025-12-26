//! Escalation Triggers - Detect when human intervention is needed
//!
//! Monitors agent actions and triggers escalation based on configurable thresholds.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Escalation level severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EscalationLevel {
    /// Low priority - notification only
    Low,
    /// Medium priority - requires acknowledgment
    Medium,
    /// High priority - requires approval
    High,
    /// Critical - immediate human intervention required
    Critical,
}

impl EscalationLevel {
    /// Get default timeout for this level (in seconds).
    pub fn default_timeout_secs(&self) -> u64 {
        match self {
            Self::Low => 3600,      // 1 hour
            Self::Medium => 1800,   // 30 minutes
            Self::High => 300,      // 5 minutes
            Self::Critical => 60,   // 1 minute
        }
    }
    
    /// Should this level pause agent execution?
    pub fn should_pause(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

/// Trust threshold configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustThreshold {
    /// Minimum trust score (0.0 - 1.0)
    pub min_score: f64,
    /// Escalation level when breached
    pub level: EscalationLevel,
    /// Custom message
    pub message: Option<String>,
}

impl TrustThreshold {
    /// Create a new threshold.
    pub fn new(min_score: f64, level: EscalationLevel) -> Self {
        Self {
            min_score,
            level,
            message: None,
        }
    }
    
    /// Check if score breaches threshold.
    pub fn is_breached(&self, score: f64) -> bool {
        score < self.min_score
    }
}

/// Type of escalation trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    /// Trust score below threshold
    TrustScore,
    /// Budget exceeded
    BudgetExceeded,
    /// Loop detected
    LoopDetected,
    /// Anomaly detected
    AnomalyDetected,
    /// Policy violation
    PolicyViolation,
    /// Error rate exceeded
    ErrorRateExceeded,
    /// Manual trigger
    Manual,
    /// Custom trigger
    Custom(String),
}

/// Configuration for a trigger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Is trigger enabled?
    pub enabled: bool,
    /// Trust thresholds
    pub thresholds: Vec<TrustThreshold>,
    /// Cooldown between triggers (seconds)
    pub cooldown_secs: u64,
    /// Custom parameters
    pub params: HashMap<String, serde_json::Value>,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            trigger_type: TriggerType::TrustScore,
            enabled: true,
            thresholds: vec![
                TrustThreshold::new(0.8, EscalationLevel::Low),
                TrustThreshold::new(0.5, EscalationLevel::Medium),
                TrustThreshold::new(0.3, EscalationLevel::High),
                TrustThreshold::new(0.1, EscalationLevel::Critical),
            ],
            cooldown_secs: 60,
            params: HashMap::new(),
        }
    }
}

/// Result of trigger evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerResult {
    /// Was escalation triggered?
    pub triggered: bool,
    /// Escalation level
    pub level: EscalationLevel,
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Agent ID
    pub agent_id: String,
    /// Trigger reason
    pub reason: String,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
    /// Timestamp (Unix ms)
    pub timestamp: u64,
}

/// Escalation trigger evaluator.
pub struct EscalationTrigger {
    config: TriggerConfig,
    last_triggered: Option<u64>,
}

impl EscalationTrigger {
    /// Create with config.
    pub fn new(config: TriggerConfig) -> Self {
        Self {
            config,
            last_triggered: None,
        }
    }
    
    /// Create with default config.
    pub fn default_trust_trigger() -> Self {
        Self::new(TriggerConfig::default())
    }
    
    /// Check if in cooldown period.
    fn is_in_cooldown(&self) -> bool {
        if let Some(last) = self.last_triggered {
            let now = chrono::Utc::now().timestamp_millis() as u64;
            let cooldown_ms = self.config.cooldown_secs * 1000;
            now - last < cooldown_ms
        } else {
            false
        }
    }
    
    /// Evaluate trust score against thresholds.
    pub fn evaluate_trust(&mut self, agent_id: &str, score: f64) -> Option<TriggerResult> {
        if !self.config.enabled || self.is_in_cooldown() {
            return None;
        }
        
        // Find highest triggered threshold
        let mut highest: Option<&TrustThreshold> = None;
        for threshold in &self.config.thresholds {
            if threshold.is_breached(score) {
                if highest.is_none() || threshold.level > highest.unwrap().level {
                    highest = Some(threshold);
                }
            }
        }
        
        highest.map(|threshold| {
            self.last_triggered = Some(chrono::Utc::now().timestamp_millis() as u64);
            
            TriggerResult {
                triggered: true,
                level: threshold.level,
                trigger_type: TriggerType::TrustScore,
                agent_id: agent_id.to_string(),
                reason: threshold.message.clone().unwrap_or_else(|| {
                    format!("Trust score {:.2} below threshold {:.2}", score, threshold.min_score)
                }),
                context: {
                    let mut ctx = HashMap::new();
                    ctx.insert("score".into(), serde_json::json!(score));
                    ctx.insert("threshold".into(), serde_json::json!(threshold.min_score));
                    ctx
                },
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            }
        })
    }
    
    /// Evaluate budget usage.
    pub fn evaluate_budget(&mut self, agent_id: &str, used: f64, limit: f64) -> Option<TriggerResult> {
        if !self.config.enabled || self.is_in_cooldown() {
            return None;
        }
        
        let usage_ratio = used / limit;
        
        if usage_ratio > 1.0 {
            self.last_triggered = Some(chrono::Utc::now().timestamp_millis() as u64);
            
            Some(TriggerResult {
                triggered: true,
                level: EscalationLevel::Critical,
                trigger_type: TriggerType::BudgetExceeded,
                agent_id: agent_id.to_string(),
                reason: format!("Budget exceeded: ${:.2} used of ${:.2} limit", used, limit),
                context: {
                    let mut ctx = HashMap::new();
                    ctx.insert("used".into(), serde_json::json!(used));
                    ctx.insert("limit".into(), serde_json::json!(limit));
                    ctx.insert("ratio".into(), serde_json::json!(usage_ratio));
                    ctx
                },
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            })
        } else if usage_ratio > 0.9 {
            self.last_triggered = Some(chrono::Utc::now().timestamp_millis() as u64);
            
            Some(TriggerResult {
                triggered: true,
                level: EscalationLevel::High,
                trigger_type: TriggerType::BudgetExceeded,
                agent_id: agent_id.to_string(),
                reason: format!("Budget warning: {} used of {} limit ({}%)", used, limit, (usage_ratio * 100.0) as u32),
                context: {
                    let mut ctx = HashMap::new();
                    ctx.insert("used".into(), serde_json::json!(used));
                    ctx.insert("limit".into(), serde_json::json!(limit));
                    ctx.insert("ratio".into(), serde_json::json!(usage_ratio));
                    ctx
                },
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            })
        } else {
            None
        }
    }
    
    /// Manual escalation.
    pub fn manual_escalate(&mut self, agent_id: &str, reason: &str, level: EscalationLevel) -> TriggerResult {
        self.last_triggered = Some(chrono::Utc::now().timestamp_millis() as u64);
        
        TriggerResult {
            triggered: true,
            level,
            trigger_type: TriggerType::Manual,
            agent_id: agent_id.to_string(),
            reason: reason.to_string(),
            context: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escalation_levels() {
        assert!(EscalationLevel::Critical > EscalationLevel::High);
        assert!(EscalationLevel::High > EscalationLevel::Medium);
        assert!(EscalationLevel::Medium > EscalationLevel::Low);
    }

    #[test]
    fn test_level_should_pause() {
        assert!(!EscalationLevel::Low.should_pause());
        assert!(!EscalationLevel::Medium.should_pause());
        assert!(EscalationLevel::High.should_pause());
        assert!(EscalationLevel::Critical.should_pause());
    }

    #[test]
    fn test_trust_threshold_breach() {
        let threshold = TrustThreshold::new(0.5, EscalationLevel::High);
        
        assert!(threshold.is_breached(0.4));
        assert!(!threshold.is_breached(0.6));
        assert!(!threshold.is_breached(0.5));
    }

    #[test]
    fn test_evaluate_trust() {
        let mut trigger = EscalationTrigger::default_trust_trigger();
        
        // High score - no trigger
        assert!(trigger.evaluate_trust("agent-1", 0.9).is_none());
        
        // Low score - trigger
        let result = trigger.evaluate_trust("agent-1", 0.2).unwrap();
        assert!(result.triggered);
        assert_eq!(result.level, EscalationLevel::High);
    }

    #[test]
    fn test_budget_escalation() {
        let config = TriggerConfig {
            trigger_type: TriggerType::BudgetExceeded,
            cooldown_secs: 0, // No cooldown for test
            ..Default::default()
        };
        let mut trigger = EscalationTrigger::new(config);
        
        // Under budget - no trigger
        assert!(trigger.evaluate_budget("agent-1", 50.0, 100.0).is_none());
        
        // Over 90% - warning
        let result = trigger.evaluate_budget("agent-1", 95.0, 100.0).unwrap();
        assert_eq!(result.level, EscalationLevel::High);
    }

    #[test]
    fn test_budget_exceeded() {
        let config = TriggerConfig {
            trigger_type: TriggerType::BudgetExceeded,
            cooldown_secs: 0,
            ..Default::default()
        };
        let mut trigger = EscalationTrigger::new(config);
        
        let result = trigger.evaluate_budget("agent-1", 150.0, 100.0).unwrap();
        assert_eq!(result.level, EscalationLevel::Critical);
        assert!(result.reason.contains("exceeded"));
    }

    #[test]
    fn test_manual_escalation() {
        let mut trigger = EscalationTrigger::default_trust_trigger();
        
        let result = trigger.manual_escalate("agent-1", "Security concern", EscalationLevel::High);
        
        assert!(result.triggered);
        assert_eq!(result.level, EscalationLevel::High);
        assert_eq!(result.trigger_type, TriggerType::Manual);
    }
}
