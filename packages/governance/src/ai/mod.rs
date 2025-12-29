//! AI Governance Module
//!
//! Regulations specific to AI systems:
//! - EU AI Act (Article 13, 14, 62)
//! - ISO/IEC 42001 (AIMS)
//! - Bias detection and mitigation

mod eu_ai_act;
pub mod iso42001;

pub use eu_ai_act::*;
pub use iso42001::*;
