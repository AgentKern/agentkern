//! Entity Formation
//!
//! Global entity formation supporting multiple frameworks:
//! - LLC (US/EU)
//! - Takaful cooperative (MENA/SEA)
//! - Waqf endowment (Islamic)
//! - DAO (Blockchain-native)

use serde::{Deserialize, Serialize};

/// Entity type for agent legal structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    /// Limited Liability Company (US, EU, etc.)
    Llc,
    /// Corporation (C-Corp, S-Corp, etc.)
    Corporation,
    /// Takaful - Islamic mutual risk sharing
    Takaful,
    /// Waqf - Islamic endowment/trust
    Waqf,
    /// Decentralized Autonomous Organization
    Dao,
    /// Partnership
    Partnership,
    /// Sole proprietorship (agent-as-individual)
    Individual,
}

impl EntityType {
    /// Get jurisdictions where this entity type is available.
    pub fn available_jurisdictions(&self) -> &[&str] {
        match self {
            Self::Llc => &["US", "EU", "UK", "SG", "AE"],
            Self::Corporation => &["US", "EU", "UK", "JP", "CN"],
            Self::Takaful => &["MY", "SA", "AE", "BH", "PK", "ID"],
            Self::Waqf => &["MY", "SA", "AE", "TR", "PK"],
            Self::Dao => &["WY", "TN", "UT", "CH"], // US states + Switzerland
            Self::Partnership => &["US", "EU", "UK"],
            Self::Individual => &["*"], // Global
        }
    }
    
    /// Check if this entity type is Shariah-compliant by default.
    pub fn is_shariah_compliant(&self) -> bool {
        matches!(self, Self::Takaful | Self::Waqf)
    }
}

/// Formation request for an agent entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationRequest {
    /// Agent DID
    pub agent_did: String,
    /// Desired entity type
    pub entity_type: EntityType,
    /// Target jurisdiction
    pub jurisdiction: String,
    /// Entity name
    pub entity_name: String,
    /// Initial capital (in smallest currency unit)
    pub initial_capital: Option<u64>,
    /// Require Shariah compliance
    pub require_shariah: bool,
    /// ESG compliance level
    pub esg_level: Option<EsgLevel>,
}

/// ESG compliance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EsgLevel {
    /// No ESG requirements
    None,
    /// Basic ESG screening
    Basic,
    /// Full ESG compliance
    Full,
    /// Impact-first (B-Corp style)
    Impact,
}

/// Formation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationResult {
    /// Entity ID
    pub entity_id: String,
    /// Registration number
    pub registration_number: String,
    /// Jurisdiction of formation
    pub jurisdiction: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Shariah certified
    pub shariah_certified: bool,
    /// Certification body (if applicable)
    pub certifier: Option<String>,
    /// Formation timestamp
    pub formed_at: String,
}

/// Entity formation service trait.
pub trait EntityFormation: Send + Sync {
    /// Form a new legal entity for an agent.
    fn form_entity(&self, request: &FormationRequest) -> Result<FormationResult, FormationError>;
    
    /// Check if formation is possible in jurisdiction.
    fn check_availability(&self, entity_type: EntityType, jurisdiction: &str) -> bool;
    
    /// Get estimated formation cost.
    fn estimate_cost(&self, entity_type: EntityType, jurisdiction: &str) -> Option<u64>;
}

/// Formation error.
#[derive(Debug, thiserror::Error)]
pub enum FormationError {
    #[error("Jurisdiction not supported: {0}")]
    UnsupportedJurisdiction(String),
    
    #[error("Entity type not available in jurisdiction")]
    EntityTypeNotAvailable,
    
    #[error("Shariah compliance not available for entity type")]
    ShariahNotAvailable,
    
    #[error("Insufficient capital: required {required}, provided {provided}")]
    InsufficientCapital { required: u64, provided: u64 },
    
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_jurisdictions() {
        assert!(EntityType::Takaful.available_jurisdictions().contains(&"MY"));
        assert!(EntityType::Takaful.available_jurisdictions().contains(&"SA"));
    }

    #[test]
    fn test_shariah_compliant_types() {
        assert!(EntityType::Takaful.is_shariah_compliant());
        assert!(EntityType::Waqf.is_shariah_compliant());
        assert!(!EntityType::Llc.is_shariah_compliant());
    }
}
