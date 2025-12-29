//! EU AI Act Compliance Export
//!
//! Per MANDATE.md Section 2: Global Compliance
//! Per GLOBAL_GAPS.md: EU AI Act takes effect Aug 2025
//!
//! Implements Article 13 (Transparency) and Article 14 (Human Oversight)
//! requirements for high-risk AI systems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI system risk classification per EU AI Act.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Minimal risk - no requirements
    Minimal,
    /// Limited risk - transparency requirements
    Limited,
    /// High risk - full compliance required
    HighRisk,
    /// Unacceptable risk - prohibited
    Prohibited,
}

impl RiskLevel {
    /// Check if this level requires FRIA (Fundamental Rights Impact Assessment).
    pub fn requires_fria(&self) -> bool {
        matches!(self, Self::HighRisk)
    }

    /// Check if this level requires conformity assessment.
    pub fn requires_conformity(&self) -> bool {
        matches!(self, Self::HighRisk)
    }
}

/// High-risk AI use cases (Annex III).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HighRiskCategory {
    /// Biometric identification
    BiometricIdentification,
    /// Critical infrastructure management
    CriticalInfrastructure,
    /// Education and vocational training
    Education,
    /// Employment, workers management
    Employment,
    /// Access to essential services
    EssentialServices,
    /// Law enforcement
    LawEnforcement,
    /// Migration, asylum, border control
    Migration,
    /// Administration of justice
    Justice,
    /// Democratic processes
    DemocraticProcesses,
}

/// Technical documentation required by Article 11.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDocumentation {
    /// System description
    pub description: SystemDescription,
    /// Design specifications
    pub design: DesignSpecifications,
    /// Risk management system
    pub risk_management: RiskManagement,
    /// Data governance
    pub data: DataGovernance,
    /// Human oversight measures
    pub human_oversight: HumanOversight,
    /// Accuracy and robustness
    pub performance: PerformanceMetrics,
    /// Cybersecurity measures
    pub cybersecurity: CybersecurityMeasures,
}

/// System description (Article 11.1.a).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDescription {
    /// System name
    pub name: String,
    /// Intended purpose
    pub purpose: String,
    /// Version
    pub version: String,
    /// Provider details
    pub provider: ProviderInfo,
    /// Date of deployment
    pub deployment_date: Option<String>,
    /// Risk classification
    pub risk_level: RiskLevel,
    /// High-risk categories if applicable
    pub high_risk_categories: Vec<HighRiskCategory>,
    /// Description of AI techniques used
    pub techniques: Vec<String>,
}

/// Provider information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub address: String,
    pub contact_email: String,
    pub eu_representative: Option<String>,
}

/// Design specifications (Article 11.1.b-c).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSpecifications {
    /// Architecture description
    pub architecture: String,
    /// Algorithmic logic
    pub algorithms: Vec<String>,
    /// Input/output specifications
    pub io_specs: IoSpecifications,
    /// Computational resources
    pub resources: ResourceRequirements,
    /// External dependencies
    pub dependencies: Vec<String>,
}

/// Input/output specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoSpecifications {
    pub inputs: Vec<DataSpecification>,
    pub outputs: Vec<DataSpecification>,
}

/// Data specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSpecification {
    pub name: String,
    pub data_type: String,
    pub description: String,
}

/// Resource requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub compute: String,
    pub memory: String,
    pub storage: String,
}

/// Risk management system (Article 9).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagement {
    /// Risk identification methodology
    pub methodology: String,
    /// Identified risks
    pub risks: Vec<IdentifiedRisk>,
    /// Mitigation measures
    pub mitigations: Vec<MitigationMeasure>,
    /// Residual risks
    pub residual_risks: Vec<String>,
    /// Testing procedures
    pub testing: TestingProcedures,
}

/// Identified risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedRisk {
    pub id: String,
    pub description: String,
    pub likelihood: String,
    pub impact: String,
    pub affected_rights: Vec<String>,
}

/// Mitigation measure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationMeasure {
    pub risk_id: String,
    pub measure: String,
    pub effectiveness: String,
}

/// Testing procedures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingProcedures {
    pub unit_tests: u32,
    pub integration_tests: u32,
    pub adversarial_tests: u32,
    pub test_datasets: Vec<String>,
    pub coverage_percentage: f32,
}

/// Data governance (Article 10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGovernance {
    /// Training data description
    pub training_data: DatasetInfo,
    /// Validation data description
    pub validation_data: DatasetInfo,
    /// Test data description
    pub test_data: DatasetInfo,
    /// Data quality measures
    pub quality_measures: Vec<String>,
    /// Bias detection and mitigation
    pub bias_mitigation: BiasMitigation,
}

/// Dataset information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub description: String,
    pub size: String,
    pub sources: Vec<String>,
    pub collection_period: Option<String>,
}

/// Bias mitigation measures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasMitigation {
    pub detection_methods: Vec<String>,
    pub mitigation_actions: Vec<String>,
    pub monitoring: String,
}

/// Human oversight measures (Article 14).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanOversight {
    /// Human oversight capability
    pub capability: String,
    /// Interface for human review
    pub interface: String,
    /// Stop/override mechanism
    pub stop_mechanism: String,
    /// Training for operators
    pub operator_training: String,
    /// Monitoring frequency
    pub monitoring_frequency: String,
}

/// Performance metrics (Article 15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Accuracy measures
    pub accuracy: HashMap<String, f64>,
    /// Robustness measures
    pub robustness: Vec<String>,
    /// Consistency measures
    pub consistency: String,
    /// Known limitations
    pub limitations: Vec<String>,
}

/// Cybersecurity measures (Article 15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CybersecurityMeasures {
    /// Security certifications
    pub certifications: Vec<String>,
    /// Vulnerability management
    pub vulnerability_management: String,
    /// Incident response
    pub incident_response: String,
    /// Access control
    pub access_control: String,
    /// Encryption
    pub encryption: String,
}

/// EU AI Act compliance exporter.
pub struct EuAiActExporter;

impl EuAiActExporter {
    /// Create a new exporter.
    pub fn new() -> Self {
        Self
    }

    /// Generate compliance report.
    pub fn generate_report(&self, doc: &TechnicalDocumentation) -> ComplianceReport {
        let mut findings: Vec<ComplianceFinding> = Vec::new();
        let mut score = 100i32;

        // Check Article 9: Risk Management
        if doc.risk_management.risks.is_empty() {
            findings.push(ComplianceFinding {
                article: "9".into(),
                requirement: "Risk identification".into(),
                status: ComplianceStatus::NonCompliant,
                detail: "No risks identified".into(),
            });
            score -= 20;
        }

        // Check Article 10: Data Governance
        if doc.data.bias_mitigation.detection_methods.is_empty() {
            findings.push(ComplianceFinding {
                article: "10".into(),
                requirement: "Bias detection".into(),
                status: ComplianceStatus::PartiallyCompliant,
                detail: "Bias detection methods not documented".into(),
            });
            score -= 10;
        }

        // Check Article 13: Transparency
        if doc.description.purpose.is_empty() {
            findings.push(ComplianceFinding {
                article: "13".into(),
                requirement: "Intended purpose".into(),
                status: ComplianceStatus::NonCompliant,
                detail: "System purpose not documented".into(),
            });
            score -= 15;
        }

        // Check Article 14: Human Oversight
        if doc.human_oversight.stop_mechanism.is_empty() {
            findings.push(ComplianceFinding {
                article: "14".into(),
                requirement: "Stop mechanism".into(),
                status: ComplianceStatus::PartiallyCompliant,
                detail: "Human override mechanism not documented".into(),
            });
            score -= 10;
        }

        // Check Article 15: Accuracy and Robustness
        if doc.risk_management.testing.coverage_percentage < 80.0 {
            findings.push(ComplianceFinding {
                article: "15".into(),
                requirement: "Testing coverage".into(),
                status: ComplianceStatus::PartiallyCompliant,
                detail: format!(
                    "Test coverage {}% below 80% threshold",
                    doc.risk_management.testing.coverage_percentage
                ),
            });
            score -= 5;
        }

        let status = if score >= 90 {
            OverallStatus::Compliant
        } else if score >= 70 {
            OverallStatus::PartiallyCompliant
        } else {
            OverallStatus::NonCompliant
        };

        ComplianceReport {
            generated_at: chrono::Utc::now().to_rfc3339(),
            system_name: doc.description.name.clone(),
            risk_level: doc.description.risk_level,
            overall_status: status,
            score: score.max(0) as u32,
            findings,
            requires_fria: doc.description.risk_level.requires_fria(),
            requires_conformity: doc.description.risk_level.requires_conformity(),
        }
    }

    /// Export to JSON.
    pub fn export_json(&self, doc: &TechnicalDocumentation) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(doc)
    }

    /// Export to human-readable text.
    pub fn export_text(&self, doc: &TechnicalDocumentation) -> String {
        let report = self.generate_report(doc);

        let mut text = String::new();
        text.push_str("═══════════════════════════════════════════════════════════════\n");
        text.push_str("              EU AI ACT COMPLIANCE REPORT\n");
        text.push_str("═══════════════════════════════════════════════════════════════\n\n");

        text.push_str(&format!("System: {}\n", report.system_name));
        text.push_str(&format!("Risk Level: {:?}\n", report.risk_level));
        text.push_str(&format!("Status: {:?}\n", report.overall_status));
        text.push_str(&format!("Score: {}%\n\n", report.score));

        if report.requires_fria {
            text.push_str("⚠️  FRIA (Fundamental Rights Impact Assessment) REQUIRED\n");
        }
        if report.requires_conformity {
            text.push_str("⚠️  Conformity Assessment REQUIRED\n");
        }

        text.push_str("\n--- FINDINGS ---\n\n");
        for finding in &report.findings {
            let icon = match finding.status {
                ComplianceStatus::Compliant => "✅",
                ComplianceStatus::PartiallyCompliant => "⚠️",
                ComplianceStatus::NonCompliant => "❌",
            };
            text.push_str(&format!(
                "{} Article {}: {}\n",
                icon, finding.article, finding.requirement
            ));
            text.push_str(&format!("   {}\n\n", finding.detail));
        }

        text.push_str(&format!("\nGenerated: {}\n", report.generated_at));

        text
    }
}

impl Default for EuAiActExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Compliance report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub generated_at: String,
    pub system_name: String,
    pub risk_level: RiskLevel,
    pub overall_status: OverallStatus,
    pub score: u32,
    pub findings: Vec<ComplianceFinding>,
    pub requires_fria: bool,
    pub requires_conformity: bool,
}

/// Overall compliance status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverallStatus {
    Compliant,
    PartiallyCompliant,
    NonCompliant,
}

/// Individual compliance finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub article: String,
    pub requirement: String,
    pub status: ComplianceStatus,
    pub detail: String,
}

/// Status of individual requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    PartiallyCompliant,
    NonCompliant,
}

// ============================================================================
// 2026 ROADMAP: Incident Reporting (Per Audit Gap Analysis)
// ============================================================================

/// EU AI Act Incident Report (Article 62).
///
/// Per Audit: "Automate reporting to EU national regulators via standardized A2A protocols"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentReport {
    /// Unique incident ID
    pub incident_id: String,
    /// System affected
    pub system_name: String,
    /// Provider information
    pub provider: ProviderInfo,
    /// Incident type
    pub incident_type: IncidentType,
    /// Description of the incident
    pub description: String,
    /// Affected users count
    pub affected_users: Option<u64>,
    /// Severity level
    pub severity: IncidentSeverity,
    /// Corrective actions taken
    pub corrective_actions: Vec<String>,
    /// Timestamp of incident
    pub occurred_at: String,
    /// Timestamp of report
    pub reported_at: String,
}

/// Types of incidents per Article 62.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentType {
    /// Serious incident causing death or injury
    SeriousIncident,
    /// Malfunction leading to safety risk
    SafetyMalfunction,
    /// Fundamental rights violation
    RightsViolation,
    /// Significant bias detected
    BiasIncident,
    /// Security breach
    SecurityBreach,
}

/// Incident severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Incident reporter for EU national authorities.
pub struct IncidentReporter {
    /// Provider information
    provider: ProviderInfo,
    /// Reporting endpoint (per member state)
    endpoints: HashMap<String, String>,
}

impl IncidentReporter {
    /// Create a new incident reporter.
    pub fn new(provider: ProviderInfo) -> Self {
        let mut endpoints = HashMap::new();
        // EU Member State notified bodies (placeholder URLs)
        endpoints.insert("DE".into(), "https://ai-registry.bfdi.de/incidents".into());
        endpoints.insert("FR".into(), "https://ai-registry.cnil.fr/incidents".into());
        endpoints.insert("NL".into(), "https://ai-registry.autoriteitpersoonsgegevens.nl/incidents".into());
        endpoints.insert("IT".into(), "https://ai-registry.gpdp.it/incidents".into());
        endpoints.insert("ES".into(), "https://ai-registry.aepd.es/incidents".into());

        Self { provider, endpoints }
    }

    /// Generate an incident report.
    pub fn create_report(
        &self,
        system_name: &str,
        incident_type: IncidentType,
        description: &str,
        severity: IncidentSeverity,
    ) -> IncidentReport {
        IncidentReport {
            incident_id: uuid::Uuid::new_v4().to_string(),
            system_name: system_name.into(),
            provider: self.provider.clone(),
            incident_type,
            description: description.into(),
            affected_users: None,
            severity,
            corrective_actions: Vec::new(),
            occurred_at: chrono::Utc::now().to_rfc3339(),
            reported_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Submit incident to relevant authority (async placeholder).
    /// In production: Uses A2A protocol to submit to national regulator.
    pub fn submit_report(&self, report: &IncidentReport, member_state: &str) -> Result<String, String> {
        if let Some(endpoint) = self.endpoints.get(member_state) {
            // In production: HTTP POST to regulator endpoint
            tracing::info!(
                incident_id = %report.incident_id,
                member_state = member_state,
                endpoint = %endpoint,
                "Submitting incident report to EU authority"
            );
            Ok(format!("ACK-{}", &report.incident_id[..8]))
        } else {
            Err(format!("No endpoint configured for member state: {}", member_state))
        }
    }

    /// Export report as JSON for manual submission.
    pub fn export_json(&self, report: &IncidentReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }
}

// ============================================================================
// 2026 ROADMAP: Live Bias Detection (Per Audit Gap Analysis)
// ============================================================================

/// Live bias detection result.
///
/// Per Audit: "Integrate the Arbiter bias detection service with live training data streams"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasDetectionResult {
    /// Metric name
    pub metric: String,
    /// Protected attribute (e.g., gender, race, age)
    pub protected_attribute: String,
    /// Measured value
    pub value: f64,
    /// Threshold for concern
    pub threshold: f64,
    /// Whether bias is detected
    pub bias_detected: bool,
    /// Recommended action
    pub recommendation: String,
}

/// Live bias detector for training data streams.
pub struct LiveBiasDetector {
    /// Thresholds per metric
    thresholds: HashMap<String, f64>,
}

impl LiveBiasDetector {
    /// Create a new bias detector with default thresholds.
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        // Common fairness metrics thresholds
        thresholds.insert("demographic_parity".into(), 0.8);
        thresholds.insert("equalized_odds".into(), 0.8);
        thresholds.insert("predictive_parity".into(), 0.8);
        thresholds.insert("statistical_parity_difference".into(), 0.1);
        thresholds.insert("disparate_impact".into(), 0.8);

        Self { thresholds }
    }

    /// Analyze a training data batch for bias.
    pub fn analyze_batch(
        &self,
        metric: &str,
        protected_attribute: &str,
        group_a_rate: f64,
        group_b_rate: f64,
    ) -> BiasDetectionResult {
        let value = if group_b_rate > 0.0 {
            group_a_rate / group_b_rate
        } else {
            1.0
        };

        let threshold = *self.thresholds.get(metric).unwrap_or(&0.8);
        let bias_detected = value < threshold || value > (1.0 / threshold);

        let recommendation = if bias_detected {
            format!(
                "Bias detected in '{}' for attribute '{}'. Consider rebalancing training data or applying fairness constraints.",
                metric, protected_attribute
            )
        } else {
            "No significant bias detected.".into()
        };

        BiasDetectionResult {
            metric: metric.into(),
            protected_attribute: protected_attribute.into(),
            value,
            threshold,
            bias_detected,
            recommendation,
        }
    }

    /// Stream handler for live training data.
    /// Returns bias alerts for real-time monitoring.
    pub fn on_training_batch(
        &self,
        batch_id: &str,
        metrics: Vec<(String, String, f64, f64)>, // (metric, attr, group_a, group_b)
    ) -> Vec<BiasDetectionResult> {
        let results: Vec<_> = metrics
            .into_iter()
            .map(|(metric, attr, ga, gb)| self.analyze_batch(&metric, &attr, ga, gb))
            .collect();

        // Log any bias alerts
        for result in &results {
            if result.bias_detected {
                tracing::warn!(
                    batch_id = batch_id,
                    metric = %result.metric,
                    attribute = %result.protected_attribute,
                    value = result.value,
                    "Bias detected in training batch"
                );
            }
        }

        results
    }

    /// Set custom threshold for a metric.
    pub fn set_threshold(&mut self, metric: &str, threshold: f64) {
        self.thresholds.insert(metric.into(), threshold);
    }
}

impl Default for LiveBiasDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_documentation() -> TechnicalDocumentation {
        TechnicalDocumentation {
            description: SystemDescription {
                name: "AgentKern Agent".into(),
                purpose: "AI agent orchestration and verification".into(),
                version: "1.0.0".into(),
                provider: ProviderInfo {
                    name: "AgentKern Inc".into(),
                    address: "123 AI Street".into(),
                    contact_email: "compliance@agentkern.com".into(),
                    eu_representative: Some("EU Rep Ltd".into()),
                },
                deployment_date: Some("2025-01-01".into()),
                risk_level: RiskLevel::HighRisk,
                high_risk_categories: vec![HighRiskCategory::CriticalInfrastructure],
                techniques: vec!["LLM".into(), "RAG".into()],
            },
            design: DesignSpecifications {
                architecture: "Microservices with Rust core".into(),
                algorithms: vec!["Transformer".into(), "Vector similarity".into()],
                io_specs: IoSpecifications {
                    inputs: vec![DataSpecification {
                        name: "prompt".into(),
                        data_type: "string".into(),
                        description: "User query".into(),
                    }],
                    outputs: vec![DataSpecification {
                        name: "response".into(),
                        data_type: "string".into(),
                        description: "Agent response".into(),
                    }],
                },
                resources: ResourceRequirements {
                    compute: "8 vCPU".into(),
                    memory: "32GB RAM".into(),
                    storage: "100GB SSD".into(),
                },
                dependencies: vec!["OpenAI API".into()],
            },
            risk_management: RiskManagement {
                methodology: "ISO 31000".into(),
                risks: vec![IdentifiedRisk {
                    id: "R001".into(),
                    description: "Prompt injection".into(),
                    likelihood: "Medium".into(),
                    impact: "High".into(),
                    affected_rights: vec!["Privacy".into()],
                }],
                mitigations: vec![MitigationMeasure {
                    risk_id: "R001".into(),
                    measure: "PromptGuard module".into(),
                    effectiveness: "High".into(),
                }],
                residual_risks: vec!["Novel attack vectors".into()],
                testing: TestingProcedures {
                    unit_tests: 320,
                    integration_tests: 50,
                    adversarial_tests: 20,
                    test_datasets: vec!["OWASP LLM Top 10".into()],
                    coverage_percentage: 85.0,
                },
            },
            data: DataGovernance {
                training_data: DatasetInfo {
                    description: "Not applicable (using pre-trained models)".into(),
                    size: "N/A".into(),
                    sources: vec![],
                    collection_period: None,
                },
                validation_data: DatasetInfo {
                    description: "Internal test suite".into(),
                    size: "1000 samples".into(),
                    sources: vec!["Internal".into()],
                    collection_period: None,
                },
                test_data: DatasetInfo {
                    description: "Adversarial test set".into(),
                    size: "500 samples".into(),
                    sources: vec!["OWASP".into()],
                    collection_period: None,
                },
                quality_measures: vec!["Manual review".into(), "Automated validation".into()],
                bias_mitigation: BiasMitigation {
                    detection_methods: vec!["Fairness metrics".into()],
                    mitigation_actions: vec!["Balanced sampling".into()],
                    monitoring: "Quarterly review".into(),
                },
            },
            human_oversight: HumanOversight {
                capability: "Full override via Arbiter kill switch".into(),
                interface: "Web dashboard (Cockpit)".into(),
                stop_mechanism: "Emergency kill switch with <1s response".into(),
                operator_training: "Required certification program".into(),
                monitoring_frequency: "Real-time with alerting".into(),
            },
            performance: PerformanceMetrics {
                accuracy: [("precision".into(), 0.95), ("recall".into(), 0.92)]
                    .into_iter()
                    .collect(),
                robustness: vec!["Adversarial testing".into(), "Chaos engineering".into()],
                consistency: "99.9% consistent responses".into(),
                limitations: vec!["May hallucinate on rare topics".into()],
            },
            cybersecurity: CybersecurityMeasures {
                certifications: vec!["SOC2".into(), "ISO 27001".into()],
                vulnerability_management: "Weekly scans, 24h critical patch".into(),
                incident_response: "24/7 SOC with <15min response".into(),
                access_control: "RBAC with MFA".into(),
                encryption: "TLS 1.3, AES-256-GCM at rest".into(),
            },
        }
    }

    #[test]
    fn test_risk_level_requirements() {
        assert!(!RiskLevel::Minimal.requires_fria());
        assert!(!RiskLevel::Limited.requires_conformity());
        assert!(RiskLevel::HighRisk.requires_fria());
        assert!(RiskLevel::HighRisk.requires_conformity());
    }

    #[test]
    fn test_generate_report() {
        let exporter = EuAiActExporter::new();
        let doc = sample_documentation();

        let report = exporter.generate_report(&doc);

        assert_eq!(report.system_name, "AgentKern Agent");
        assert_eq!(report.risk_level, RiskLevel::HighRisk);
        assert!(report.requires_fria);
    }

    #[test]
    fn test_compliance_score() {
        let exporter = EuAiActExporter::new();
        let doc = sample_documentation();

        let report = exporter.generate_report(&doc);

        // Should be mostly compliant
        assert!(report.score >= 80);
    }

    #[test]
    fn test_export_json() {
        let exporter = EuAiActExporter::new();
        let doc = sample_documentation();

        let json = exporter.export_json(&doc).unwrap();

        assert!(json.contains("AgentKern Agent"));
        assert!(json.contains("high_risk"));
    }

    #[test]
    fn test_export_text() {
        let exporter = EuAiActExporter::new();
        let doc = sample_documentation();

        let text = exporter.export_text(&doc);

        assert!(text.contains("EU AI ACT COMPLIANCE REPORT"));
        assert!(text.contains("AgentKern Agent"));
    }

    #[test]
    fn test_incomplete_documentation() {
        let exporter = EuAiActExporter::new();
        let mut doc = sample_documentation();
        doc.risk_management.risks.clear();
        doc.human_oversight.stop_mechanism.clear();

        let report = exporter.generate_report(&doc);

        assert!(report.score < 80);
        assert!(report.findings.iter().any(|f| f.article == "9"));
    }
}
