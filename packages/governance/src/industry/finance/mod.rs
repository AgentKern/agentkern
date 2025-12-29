//! Finance Compliance Module
//!
//! - PCI-DSS (Payment Card Industry Data Security Standard)
//! - Shariah Compliance (Islamic Finance)
//! - Basel III (Banking regulation)

pub mod pci;
pub mod shariah;

// Explicit exports to avoid ambiguous re-exports of RiskLevel
pub use pci::{CardBrand, CardDataType, CardScanResult, CardToken, PciError, PciValidator, RiskLevel as PciRiskLevel};
pub use shariah::{
    ComplianceResult, RiskLevel as ShariahRiskLevel, ShariahComplianceError,
    ShariahComplianceValidator, TransactionDetails, TransactionType,
};

