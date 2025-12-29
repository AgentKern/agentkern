//! AgentKern-Compliance: Industry-Specific Compliance Modules
//!
//! Extracted from agentkern-gate core for modularity and WASM compilation.
//!
//! Supported Regulations:
//! - HIPAA: Healthcare data protection
//! - PCI-DSS: Payment card industry
//! - Shariah: Islamic finance compliance

pub mod hipaa;
pub mod pci;
pub mod shariah_compliance;

// Re-exports
pub use hipaa::{HipaaError, HipaaRole, HipaaValidator, PhiScanResult};
pub use pci::{CardBrand, CardScanResult, CardToken, PciError, PciValidator, RiskLevel};
pub use shariah_compliance::{
    ComplianceResult, ShariahComplianceError, ShariahComplianceValidator,
};
