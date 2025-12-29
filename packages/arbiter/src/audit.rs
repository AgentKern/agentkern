//! AgentKern-Arbiter: ISO 42001 Audit Ledger
//!
//! Per GLOBAL_GAPS.md §3: ISO/IEC 42001 (AIMS) Compliance
//!
//! Features:
//! - Traceability: Every autonomous action is logged
//! - Risk Management: Risk scores are recorded
//! - Human Oversight: Policy IDs and model versions are tracked
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_arbiter::audit::{AuditLedger, AuditRecord};
//!
//! let mut ledger = AuditLedger::new();
//! ledger.record(AuditRecord::new(
//!     "agent-123",
//!     "transfer_funds",
//!     "spending-limits-v2",
//!     75,
//!     AuditOutcome::Allowed,
//! ));
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Maximum records to keep in memory (older records are pruned).
const DEFAULT_MAX_RECORDS: usize = 100_000;

/// Outcome of an audited action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditOutcome {
    /// Action was allowed
    Allowed,
    /// Action was denied
    Denied,
    /// Action requires human review
    Review,
    /// Action was logged only (no enforcement)
    Logged,
}

/// A single audit record for ISO 42001 compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Unique record ID
    pub id: Uuid,
    /// Timestamp of the action
    pub timestamp: DateTime<Utc>,
    /// Agent that performed the action
    pub agent_id: String,
    /// Action that was attempted
    pub action: String,
    /// Policy ID that was evaluated
    pub policy_id: String,
    /// Policy version (for traceability)
    pub policy_version: String,
    /// Model version used for neural evaluation (if any)
    pub model_version: Option<String>,
    /// Risk score (0-100)
    pub risk_score: u8,
    /// Outcome of the evaluation
    pub outcome: AuditOutcome,
    /// Human-readable reasoning
    pub reasoning: String,
    /// Data region where action was evaluated
    pub region: String,
    /// Latency in microseconds
    pub latency_us: u64,
    /// Additional metadata (JSON-serializable)
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl AuditRecord {
    /// Create a new audit record.
    pub fn new(
        agent_id: impl Into<String>,
        action: impl Into<String>,
        policy_id: impl Into<String>,
        risk_score: u8,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            agent_id: agent_id.into(),
            action: action.into(),
            policy_id: policy_id.into(),
            policy_version: "1.0.0".to_string(),
            model_version: None,
            risk_score,
            outcome,
            reasoning: String::new(),
            region: "global".to_string(),
            latency_us: 0,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set the policy version.
    pub fn with_policy_version(mut self, version: impl Into<String>) -> Self {
        self.policy_version = version.into();
        self
    }

    /// Set the model version.
    pub fn with_model_version(mut self, version: impl Into<String>) -> Self {
        self.model_version = Some(version.into());
        self
    }

    /// Set the reasoning.
    pub fn with_reasoning(mut self, reasoning: impl Into<String>) -> Self {
        self.reasoning = reasoning.into();
        self
    }

    /// Set the region.
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Set the latency.
    pub fn with_latency(mut self, latency_us: u64) -> Self {
        self.latency_us = latency_us;
        self
    }

    /// Set additional metadata.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Audit ledger for storing and querying audit records.
#[derive(Debug)]
pub struct AuditLedger {
    records: Arc<RwLock<VecDeque<AuditRecord>>>,
    max_records: usize,
}

impl Default for AuditLedger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLedger {
    /// Create a new audit ledger with default capacity.
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(VecDeque::new())),
            max_records: DEFAULT_MAX_RECORDS,
        }
    }

    /// Create a new audit ledger with custom capacity.
    pub fn with_capacity(max_records: usize) -> Self {
        Self {
            records: Arc::new(RwLock::new(VecDeque::with_capacity(max_records))),
            max_records,
        }
    }

    /// Record an audit entry.
    pub async fn record(&self, record: AuditRecord) {
        let mut records = self.records.write().await;

        // Prune old records if at capacity
        while records.len() >= self.max_records {
            records.pop_front();
        }

        records.push_back(record);
    }

    /// Get the total number of records.
    pub async fn count(&self) -> usize {
        self.records.read().await.len()
    }

    /// Query records by agent ID.
    pub async fn query_by_agent(&self, agent_id: &str) -> Vec<AuditRecord> {
        let records = self.records.read().await;
        records
            .iter()
            .filter(|r| r.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Query records by action.
    pub async fn query_by_action(&self, action: &str) -> Vec<AuditRecord> {
        let records = self.records.read().await;
        records
            .iter()
            .filter(|r| r.action == action)
            .cloned()
            .collect()
    }

    /// Query records by outcome.
    pub async fn query_by_outcome(&self, outcome: AuditOutcome) -> Vec<AuditRecord> {
        let records = self.records.read().await;
        records
            .iter()
            .filter(|r| r.outcome == outcome)
            .cloned()
            .collect()
    }

    /// Query records within a time range.
    pub async fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<AuditRecord> {
        let records = self.records.read().await;
        records
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Query high-risk records (risk_score >= threshold).
    pub async fn query_high_risk(&self, threshold: u8) -> Vec<AuditRecord> {
        let records = self.records.read().await;
        records
            .iter()
            .filter(|r| r.risk_score >= threshold)
            .cloned()
            .collect()
    }

    /// Export all records as JSON (for ISO auditors).
    pub async fn export_json(&self) -> Result<String, serde_json::Error> {
        let records = self.records.read().await;
        let records_vec: Vec<_> = records.iter().collect();
        serde_json::to_string_pretty(&records_vec)
    }

    /// Get statistics for compliance reporting.
    pub async fn get_statistics(&self) -> AuditStatistics {
        let records = self.records.read().await;

        let total = records.len();
        let allowed = records
            .iter()
            .filter(|r| r.outcome == AuditOutcome::Allowed)
            .count();
        let denied = records
            .iter()
            .filter(|r| r.outcome == AuditOutcome::Denied)
            .count();
        let review = records
            .iter()
            .filter(|r| r.outcome == AuditOutcome::Review)
            .count();

        let avg_risk = if total > 0 {
            records.iter().map(|r| r.risk_score as u32).sum::<u32>() / total as u32
        } else {
            0
        };

        let avg_latency = if total > 0 {
            records.iter().map(|r| r.latency_us).sum::<u64>() / total as u64
        } else {
            0
        };

        AuditStatistics {
            total_records: total,
            allowed_count: allowed,
            denied_count: denied,
            review_count: review,
            average_risk_score: avg_risk as u8,
            average_latency_us: avg_latency,
        }
    }
}

/// Statistics for compliance reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_records: usize,
    pub allowed_count: usize,
    pub denied_count: usize,
    pub review_count: usize,
    pub average_risk_score: u8,
    pub average_latency_us: u64,
}

// ============================================================================
// 2026 ROADMAP: Infrastructure Evidence Collection (Per Audit Gap Analysis)
// Per Audit: "Needs expansion to handle infrastructure evidence (cloud config, vuln scans)"
// ============================================================================

/// Evidence type for compliance frameworks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Cloud configuration snapshot
    CloudConfig,
    /// Vulnerability scan results
    VulnerabilityScan,
    /// Access control list
    AccessControl,
    /// Network security configuration
    NetworkSecurity,
    /// Encryption status
    EncryptionStatus,
    /// Backup verification
    BackupVerification,
    /// Incident response logs
    IncidentResponse,
    /// Change management records
    ChangeManagement,
}

/// Compliance framework target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceFramework {
    Soc2TypeI,
    Soc2TypeII,
    Iso27001,
    FedRampLow,
    FedRampModerate,
    FedRampHigh,
    Hipaa,
    PciDss,
}

/// Collected evidence artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifact {
    /// Unique evidence ID
    pub id: uuid::Uuid,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Applicable frameworks
    pub frameworks: Vec<ComplianceFramework>,
    /// Control ID (e.g., "CC6.1" for SOC 2, "A.9.4" for ISO 27001)
    pub control_id: String,
    /// Evidence description
    pub description: String,
    /// Collected timestamp
    pub collected_at: DateTime<Utc>,
    /// Evidence data (JSON blob)
    pub data: serde_json::Value,
    /// Collection method
    pub collection_method: String,
    /// Pass/fail status
    pub status: EvidenceStatus,
}

/// Status of evidence collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceStatus {
    Passed,
    Failed,
    NeedsReview,
    NotApplicable,
}

/// Infrastructure evidence collector for automated compliance.
pub struct InfrastructureEvidenceCollector {
    /// Collected evidence
    evidence: Vec<EvidenceArtifact>,
    /// Target frameworks
    frameworks: Vec<ComplianceFramework>,
}

impl InfrastructureEvidenceCollector {
    /// Create a new collector for specific frameworks.
    pub fn new(frameworks: Vec<ComplianceFramework>) -> Self {
        Self {
            evidence: Vec::new(),
            frameworks,
        }
    }

    /// Create collector for SOC 2 Type II.
    pub fn for_soc2() -> Self {
        Self::new(vec![ComplianceFramework::Soc2TypeII])
    }

    /// Create collector for FedRAMP Moderate.
    pub fn for_fedramp_moderate() -> Self {
        Self::new(vec![ComplianceFramework::FedRampModerate])
    }

    /// Collect cloud configuration evidence.
    pub fn collect_cloud_config(&mut self, provider: &str, config_data: serde_json::Value) {
        let artifact = EvidenceArtifact {
            id: uuid::Uuid::new_v4(),
            evidence_type: EvidenceType::CloudConfig,
            frameworks: self.frameworks.clone(),
            control_id: "CC6.1".into(), // SOC 2 Logical Access
            description: format!("Cloud configuration snapshot from {}", provider),
            collected_at: Utc::now(),
            data: config_data,
            collection_method: "API".into(),
            status: EvidenceStatus::Passed,
        };
        self.evidence.push(artifact);
    }

    /// Collect vulnerability scan results.
    pub fn collect_vuln_scan(&mut self, scanner: &str, results: serde_json::Value) {
        let critical_count = results
            .get("critical")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let status = if critical_count > 0 {
            EvidenceStatus::Failed
        } else {
            EvidenceStatus::Passed
        };

        let artifact = EvidenceArtifact {
            id: uuid::Uuid::new_v4(),
            evidence_type: EvidenceType::VulnerabilityScan,
            frameworks: self.frameworks.clone(),
            control_id: "CC7.1".into(), // SOC 2 System Operations
            description: format!("Vulnerability scan from {}", scanner),
            collected_at: Utc::now(),
            data: results,
            collection_method: scanner.into(),
            status,
        };
        self.evidence.push(artifact);
    }

    /// Collect access control evidence.
    pub fn collect_access_controls(&mut self, iam_data: serde_json::Value) {
        let artifact = EvidenceArtifact {
            id: uuid::Uuid::new_v4(),
            evidence_type: EvidenceType::AccessControl,
            frameworks: self.frameworks.clone(),
            control_id: "AC-2".into(), // FedRAMP Account Management
            description: "IAM access control configuration".into(),
            collected_at: Utc::now(),
            data: iam_data,
            collection_method: "IAM API".into(),
            status: EvidenceStatus::Passed,
        };
        self.evidence.push(artifact);
    }

    /// Collect encryption status.
    pub fn collect_encryption_status(&mut self, encryption_data: serde_json::Value) {
        let artifact = EvidenceArtifact {
            id: uuid::Uuid::new_v4(),
            evidence_type: EvidenceType::EncryptionStatus,
            frameworks: self.frameworks.clone(),
            control_id: "SC-28".into(), // FedRAMP Protection at Rest
            description: "Encryption at rest and in transit status".into(),
            collected_at: Utc::now(),
            data: encryption_data,
            collection_method: "KMS API".into(),
            status: EvidenceStatus::Passed,
        };
        self.evidence.push(artifact);
    }

    /// Get all collected evidence.
    pub fn get_evidence(&self) -> &[EvidenceArtifact] {
        &self.evidence
    }

    /// Get evidence by type.
    pub fn get_by_type(&self, evidence_type: EvidenceType) -> Vec<&EvidenceArtifact> {
        self.evidence
            .iter()
            .filter(|e| e.evidence_type == evidence_type)
            .collect()
    }

    /// Get failed evidence (for remediation).
    pub fn get_failed(&self) -> Vec<&EvidenceArtifact> {
        self.evidence
            .iter()
            .filter(|e| e.status == EvidenceStatus::Failed)
            .collect()
    }

    /// Export evidence package for auditors.
    pub fn export_package(&self) -> Result<String, serde_json::Error> {
        let package = serde_json::json!({
            "generated_at": Utc::now().to_rfc3339(),
            "frameworks": self.frameworks,
            "evidence_count": self.evidence.len(),
            "passed": self.evidence.iter().filter(|e| e.status == EvidenceStatus::Passed).count(),
            "failed": self.evidence.iter().filter(|e| e.status == EvidenceStatus::Failed).count(),
            "artifacts": self.evidence,
        });
        serde_json::to_string_pretty(&package)
    }

    /// Generate compliance summary.
    pub fn summary(&self) -> EvidenceSummary {
        EvidenceSummary {
            total: self.evidence.len(),
            passed: self.evidence.iter().filter(|e| e.status == EvidenceStatus::Passed).count(),
            failed: self.evidence.iter().filter(|e| e.status == EvidenceStatus::Failed).count(),
            needs_review: self.evidence.iter().filter(|e| e.status == EvidenceStatus::NeedsReview).count(),
            frameworks: self.frameworks.clone(),
        }
    }
}

/// Summary of collected evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub needs_review: usize,
    pub frameworks: Vec<ComplianceFramework>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_record_creation() {
        let record = AuditRecord::new(
            "agent-123",
            "transfer_funds",
            "spending-limits-v2",
            75,
            AuditOutcome::Allowed,
        )
        .with_policy_version("2.1.0")
        .with_model_version("distilbert-v1")
        .with_reasoning("Risk within acceptable limits")
        .with_region("eu")
        .with_latency(500);

        assert_eq!(record.agent_id, "agent-123");
        assert_eq!(record.policy_version, "2.1.0");
        assert_eq!(record.model_version, Some("distilbert-v1".to_string()));
        assert_eq!(record.risk_score, 75);
        assert_eq!(record.outcome, AuditOutcome::Allowed);
    }

    #[tokio::test]
    async fn test_audit_ledger_record() {
        let ledger = AuditLedger::new();

        let record = AuditRecord::new(
            "agent-1",
            "send_email",
            "email-policy",
            30,
            AuditOutcome::Allowed,
        );

        ledger.record(record).await;
        assert_eq!(ledger.count().await, 1);
    }

    #[tokio::test]
    async fn test_audit_ledger_query_by_agent() {
        let ledger = AuditLedger::new();

        ledger
            .record(AuditRecord::new(
                "agent-1",
                "action-a",
                "policy-1",
                20,
                AuditOutcome::Allowed,
            ))
            .await;
        ledger
            .record(AuditRecord::new(
                "agent-2",
                "action-b",
                "policy-1",
                30,
                AuditOutcome::Allowed,
            ))
            .await;
        ledger
            .record(AuditRecord::new(
                "agent-1",
                "action-c",
                "policy-2",
                40,
                AuditOutcome::Denied,
            ))
            .await;

        let agent1_records = ledger.query_by_agent("agent-1").await;
        assert_eq!(agent1_records.len(), 2);
    }

    #[tokio::test]
    async fn test_audit_ledger_statistics() {
        let ledger = AuditLedger::new();

        ledger
            .record(AuditRecord::new("a", "x", "p", 20, AuditOutcome::Allowed))
            .await;
        ledger
            .record(AuditRecord::new("b", "y", "p", 80, AuditOutcome::Denied))
            .await;
        ledger
            .record(AuditRecord::new("c", "z", "p", 50, AuditOutcome::Review))
            .await;

        let stats = ledger.get_statistics().await;
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.allowed_count, 1);
        assert_eq!(stats.denied_count, 1);
        assert_eq!(stats.review_count, 1);
        assert_eq!(stats.average_risk_score, 50);
    }

    #[tokio::test]
    async fn test_audit_ledger_high_risk_query() {
        let ledger = AuditLedger::new();

        ledger
            .record(AuditRecord::new("a", "x", "p", 20, AuditOutcome::Allowed))
            .await;
        ledger
            .record(AuditRecord::new("b", "y", "p", 80, AuditOutcome::Denied))
            .await;
        ledger
            .record(AuditRecord::new("c", "z", "p", 90, AuditOutcome::Denied))
            .await;

        let high_risk = ledger.query_high_risk(75).await;
        assert_eq!(high_risk.len(), 2);
    }

    #[tokio::test]
    async fn test_audit_ledger_export_json() {
        let ledger = AuditLedger::new();

        ledger
            .record(AuditRecord::new(
                "agent-1",
                "action",
                "policy",
                50,
                AuditOutcome::Allowed,
            ))
            .await;

        let json = ledger.export_json().await.unwrap();
        assert!(json.contains("agent-1"));
        assert!(json.contains("policy"));
    }
}
