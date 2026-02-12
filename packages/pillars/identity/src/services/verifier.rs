// ============================================================================
// VERIFICATION SERVICE
// ============================================================================

use crate::models::{LiabilityProof, LiabilityProofPayload, VerificationKey};
use agentkern_crypto::{CryptoMode, CryptoProvider};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Timelike, Utc};
use serde_json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Invalid proof format")]
    InvalidFormat,
    #[error("Proof expired at {0}")]
    Expired(String),
    #[error("Proof issued in the future: {0}")]
    FutureIssue(String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("Key mismatch or not found")]
    KeyError,
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

pub struct VerificationService {
    // In a real implementation, we might inject a Repo here,
    // but for now we'll assume the caller passes the Key for purity.
    crypto_hybrid: CryptoProvider,
    replay_cache: Box<dyn ReplayCache>,
    revocation_cache: Box<dyn RevocationCache>,
}

impl VerificationService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_caches(
        replay_cache: Box<dyn ReplayCache>,
        revocation_cache: Box<dyn RevocationCache>,
    ) -> Self {
        Self {
            crypto_hybrid: CryptoProvider::new(CryptoMode::Hybrid),
            replay_cache,
            revocation_cache,
        }
    }
}

impl Default for VerificationService {
    fn default() -> Self {
        Self {
            crypto_hybrid: CryptoProvider::new(CryptoMode::Hybrid),
            replay_cache: Box::new(InMemoryReplayCache::new()),
            revocation_cache: Box::new(InMemoryRevocationCache::new()),
        }
    }
}

impl VerificationService {
    /// Parse the generic "header" string format: version.payloadBase64.signature
    pub fn parse_header(&self, header: &str) -> Result<LiabilityProof, VerificationError> {
        let parts: Vec<&str> = header.split('.').collect();
        if parts.len() != 3 {
            return Err(VerificationError::Internal(format!(
                "Invalid parts count: {}, header: {}",
                parts.len(),
                header
            )));
        }

        let version = parts[0].to_string();
        let payload_b64 = parts[1];
        let signature = parts[2].to_string();

        let payload_bytes = URL_SAFE_NO_PAD
            .decode(payload_b64)
            .map_err(|e| VerificationError::Internal(format!("Base64 decode failed: {}", e)))?;

        // Fix: Use generic FromReader or verify structure
        let payload: LiabilityProofPayload = serde_json::from_slice(&payload_bytes)
            .map_err(|e| VerificationError::Internal(format!("JSON parse failed: {}", e)))?;

        Ok(LiabilityProof {
            version,
            payload,
            signature,
            raw_payload: Some(payload_bytes),
        })
    }

    /// Verify a proof against a known public key
    pub async fn verify(
        &self,
        proof: &LiabilityProof,
        key: &VerificationKey,
    ) -> Result<bool, VerificationError> {
        let now = Utc::now();

        // 1. Replay Protection (Critical Security Fix)
        // Check if proof_id has been seen before
        if self.replay_cache.has_seen(&proof.payload.proof_id).await {
            return Err(VerificationError::ConstraintViolation(format!(
                "Replay detected: proof_id {} already used",
                proof.payload.proof_id
            )));
        }

        // 1b. Revocation Check (New)
        if self
            .revocation_cache
            .is_revoked(&proof.payload.proof_id)
            .await
        {
            return Err(VerificationError::ConstraintViolation(format!(
                "Revoked: proof_id {} is found in blacklist",
                proof.payload.proof_id
            )));
        }

        // 2. Check Expiration
        let expires_at = DateTime::parse_from_rfc3339(&proof.payload.expires_at)
            .map_err(|_| VerificationError::InvalidFormat)?
            .with_timezone(&Utc);

        if expires_at < now {
            return Err(VerificationError::Expired(proof.payload.expires_at.clone()));
        }

        // 3. Check Issue Time
        let issued_at = DateTime::parse_from_rfc3339(&proof.payload.issued_at)
            .map_err(|_| VerificationError::InvalidFormat)?
            .with_timezone(&Utc);

        if issued_at > now {
            return Err(VerificationError::FutureIssue(
                proof.payload.issued_at.clone(),
            ));
        }

        // 4. Verify Constraints (Time of Day)
        if let Some(constraints) = &proof.payload.constraints
            && let Some(valid_hours) = &constraints.valid_hours
        {
            let current_hour = now.hour() as u8;
            if current_hour < valid_hours.start || current_hour > valid_hours.end {
                return Err(VerificationError::ConstraintViolation(format!(
                    "Current hour {} outside allowed {}-{}",
                    current_hour, valid_hours.start, valid_hours.end
                )));
            }
        }

        // 5. Verify Signature
        if self.verify_signature(proof, key)? {
            // Mark proof as seen only after successful verification
            // Calculate TTL based on expiration
            let ttl = (expires_at - now).num_seconds().max(60) as u64;
            self.replay_cache
                .mark_seen(&proof.payload.proof_id, ttl)
                .await;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn verify_signature(
        &self,
        proof: &LiabilityProof,
        key: &VerificationKey,
    ) -> Result<bool, VerificationError> {
        // Use raw_payload if available to avoid canonicalization issues
        let data_bytes = if let Some(raw) = &proof.raw_payload {
            raw.clone() // Clone is necessary as verify takes &[u8]
        } else {
            // Fallback: re-serialize (RISKY: assumes consistent formatting)
            let payload_json = serde_json::to_string(&proof.payload)
                .map_err(|e| VerificationError::Internal(e.to_string()))?;
            payload_json.into_bytes()
        };
        let data_bytes = data_bytes.as_slice();

        // Detect signature type (Hybrid vs Classic)
        let (classical_part, pq_part) = if proof.signature.contains('~') {
            let parts: Vec<&str> = proof.signature.split('~').collect();
            (Some(parts[0].to_string()), Some(parts[1].to_string()))
        } else {
            (Some(proof.signature.clone()), None)
        };

        // Map key algorithm string to agentkern_crypto::Algorithm
        let algo = match key.algorithm.as_str() {
            "Ed25519" => agentkern_crypto::Algorithm::Ed25519,
            "P256" => agentkern_crypto::Algorithm::EcdsaP256,
            // Fallback for others - or better, return error
            other => return Err(VerificationError::UnsupportedAlgorithm(other.to_string())),
        };

        // Construct Signature Object for AgentKern-Crypto
        let signature_obj = agentkern_crypto::Signature {
            algorithm: algo,
            value: proof.signature.clone(),
            key_id: key.id.to_string(),
            classical_component: classical_part,
            pq_component: pq_part,
        };

        // Verify using the crypto provider
        self.crypto_hybrid
            .verify(data_bytes, &signature_obj, &key.public_key)
            .map_err(|_| VerificationError::InvalidSignature)
    }
}

// ============================================================================
// REPLAY PROTECTION
// ============================================================================

/// Cache trait for replay protection nonce storage.
#[async_trait::async_trait]
pub trait ReplayCache: Send + Sync {
    /// Check if a proof ID has been seen.
    async fn has_seen(&self, proof_id: &str) -> bool;
    /// Mark a proof ID as seen with a TTL (seconds).
    async fn mark_seen(&self, proof_id: &str, ttl_seconds: u64);
}

/// In-memory implementation of ReplayCache.
/// ⚠️ WARNING: Not suitable for distributed deployments. Use Redis in production.
pub struct InMemoryReplayCache {
    // using std::sync explicitly to avoid conflicts
    cache: std::sync::RwLock<std::collections::HashMap<String, DateTime<Utc>>>,
}

impl Default for InMemoryReplayCache {
    fn default() -> Self {
        Self {
            cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl InMemoryReplayCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Prune expired entries (manual maintenance)
    pub fn prune(&self) {
        let now = Utc::now();
        if let Ok(mut guard) = self.cache.write() {
            guard.retain(|_, expiry| *expiry > now);
        }
    }
}

#[async_trait::async_trait]
impl ReplayCache for InMemoryReplayCache {
    async fn has_seen(&self, proof_id: &str) -> bool {
        let now = Utc::now();
        if let Ok(guard) = self.cache.read()
            && let Some(expiry) = guard.get(proof_id)
        {
            return *expiry > now;
        }
        false
    }

    async fn mark_seen(&self, proof_id: &str, ttl_seconds: u64) {
        let now = Utc::now();
        let expiry = now + chrono::Duration::seconds(ttl_seconds as i64);
        if let Ok(mut guard) = self.cache.write() {
            guard.insert(proof_id.to_string(), expiry);
        }
    }
}

// ============================================================================
// REVOCATION CACHE (BLACKLIST)
// ============================================================================

/// Cache trait for token revocation (blacklist).
#[async_trait::async_trait]
pub trait RevocationCache: Send + Sync {
    /// Check if a proof ID or token ID is revoked.
    async fn is_revoked(&self, id: &str) -> bool;
    /// Revoke a proof ID or token ID with a TTL.
    async fn revoke(&self, id: &str, ttl_seconds: u64);
}

/// In-memory implementation of RevocationCache.
pub struct InMemoryRevocationCache {
    cache: std::sync::RwLock<std::collections::HashMap<String, DateTime<Utc>>>,
}

impl Default for InMemoryRevocationCache {
    fn default() -> Self {
        Self {
            cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl InMemoryRevocationCache {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl RevocationCache for InMemoryRevocationCache {
    async fn is_revoked(&self, id: &str) -> bool {
        let now = Utc::now();
        if let Ok(guard) = self.cache.read()
            && let Some(expiry) = guard.get(id)
        {
            return *expiry > now;
        }
        false
    }

    async fn revoke(&self, id: &str, ttl_seconds: u64) {
        let now = Utc::now();
        let expiry = now + chrono::Duration::seconds(ttl_seconds as i64);
        if let Ok(mut guard) = self.cache.write() {
            guard.insert(id.to_string(), expiry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AgentInfo, Intent, IntentTarget, Liability, LiabilityProofPayload, Principal,
    };
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    fn create_dummy_payload() -> LiabilityProofPayload {
        LiabilityProofPayload {
            version: "1".to_string(),
            proof_id: "test-proof-id".to_string(),
            issued_at: Utc::now().to_rfc3339(),
            expires_at: (Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
            principal: Principal {
                id: "user-1".to_string(),
                credential_id: "cred-1".to_string(),
                device_attestation: None,
            },
            agent: AgentInfo {
                id: "agent-1".to_string(),
                name: "Test Agent".to_string(),
                version: "1.0.0".to_string(),
            },
            intent: Intent {
                action: "test".to_string(),
                target: IntentTarget {
                    service: "test-service".to_string(),
                    endpoint: "/test".to_string(),
                    method: "GET".to_string(),
                },
                parameters: None,
            },
            constraints: None,
            liability: Liability {
                accepted_by: "principal".to_string(),
                terms_version: "1.0".to_string(),
                dispute_window_hours: 24,
            },
        }
    }

    #[test]
    fn test_parse_header_valid() {
        let payload = create_dummy_payload();
        let payload_json = serde_json::to_string(&payload).unwrap();
        let payload_b64 = URL_SAFE_NO_PAD.encode(payload_json);
        let header = format!("1.{}.signature", payload_b64);

        let service = VerificationService::new();
        let result = service.parse_header(&header);
        let proof = result.expect("Parse header failed");
        assert_eq!(proof.version, "1");
        assert_eq!(proof.signature, "signature");
        assert_eq!(proof.payload.proof_id, "test-proof-id");
    }

    #[tokio::test]
    async fn test_verify_expiration() {
        let mut payload = create_dummy_payload();
        // Set expiration in the past
        payload.expires_at = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();

        let proof = LiabilityProof {
            version: "1.0".to_string(),
            payload,
            signature: "sig".to_string(),
            raw_payload: None,
        };

        let key = VerificationKey {
            id: uuid::Uuid::new_v4(),
            principal_id: "user-1".to_string(),
            namespace: "default".to_string(),
            credential_id: "cred-1".to_string(),
            public_key: "dummy-key".to_string(),
            algorithm: "ES256".to_string(),
            format: "pem".to_string(),
            active: true,
            expires_at: None,
            last_used_at: None,
            usage_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let service = VerificationService::new();
        // verify should fail on expiration BEFORE signature check
        let result = service.verify(&proof, &key).await;

        match result {
            Err(VerificationError::Expired(_)) => {}
            _ => panic!("Should have failed with Expired"),
        }
    }
}
