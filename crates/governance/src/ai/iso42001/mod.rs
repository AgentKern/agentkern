//! ISO 42001 Compliance Module
//!
//! Automated audit report generation for AI Management System.
//! Per GLOBAL_GAPS.md: "ISO 42001 Audit Ledger"

pub mod report;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use report::{AuditReport, ReportFormat, ReportGenerator};

/// ISO 42001 audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Agent ID
    pub agent_id: String,
    /// Action taken
    pub action: String,
    /// Policy ID that allowed/blocked
    pub policy_id: Option<String>,
    /// Model version used
    pub model_version: Option<String>,
    /// Risk score at decision time
    pub risk_score: u8,
    /// Human oversight status
    pub human_oversight: HumanOversight,
    /// Decision outcome
    pub outcome: AuditOutcome,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Human oversight status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanOversight {
    /// No human involved
    None,
    /// Human was notified
    Notified,
    /// Human approved action
    Approved,
    /// Human rejected action
    Rejected,
    /// Human review pending
    Pending,
}

/// Audit outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOutcome {
    /// Action was allowed
    Allowed,
    /// Action was denied
    Denied,
    /// Action requires escalation
    Escalated,
    /// Action was audited only
    AuditOnly,
}

/// ISO 42001 compliance ledger.
pub struct ComplianceLedger {
    /// All audit events
    events: Vec<AuditEvent>,
    /// Organization ID
    organization_id: String,
    /// System version
    system_version: String,
}

impl ComplianceLedger {
    /// Create a new compliance ledger.
    pub fn new(organization_id: String, system_version: String) -> Self {
        Self {
            events: Vec::new(),
            organization_id,
            system_version,
        }
    }

    /// Record an audit event.
    pub fn record(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    /// Get events for a time range.
    pub fn events_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    /// Get events by agent.
    pub fn events_by_agent(&self, agent_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .collect()
    }

    /// Generate ISO 42001 compliance report.
    pub fn generate_report(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> AuditReport {
        let events = self.events_in_range(period_start, period_end);

        let total_events = events.len();
        let denied_count = events
            .iter()
            .filter(|e| e.outcome == AuditOutcome::Denied)
            .count();
        let human_oversight_count = events
            .iter()
            .filter(|e| e.human_oversight != HumanOversight::None)
            .count();
        let high_risk_count = events.iter().filter(|e| e.risk_score >= 70).count();

        AuditReport {
            organization_id: self.organization_id.clone(),
            system_version: self.system_version.clone(),
            period_start,
            period_end,
            generated_at: Utc::now(),
            total_ai_decisions: total_events,
            denied_actions: denied_count,
            human_oversight_percentage: if total_events > 0 {
                (human_oversight_count as f64 / total_events as f64) * 100.0
            } else {
                0.0
            },
            high_risk_actions: high_risk_count,
            compliance_score: Self::calculate_compliance_score(&events),
            findings: Self::generate_findings(&events),
        }
    }

    /// Calculate compliance score (0-100).
    fn calculate_compliance_score(events: &[&AuditEvent]) -> u8 {
        if events.is_empty() {
            return 100;
        }

        let mut score = 100i32;

        // Deduct for high-risk actions without oversight
        let unmonitored_high_risk = events
            .iter()
            .filter(|e| e.risk_score >= 70 && e.human_oversight == HumanOversight::None)
            .count();
        score -= (unmonitored_high_risk * 5) as i32;

        // Deduct for missing policy IDs
        let missing_policy = events.iter().filter(|e| e.policy_id.is_none()).count();
        score -= (missing_policy * 2) as i32;

        // Deduct for missing model versions
        let missing_model = events.iter().filter(|e| e.model_version.is_none()).count();
        score -= missing_model as i32;

        score.max(0) as u8
    }

    /// Generate compliance findings.
    fn generate_findings(events: &[&AuditEvent]) -> Vec<ComplianceFinding> {
        let mut findings = Vec::new();

        let unmonitored = events
            .iter()
            .filter(|e| e.risk_score >= 70 && e.human_oversight == HumanOversight::None)
            .count();

        if unmonitored > 0 {
            findings.push(ComplianceFinding {
                severity: FindingSeverity::High,
                category: "Human Oversight".to_string(),
                description: format!("{} high-risk actions without human oversight", unmonitored),
                recommendation: "Implement mandatory human review for risk scores >= 70"
                    .to_string(),
            });
        }

        let undocumented = events.iter().filter(|e| e.policy_id.is_none()).count();
        if undocumented > 0 {
            findings.push(ComplianceFinding {
                severity: FindingSeverity::Medium,
                category: "Traceability".to_string(),
                description: format!("{} actions without policy attribution", undocumented),
                recommendation: "Ensure all AI decisions reference governing policy".to_string(),
            });
        }

        findings
    }
}

/// Compliance finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub severity: FindingSeverity,
    pub category: String,
    pub description: String,
    pub recommendation: String,
}

/// Finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(risk: u8, oversight: HumanOversight) -> AuditEvent {
        AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            agent_id: "agent-1".to_string(),
            action: "test_action".to_string(),
            policy_id: Some("policy-1".to_string()),
            model_version: Some("v1.0".to_string()),
            risk_score: risk,
            human_oversight: oversight,
            outcome: AuditOutcome::Allowed,
            context: HashMap::new(),
        }
    }

    #[test]
    fn test_ledger_creation() {
        let ledger = ComplianceLedger::new("org-1".to_string(), "1.0.0".to_string());
        assert!(ledger.events.is_empty());
    }

    #[test]
    fn test_record_event() {
        let mut ledger = ComplianceLedger::new("org-1".to_string(), "1.0.0".to_string());
        ledger.record(create_test_event(50, HumanOversight::None));

        assert_eq!(ledger.events.len(), 1);
    }

    #[test]
    fn test_compliance_score_perfect() {
        let events: Vec<&AuditEvent> = Vec::new();
        let score = ComplianceLedger::calculate_compliance_score(&events);
        assert_eq!(score, 100);
    }

    #[test]
    fn test_high_risk_without_oversight_deducts() {
        let event = create_test_event(80, HumanOversight::None);
        let events: Vec<&AuditEvent> = vec![&event];
        let score = ComplianceLedger::calculate_compliance_score(&events);

        // 100 - 5 (high risk without oversight) = 95
        assert!(score < 100);
    }

    #[test]
    fn test_report_generation() {
        let mut ledger = ComplianceLedger::new("org-1".to_string(), "1.0.0".to_string());
        ledger.record(create_test_event(30, HumanOversight::Approved));
        ledger.record(create_test_event(80, HumanOversight::None));

        let report = ledger.generate_report(
            Utc::now() - chrono::Duration::hours(1),
            Utc::now() + chrono::Duration::hours(1),
        );

        assert_eq!(report.total_ai_decisions, 2);
        assert!(report.human_oversight_percentage > 0.0);
    }
}
