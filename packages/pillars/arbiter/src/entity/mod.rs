//! Agent Legal Entity Module
//!
//! Global compliance frameworks for agent legal entities:
//! - LLC/Corp (US/EU)
//! - Takaful (Islamic mutual pooling)
//! - Waqf (Islamic endowment)
//! - DAO (Decentralized Autonomous Organization)
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod formation;
pub mod liability;
pub mod compliance;
pub mod shariah;
pub mod screening;

pub use formation::{EntityFormation, EntityType, FormationRequest, FormationResult};
pub use liability::{LiabilityProtection, LiabilityModel, CoverageType};
pub use compliance::{JurisdictionCompliance, Jurisdiction, ComplianceCheck};
pub use shariah::{ShariahCompliance, ShariahCheck, ShariahViolation};
pub use screening::{InvestmentScreener, ScreeningCriteria, ScreeningResult};
