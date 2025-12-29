//! Liability Protection Models
//!
//! Global liability frameworks:
//! - LLC/Corp: Traditional limited liability
//! - Takaful: Islamic mutual risk sharing
//! - DAO: Smart contract-based protection
//! - Insurance: Traditional coverage binding

use serde::{Deserialize, Serialize};

/// Liability protection model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiabilityModel {
    /// Traditional LLC/Corporation - shareholders protected
    CorporateLiability,
    /// Takaful - mutual pooling, no insurer profit
    TakafulMutual,
    /// Conventional insurance
    ConventionalInsurance,
    /// DAO treasury-backed
    DaoTreasury,
    /// Self-insured (agent holds reserves)
    SelfInsured,
    /// No protection (personal liability)
    None,
}

impl LiabilityModel {
    /// Is this model Shariah-compliant?
    pub fn is_shariah_compliant(&self) -> bool {
        match self {
            Self::TakafulMutual | Self::SelfInsured | Self::DaoTreasury => true,
            Self::ConventionalInsurance => false, // Contains riba/gharar
            Self::CorporateLiability => true, // Structure is permissible
            Self::None => true,
        }
    }
    
    /// Get model description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::CorporateLiability => "Shareholders protected from personal liability",
            Self::TakafulMutual => "Mutual pooling - participants share risk and surplus",
            Self::ConventionalInsurance => "Insurance company bears risk for premium",
            Self::DaoTreasury => "Smart contract holds reserves for claims",
            Self::SelfInsured => "Agent maintains reserve fund for claims",
            Self::None => "No liability protection - full personal exposure",
        }
    }
}

/// Coverage type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverageType {
    /// General liability
    General,
    /// Professional/errors & omissions
    Professional,
    /// Cyber liability
    Cyber,
    /// Directors & officers
    DirectorsOfficers,
    /// Product liability
    Product,
}

/// Liability protection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilityProtection {
    /// Primary model
    pub model: LiabilityModel,
    /// Coverage types
    pub coverages: Vec<CoverageType>,
    /// Coverage limit (in smallest currency unit)
    pub limit: u64,
    /// Deductible
    pub deductible: u64,
    /// Currency
    pub currency: String,
    /// Takaful pool ID (if applicable)
    pub takaful_pool: Option<String>,
}

impl LiabilityProtection {
    /// Create Takaful protection.
    pub fn takaful(pool_id: &str, limit: u64, currency: &str) -> Self {
        Self {
            model: LiabilityModel::TakafulMutual,
            coverages: vec![CoverageType::General, CoverageType::Professional],
            limit,
            deductible: limit / 100, // 1% deductible
            currency: currency.into(),
            takaful_pool: Some(pool_id.into()),
        }
    }
    
    /// Create standard corporate protection.
    pub fn corporate(limit: u64, currency: &str) -> Self {
        Self {
            model: LiabilityModel::CorporateLiability,
            coverages: vec![CoverageType::General],
            limit,
            deductible: 0,
            currency: currency.into(),
            takaful_pool: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shariah_compliant_models() {
        assert!(LiabilityModel::TakafulMutual.is_shariah_compliant());
        assert!(!LiabilityModel::ConventionalInsurance.is_shariah_compliant());
    }

    #[test]
    fn test_takaful_protection() {
        let protection = LiabilityProtection::takaful("pool-001", 1_000_000, "SAR");
        assert_eq!(protection.model, LiabilityModel::TakafulMutual);
        assert!(protection.takaful_pool.is_some());
    }
}
