//! Infrastructure Evidence Collection
//!
//! 2026 ROADMAP: Automated evidence collection for SOC 2/FedRAMP compliance.
//! Per Audit: "Needs expansion to handle infrastructure evidence (cloud config, vuln scans)"

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

    #[test]
    fn test_evidence_collector() {
        let mut collector = InfrastructureEvidenceCollector::for_soc2();
        collector.collect_cloud_config("AWS", serde_json::json!({"region": "us-east-1"}));
        assert_eq!(collector.get_evidence().len(), 1);
    }
}
