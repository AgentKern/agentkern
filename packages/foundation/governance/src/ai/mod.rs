//! AI Governance Module
//!
//! Regulations specific to AI systems:
//! - EU AI Act (Article 13, 14, 62)
//! - ISO/IEC 42001 (AIMS)
//! - Bias detection and mitigation

pub mod eu_ai_act;
pub mod iso42001;

// Explicit exports to avoid ambiguous re-exports of HumanOversight and ComplianceFinding
// Use type aliases to disambiguate identical names in different modules
pub use eu_ai_act::{
    BiasDetectionResult, ComplianceFinding as EuComplianceFinding, ComplianceReport,
    ComplianceStatus, CybersecurityMeasures, DataGovernance, EuAiActExporter, HighRiskCategory,
    HumanOversight as EuHumanOversight, IncidentReport, IncidentReporter, LiveBiasDetector,
    OverallStatus, PerformanceMetrics, RiskLevel, RiskManagement, TechnicalDocumentation,
};
pub use iso42001::{
    AuditEvent, AuditOutcome, ComplianceFinding as IsoComplianceFinding, ComplianceLedger,
    FindingSeverity, HumanOversight as IsoHumanOversight,
    report::{AuditReport, ReportFormat, ReportGenerator},
};
