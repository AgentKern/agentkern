//! AgentKern-Crypto: Unified Cryptographic Foundation
//!
//! Provides swappable classical and post-quantum cryptographic primitives.

use base64::Engine;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Cryptographic errors.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Key generation failed: {0}")]
    KeyGeneration(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Invalid key format")]
    InvalidKeyFormat,
}

/// Cryptographic mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CryptoMode {
    Classical,
    PostQuantum,
    #[default]
    Hybrid,
}

/// Cryptographic algorithm per NIST FIPS standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    EcdsaP256,
    Ed25519,
    MlDsa44,
    MlDsa65,
    MlDsa87,
    MlKem512,
    MlKem768,
    MlKem1024,
    HybridEd25519MlDsa,
}

impl Algorithm {
    pub fn is_post_quantum(&self) -> bool {
        matches!(self, Self::MlDsa44 | Self::MlDsa65 | Self::MlDsa87)
    }

    pub fn is_hybrid(&self) -> bool {
        matches!(self, Self::HybridEd25519MlDsa)
    }
}

/// A cryptographic key pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub algorithm: Algorithm,
    pub public_key: String,
    #[serde(skip_serializing)]
    pub private_key: String,
    pub key_id: String,
    pub created_at: u64,
}

/// A cryptographic signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub algorithm: Algorithm,
    pub value: String,
    pub key_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classical_component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pq_component: Option<String>,
}

/// Crypto provider with swappable algorithms.
#[derive(Debug)]
pub struct CryptoProvider {
    mode: CryptoMode,
    signing_algorithm: Algorithm,
}

impl Default for CryptoProvider {
    fn default() -> Self {
        Self::new(CryptoMode::Hybrid)
    }
}

impl CryptoProvider {
    pub fn new(mode: CryptoMode) -> Self {
        let signing = match mode {
            CryptoMode::Classical => Algorithm::Ed25519,
            CryptoMode::PostQuantum => Algorithm::MlDsa65,
            CryptoMode::Hybrid => Algorithm::HybridEd25519MlDsa,
        };

        Self {
            mode,
            signing_algorithm: signing,
        }
    }

    pub fn generate_keypair(&self) -> Result<KeyPair, CryptoError> {
        use ed25519_dalek::SigningKey;
        use rand::RngCore;

        let key_id = uuid::Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| CryptoError::KeyGeneration(e.to_string()))?
            .as_secs();

        // Standard Ed25519 for now
        let mut secret_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        let public_key = base64::engine::general_purpose::STANDARD.encode(verifying_key.as_bytes());
        let private_key = base64::engine::general_purpose::STANDARD.encode(signing_key.as_bytes());

        Ok(KeyPair {
            algorithm: self.signing_algorithm,
            public_key,
            private_key,
            key_id,
            created_at: timestamp,
        })
    }

    pub fn sign(&self, message: &[u8], keypair: &KeyPair) -> Result<Signature, CryptoError> {
        use ed25519_dalek::{Signer, SigningKey};

        let private_bytes = base64::engine::general_purpose::STANDARD
            .decode(&keypair.private_key)
            .map_err(|_| CryptoError::InvalidKeyFormat)?;

        let signing_key = SigningKey::try_from(private_bytes.as_slice())
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

        let classical_sig = signing_key.sign(message);
        let classical_b64 =
            base64::engine::general_purpose::STANDARD.encode(classical_sig.to_bytes());

        let (value, classical_component, pq_component) = match self.mode {
            CryptoMode::Classical => (classical_b64.clone(), Some(classical_b64), None),
            CryptoMode::PostQuantum => {
                let pq_sig = self.generate_pq_signature(message);
                (pq_sig.clone(), None, Some(pq_sig))
            }
            CryptoMode::Hybrid => {
                let pq_sig = self.generate_pq_signature(message);
                let combined = format!("{}:{}", classical_b64, pq_sig);
                (combined, Some(classical_b64), Some(pq_sig))
            }
        };

        Ok(Signature {
            algorithm: self.signing_algorithm,
            value,
            key_id: keypair.key_id.clone(),
            classical_component,
            pq_component,
        })
    }

    /// Generate a post-quantum signature placeholder.
    ///
    /// # Security Warning
    ///
    /// This is a **deterministic hash-based placeholder** — NOT a real post-quantum
    /// digital signature. It does not involve a private key and provides no
    /// cryptographic assurance. It exists solely to preserve the API contract
    /// while real ML-DSA (FIPS 204) support is implemented behind the `pqc` feature.
    ///
    /// Enable the `pqc` feature for real post-quantum signatures.
    fn generate_pq_signature(&self, message: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        tracing::warn!(
            "PQ signature is a placeholder (SHA256 hash). Enable the `pqc` feature for real ML-DSA signatures."
        );
        let mut hasher = Sha256::new();
        hasher.update(b"PQ-PLACEHOLDER-NOT-SECURE-");
        hasher.update(message);
        base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
    }

    /// Verify a signature against the given message and public key.
    ///
    /// Verification is mode-aware:
    /// - **Classical**: Requires and verifies the classical (Ed25519) component.
    /// - **PostQuantum**: Requires the PQ component (placeholder verification for now).
    /// - **Hybrid**: Requires and verifies BOTH classical and PQ components.
    ///
    /// Returns `Err(CryptoError::VerificationFailed)` if any required component
    /// is missing or invalid.
    pub fn verify(
        &self,
        message: &[u8],
        signature: &Signature,
        public_key: &str,
    ) -> Result<bool, CryptoError> {
        use ed25519_dalek::{Verifier, VerifyingKey};

        // Verify classical component when required (Classical or Hybrid mode)
        let classical_required = matches!(self.mode, CryptoMode::Classical | CryptoMode::Hybrid);
        if classical_required {
            let classical_b64 = signature
                .classical_component
                .as_ref()
                .ok_or(CryptoError::VerificationFailed)?;

            let pub_bytes = base64::engine::general_purpose::STANDARD
                .decode(public_key)
                .map_err(|_| CryptoError::InvalidKeyFormat)?;

            let verifying_key = VerifyingKey::try_from(pub_bytes.as_slice())
                .map_err(|_| CryptoError::InvalidKeyFormat)?;

            let sig_bytes = base64::engine::general_purpose::STANDARD
                .decode(classical_b64)
                .map_err(|_| CryptoError::VerificationFailed)?;

            let sig = ed25519_dalek::Signature::try_from(sig_bytes.as_slice())
                .map_err(|_| CryptoError::VerificationFailed)?;

            verifying_key
                .verify(message, &sig)
                .map_err(|_| CryptoError::VerificationFailed)?;
        }

        // Verify PQ component when required (PostQuantum or Hybrid mode)
        let pq_required = matches!(self.mode, CryptoMode::PostQuantum | CryptoMode::Hybrid);
        if pq_required {
            let pq_b64 = signature
                .pq_component
                .as_ref()
                .ok_or(CryptoError::VerificationFailed)?;

            // Placeholder verification: recompute the expected hash and compare.
            // This ensures at minimum that the PQ component matches the message,
            // even though it provides no real cryptographic security.
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(b"PQ-PLACEHOLDER-NOT-SECURE-");
            hasher.update(message);
            let expected = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

            if *pq_b64 != expected {
                return Err(CryptoError::VerificationFailed);
            }
        }

        Ok(true)
    }

    pub fn mode(&self) -> CryptoMode {
        self.mode
    }
}
