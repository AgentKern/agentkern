use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Algorithm types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    ES256,
    EdDSA,
    // Add Hybrid-PQC if needed, e.g. "Dilithium3"
    HybridPQC,
    Other(String),
}

impl Default for Algorithm {
    fn default() -> Self {
        Self::ES256
    }
}

/// Key Format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "key_format", rename_all = "lowercase")]
pub enum KeyFormat {
    Pem,
    Jwk,
}

impl Default for KeyFormat {
    fn default() -> Self {
        Self::Pem
    }
}

/// Verification Key for Liability Proofs
///
/// Corresponds to `verification_keys` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VerificationKey {
    /// Unique Key ID
    pub id: Uuid,

    /// The Principal (User/Agent) ID owning this key
    #[sqlx(rename = "principal_id")]
    pub principal_id: String,

    /// Namespace isolation
    pub namespace: String,

    /// Credential ID (e.g. from WebAuthn or KeyPair)
    #[sqlx(rename = "credential_id")]
    pub credential_id: String,

    /// Public Key Material
    #[sqlx(rename = "public_key")]
    pub public_key: String,

    /// Algorithm (string in DB)
    pub algorithm: String,

    /// Format (enum in DB or string)
    // We treat it as string in DB for compatibility if no enum type exists yet,
    // but here we used sqlx::Type above. Stick to string for simpler migration if needed?
    // Let's assume postgres enum "key_format" exists or we map string.
    // For safety, let's use String and map manually or use sqlx::Type if types match.
    // Using simple String for robustness against arbitrary DB text.
    pub format: String,

    /// Active status
    pub active: bool,

    /// Expiration
    #[sqlx(rename = "expires_at")]
    pub expires_at: Option<DateTime<Utc>>,

    /// Last Used
    #[sqlx(rename = "last_used_at")]
    pub last_used_at: Option<DateTime<Utc>>,

    /// Usage Count
    #[sqlx(rename = "usage_count")]
    pub usage_count: i32,

    /// Timestamps
    #[sqlx(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    #[sqlx(rename = "updated_at")]
    pub updated_at: DateTime<Utc>,
}
