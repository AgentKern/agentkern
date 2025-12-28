//! AgentKern-Gate: Core Types
//!
//! Domain types for the verification engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request for action verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    /// Unique request ID
    pub request_id: Uuid,
    /// Agent requesting the action
    pub agent_id: String,
    /// Action being requested (e.g., "send_email", "transfer_funds")
    pub action: String,
    /// Context for policy evaluation
    pub context: VerificationContext,
    /// Timestamp of the request
    pub timestamp: DateTime<Utc>,
}

/// Context for policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerificationContext {
    /// Key-value pairs for policy evaluation
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// Result of action verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Request ID for correlation
    pub request_id: Uuid,
    /// Was the action allowed?
    pub allowed: bool,
    /// Policies that were evaluated
    pub evaluated_policies: Vec<String>,
    /// Policies that blocked the action
    pub blocking_policies: Vec<String>,
    /// Risk score from symbolic evaluation (0-100)
    pub symbolic_risk_score: u8,
    /// Risk score from neural evaluation (0-100), if triggered
    pub neural_risk_score: Option<u8>,
    /// Combined final risk score
    pub final_risk_score: u8,
    /// Human-readable reasoning
    pub reasoning: String,
    /// Latency breakdown
    pub latency: LatencyBreakdown,
}

/// Latency breakdown for performance monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyBreakdown {
    /// Total latency in microseconds
    pub total_us: u64,
    /// Symbolic path latency in microseconds
    pub symbolic_us: u64,
    /// Neural path latency in microseconds (if triggered)
    pub neural_us: Option<u64>,
}

/// Data residency region for sovereignty compliance.
/// 
/// Per December 2025 research on AWS/Azure regional strategies:
/// - Tier 1: Major regulatory blocs with strict localization
/// - Tier 2: Emerging sovereignty blocs with specific laws
/// - Tier 3: Regional fallbacks for broader compliance
/// 
/// Reference: GLOBAL_GAPS.md, ENGINEERING_STANDARD.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataRegion {
    // ═══════════════════════════════════════════════════════════════
    // Tier 1: Major Regulatory Blocs (strict localization required)
    // ═══════════════════════════════════════════════════════════════
    
    /// United States (HIPAA, CCPA, SOX, Sales Tax)
    Us,
    /// European Union (GDPR, EU Data Act 2025, VAT)
    Eu,
    /// China (PIPL - requires in-country processing)
    Cn,
    
    // ═══════════════════════════════════════════════════════════════
    // Tier 2: Emerging Sovereignty Blocs (country-specific laws)
    // ═══════════════════════════════════════════════════════════════
    
    /// Middle East & North Africa (GCC Vision 2030, Saudi PDPL, Islamic Finance/Takaful)
    Mena,
    /// India (DPDP Act 2023 - strict consent, purpose limitation)
    India,
    /// Brazil (LGPD - similar to GDPR)
    Brazil,
    
    // ═══════════════════════════════════════════════════════════════
    // Tier 3: Regional Fallbacks (less strict, grouped compliance)
    // ═══════════════════════════════════════════════════════════════
    
    /// Asia-Pacific (Singapore PDPA, Japan APPI, Korea PIPA, Australia Privacy Act)
    AsiaPac,
    /// Africa (varying data localization by country)
    Africa,
    
    // ═══════════════════════════════════════════════════════════════
    // Default
    // ═══════════════════════════════════════════════════════════════
    
    /// Global (no specific residency, universal policies)
    Global,
}

impl DataRegion {
    /// Returns true if this region requires strict data localization.
    pub fn requires_localization(&self) -> bool {
        matches!(self, Self::Cn | Self::Eu | Self::India | Self::Mena)
    }
    
    /// Returns the privacy law name for this region.
    pub fn privacy_law(&self) -> &'static str {
        match self {
            Self::Us => "HIPAA/CCPA",
            Self::Eu => "GDPR",
            Self::Cn => "PIPL",
            Self::Mena => "Saudi PDPL/GCC",
            Self::India => "DPDP Act 2023",
            Self::Brazil => "LGPD",
            Self::AsiaPac => "PDPA/APPI/PIPA",
            Self::Africa => "Various",
            Self::Global => "None",
        }
    }
}

impl Default for DataRegion {
    fn default() -> Self {
        Self::Global
    }
}


pub type AgentId = String;
