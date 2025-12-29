//! AgentKern Governance: Unified Compliance & Regulatory Framework
//!
//! Per EXECUTION_MANDATE.md: "Compliance by Design" architecture
//! Per DDD: Bounded contexts for each regulatory domain
//!
//! # Structure
//!
//! - `ai`: AI-specific regulations (EU AI Act, ISO 42001)
//! - `privacy`: Data privacy (GDPR, CCPA, LGPD, PIPL)
//! - `industry`: Industry-specific (healthcare, finance, government)
//! - `audit`: Cross-cutting audit and evidence collection
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_governance::ai::EuAiActExporter;
//! use agentkern_governance::industry::healthcare::HipaaValidator;
//! use agentkern_governance::privacy::GlobalPrivacyRegistry;
//! ```

pub mod ai;
pub mod audit;
pub mod industry;
pub mod privacy;

// Re-export commonly used types from each domain
pub use ai::{EuAiActExporter, RiskLevel, TechnicalDocumentation};
pub use audit::{AuditLedger, AuditRecord, AuditOutcome, InfrastructureEvidenceCollector};
pub use privacy::GlobalPrivacyRegistry;
