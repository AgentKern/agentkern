use crate::models::{LiabilityProof, LiabilityProofPayload, VerificationKey};
use agentkern_crypto::{CryptoProvider, CryptoMode}; // Signature removed if unused
use chrono::{DateTime, Utc, Timelike};
use serde_json;
use thiserror::Error;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

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
    #[error("Internal error: {0}")]
    Internal(String),
}

pub struct VerificationService {
    // In a real implementation, we might inject a Repo here,
    // but for now we'll assume the caller passes the Key for purity.
    crypto_hybrid: CryptoProvider,
}

impl VerificationService {
    pub fn new() -> Self {
        Self {
            crypto_hybrid: CryptoProvider::new(CryptoMode::Hybrid),
        }
    }

    /// Parse the generic "header" string format: version.payloadBase64.signature
    pub fn parse_header(&self, header: &str) -> Result<LiabilityProof, VerificationError> {
        let parts: Vec<&str> = header.split('.').collect();
        if parts.len() != 3 {
            return Err(VerificationError::InvalidFormat);
        }

        let version = parts[0].to_string();
        let payload_b64 = parts[1];
        let signature = parts[2].to_string();

        let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64)
            .map_err(|_| VerificationError::InvalidFormat)?;

        // Fix: Use generic FromReader or verify structure
        let payload: LiabilityProofPayload = serde_json::from_slice(&payload_bytes)
            .map_err(|_| VerificationError::InvalidFormat)?;

        Ok(LiabilityProof {
            version,
            payload,
            signature,
        })
    }

    /// Verify a proof against a known public key
    pub fn verify(&self, proof: &LiabilityProof, key: &VerificationKey) -> Result<bool, VerificationError> {
        let now = Utc::now();

        // 1. Check Expiration
        let expires_at = DateTime::parse_from_rfc3339(&proof.payload.expires_at)
            .map_err(|_| VerificationError::InvalidFormat)?
            .with_timezone(&Utc);

        if expires_at < now {
            return Err(VerificationError::Expired(proof.payload.expires_at.clone()));
        }

        // 2. Check Issue Time
        let issued_at = DateTime::parse_from_rfc3339(&proof.payload.issued_at)
            .map_err(|_| VerificationError::InvalidFormat)?
            .with_timezone(&Utc);

        if issued_at > now {
            return Err(VerificationError::FutureIssue(proof.payload.issued_at.clone()));
        }

        // 3. Verify Constraints (Time of Day)
        if let Some(constraints) = &proof.payload.constraints {
            if let Some(valid_hours) = &constraints.valid_hours {
                let current_hour = now.hour() as u8;
                if current_hour < valid_hours.start || current_hour > valid_hours.end {
                    return Err(VerificationError::ConstraintViolation(
                        format!("Current hour {} outside allowed {}-{}", current_hour, valid_hours.start, valid_hours.end)
                    ));
                }
            }
        }

        // 4. Verify Signature
        self.verify_signature(proof, key)
    }

    fn verify_signature(&self, proof: &LiabilityProof, _key: &VerificationKey) -> Result<bool, VerificationError> {
        // Reconstruct signed data: payload is what was signed.
        // In JWT world, typically header.payload is signed. Here, likely just payload JSON.
        // The Node.js code said: `Buffer.from(payloadJson).toString('base64url')...` but then used jose generic verify.
        // Actually Node.js code: `compactVerify( header stuff + . + sig )`.
        // Wait, Node.js code reconstructs: `eyJhbGciOiJFUzI1NiJ9.${payloadB64}.${classicSig}`
        // This effectively wraps it in a JWT structure for `jose`.

        // If we use agentkern-crypto, we just need the data + sig + key.
        // The data signed is likely the JSON string of the payload.
        let payload_json = serde_json::to_string(&proof.payload)
            .map_err(|e| VerificationError::Internal(e.to_string()))?;
        let _data_bytes = payload_json.as_bytes();

        // Check for hybrid signature (separator ~)
        let (_classic_sig, pqc_sig) = if proof.signature.contains('~') {
            let parts: Vec<&str> = proof.signature.split('~').collect();
            (parts[0], Some(parts[1]))
        } else {
            (proof.signature.as_str(), None)
        };

        // TODO: Import Key from PEM/String to internal crypto format?
        // agentkern-crypto likely takes raw bytes or PEM.
        // Assuming verify(data, sig_bytes, public_key_bytes/pem)

        // Verify Classic (ES256)
        // ... integration with crypto lib ...
        // For scaffold, we return True if crypto lib compiles

        // PQC Check
        if let Some(_pqc) = pqc_sig {
             // self.crypto_hybrid.verify_hybrid(...)
        }

        Ok(true) // Placeholder until crypto lib integration specifics are verified
    }
}
