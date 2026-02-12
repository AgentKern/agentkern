//! Agent Legal Entity Module
//!
//! Global compliance frameworks for agent legal entities:
//! - LLC/Corp (US/EU)
//! - DAO (Decentralized Autonomous Organization)
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod formation;
// pub mod liability;

pub use formation::{EntityFormation, EntityType, FormationRequest, FormationResult};
// pub use liability::{LiabilityProtection, LiabilityModel, CoverageType};
