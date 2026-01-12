use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub id: String,
    #[serde(rename = "credentialId")]
    pub credential_id: String,
    #[serde(rename = "deviceAttestation")]
    pub device_attestation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentTarget {
    pub service: String,
    pub endpoint: String,
    pub method: String, // Enum in TS, string here for flexibility (GET, POST, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub action: String,
    pub target: IntentTarget,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidHours {
    pub start: u8,
    pub end: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    #[serde(rename = "maxAmount")]
    pub max_amount: Option<f64>,
    #[serde(rename = "allowedRecipients")]
    pub allowed_recipients: Option<Vec<String>>,
    #[serde(rename = "geoFence")]
    pub geo_fence: Option<Vec<String>>,
    #[serde(rename = "validHours")]
    pub valid_hours: Option<ValidHours>,
    #[serde(rename = "requireConfirmationAbove")]
    pub require_confirmation_above: Option<f64>,
    #[serde(rename = "singleUse")]
    pub single_use: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liability {
    #[serde(rename = "acceptedBy")]
    pub accepted_by: String, // 'principal' | 'agent_operator'
    #[serde(rename = "termsVersion")]
    pub terms_version: String,
    #[serde(rename = "disputeWindowHours")]
    pub dispute_window_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilityProofPayload {
    pub version: String,
    #[serde(rename = "proofId")]
    pub proof_id: String,
    #[serde(rename = "issuedAt")]
    pub issued_at: String, // ISO date string
    #[serde(rename = "expiresAt")]
    pub expires_at: String, // ISO date string
    pub principal: Principal,
    pub agent: AgentInfo,
    pub intent: Intent,
    pub constraints: Option<Constraints>,
    pub liability: Liability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilityProof {
    pub version: String,
    pub payload: LiabilityProofPayload,
    pub signature: String,
}
