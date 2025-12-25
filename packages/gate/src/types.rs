//! VeriMantle-Gate: Core Types
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataRegion {
    /// United States
    Us,
    /// European Union (GDPR)
    Eu,
    /// China (PIPL)
    Cn,
    /// Saudi Arabia (Vision 2030)
    Sa,
    /// India (DPDP)
    In,
    /// Brazil (LGPD)
    Br,
    /// Global (no specific residency)
    Global,
}

impl Default for DataRegion {
    fn default() -> Self {
        Self::Global
    }
}
