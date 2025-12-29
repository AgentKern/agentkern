//! AgentKern-Compliance: Industry-Specific Compliance Modules
//!
//! **NOTE**: This crate is being consolidated into `agentkern-governance`.
//! For new development, use:
//! - `agentkern_governance::industry::healthcare` for HIPAA
//! - `agentkern_governance::industry::finance` for PCI-DSS, Shariah
//!
//! This crate remains for backward compatibility.

pub mod hipaa;
pub mod pci;
pub mod shariah_compliance;

// Re-exports for backward compatibility
pub use hipaa::{HipaaError, HipaaRole, HipaaValidator, PhiScanResult};
pub use pci::{CardBrand, CardScanResult, CardToken, PciError, PciValidator, RiskLevel};
pub use shariah_compliance::{
    ComplianceResult, ShariahComplianceError, ShariahComplianceValidator,
};

// Future: re-export from governance
// pub use agentkern_governance::industry::healthcare as hipaa_v2;
// pub use agentkern_governance::industry::finance as finance_v2;
