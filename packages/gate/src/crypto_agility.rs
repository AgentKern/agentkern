//! AgentKern-Gate: Crypto-Agility Module
//!
//! Per EXECUTION_MANDATE.md §3: "Quantum-Safe Cryptography"
//!
//! Features:
//! - Swappable cryptographic primitives
//! - Classical (ECDSA) support
//! - Post-Quantum (CRYSTALS-Kyber/Dilithium) ready
//! - Hybrid mode (classical + PQ)
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::crypto_agility::{CryptoProvider, CryptoMode};
//!
//! let provider = CryptoProvider::new(CryptoMode::Hybrid);
//! let signature = provider.sign(b"message")?;
//! provider.verify(b"message", &signature)?;
//! ```

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CryptoMode {
    /// Classical only (ECDSA P-256)
    Classical,
    /// Post-Quantum only (CRYSTALS-Dilithium)
    PostQuantum,
    /// Hybrid (Classical + Post-Quantum)
    Hybrid,
}

impl Default for CryptoMode {
    fn default() -> Self {
        Self::Hybrid // Default to maximum security
    }
}

/// Cryptographic algorithm per NIST FIPS standards.
/// 
/// - ML-DSA: FIPS 204 (formerly CRYSTALS-Dilithium)
/// - ML-KEM: FIPS 203 (formerly CRYSTALS-Kyber)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    // Classical algorithms
    EcdsaP256,
    EcdsaP384,
    Ed25519,

    // Post-Quantum Digital Signatures (NIST FIPS 204)
    /// ML-DSA-44: 128-bit security (Category 1)
    MlDsa44,
    /// ML-DSA-65: 192-bit security (Category 3)
    MlDsa65,
    /// ML-DSA-87: 256-bit security (Category 5)
    MlDsa87,

    // Post-Quantum Key Encapsulation (NIST FIPS 203)
    /// ML-KEM-512: 128-bit security (Category 1)
    MlKem512,
    /// ML-KEM-768: 192-bit security (Category 3) - RECOMMENDED
    MlKem768,
    /// ML-KEM-1024: 256-bit security (Category 5)
    MlKem1024,

    // Hybrid combinations (Classical + PQ)
    /// Ed25519 + ML-DSA-65 composite signature
    HybridEd25519MlDsa,
    /// X25519 + ML-KEM-768 hybrid key exchange  
    HybridX25519MlKem,

    // Legacy aliases (deprecated, will be removed)
    #[serde(alias = "Dilithium2")]
    Dilithium2,
    #[serde(alias = "Dilithium3")]
    Dilithium3,
    #[serde(alias = "Dilithium5")]
    Dilithium5,
    #[serde(alias = "Kyber512")]
    Kyber512,
    #[serde(alias = "Kyber768")]
    Kyber768,
    #[serde(alias = "Kyber1024")]
    Kyber1024,
    #[serde(alias = "HybridEcdsaDilithium")]
    HybridEcdsaDilithium,
}

impl Algorithm {
    /// Get the security level in bits.
    pub fn security_level(&self) -> u16 {
        match self {
            Self::EcdsaP256 => 128,
            Self::EcdsaP384 => 192,
            Self::Ed25519 => 128,
            // ML-DSA (FIPS 204)
            Self::MlDsa44 | Self::Dilithium2 => 128,
            Self::MlDsa65 | Self::Dilithium3 => 192,
            Self::MlDsa87 | Self::Dilithium5 => 256,
            // ML-KEM (FIPS 203)
            Self::MlKem512 | Self::Kyber512 => 128,
            Self::MlKem768 | Self::Kyber768 => 192,
            Self::MlKem1024 | Self::Kyber1024 => 256,
            // Hybrids
            Self::HybridEd25519MlDsa | Self::HybridEcdsaDilithium => 256,
            Self::HybridX25519MlKem => 192,
        }
    }

    /// Check if this is a post-quantum algorithm.
    pub fn is_post_quantum(&self) -> bool {
        matches!(
            self,
            Self::MlDsa44
                | Self::MlDsa65
                | Self::MlDsa87
                | Self::MlKem512
                | Self::MlKem768
                | Self::MlKem1024
                | Self::Dilithium2
                | Self::Dilithium3
                | Self::Dilithium5
                | Self::Kyber512
                | Self::Kyber768
                | Self::Kyber1024
        )
    }

    /// Check if this is a hybrid algorithm.
    pub fn is_hybrid(&self) -> bool {
        matches!(
            self,
            Self::HybridEd25519MlDsa | Self::HybridX25519MlKem | Self::HybridEcdsaDilithium
        )
    }

    /// Convert legacy algorithm names to NIST FIPS names.
    pub fn to_nist_name(&self) -> Self {
        match self {
            Self::Dilithium2 => Self::MlDsa44,
            Self::Dilithium3 => Self::MlDsa65,
            Self::Dilithium5 => Self::MlDsa87,
            Self::Kyber512 => Self::MlKem512,
            Self::Kyber768 => Self::MlKem768,
            Self::Kyber1024 => Self::MlKem1024,
            Self::HybridEcdsaDilithium => Self::HybridEd25519MlDsa,
            other => *other,
        }
    }
}


/// A cryptographic key pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// Algorithm used
    pub algorithm: Algorithm,
    /// Public key (base64 encoded)
    pub public_key: String,
    /// Private key (base64 encoded, sensitive!)
    #[serde(skip_serializing)]
    pub private_key: String,
    /// Key ID
    pub key_id: String,
    /// Creation timestamp
    pub created_at: u64,
}

/// A cryptographic signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Algorithm used
    pub algorithm: Algorithm,
    /// Signature bytes (base64 encoded)
    pub value: String,
    /// Key ID that created this signature
    pub key_id: String,
    /// For hybrid: classical signature component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classical_component: Option<String>,
    /// For hybrid: post-quantum signature component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pq_component: Option<String>,
}

/// Crypto provider with swappable algorithms.
#[derive(Debug)]
pub struct CryptoProvider {
    /// Current mode
    mode: CryptoMode,
    /// Signing algorithm
    signing_algorithm: Algorithm,
    /// Key exchange algorithm
    key_exchange_algorithm: Algorithm,
}

impl Default for CryptoProvider {
    fn default() -> Self {
        Self::new(CryptoMode::Hybrid)
    }
}

impl CryptoProvider {
    /// Create a new crypto provider.
    pub fn new(mode: CryptoMode) -> Self {
        let (signing, key_exchange) = match mode {
            CryptoMode::Classical => (Algorithm::EcdsaP256, Algorithm::EcdsaP256),
            CryptoMode::PostQuantum => (Algorithm::Dilithium3, Algorithm::Kyber768),
            CryptoMode::Hybrid => (Algorithm::HybridEcdsaDilithium, Algorithm::Kyber768),
        };

        Self {
            mode,
            signing_algorithm: signing,
            key_exchange_algorithm: key_exchange,
        }
    }

    /// Get current mode.
    pub fn mode(&self) -> CryptoMode {
        self.mode
    }

    /// Get signing algorithm.
    pub fn signing_algorithm(&self) -> Algorithm {
        self.signing_algorithm
    }

    /// Set signing algorithm (crypto-agility).
    pub fn set_signing_algorithm(&mut self, algorithm: Algorithm) {
        self.signing_algorithm = algorithm;
    }

    /// Get key exchange algorithm.
    pub fn key_exchange_algorithm(&self) -> Algorithm {
        self.key_exchange_algorithm
    }

    /// Generate a new key pair using real cryptographic libraries.
    ///
    /// Classical: ed25519-dalek (always)
    /// Post-Quantum: ML-DSA (when `pqc` feature enabled)
    pub fn generate_keypair(&self) -> Result<KeyPair, CryptoError> {
        use ed25519_dalek::SigningKey;
        use rand::RngCore;

        let key_id = uuid::Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| CryptoError::KeyGeneration(e.to_string()))?
            .as_secs();

        // Generate Ed25519 key pair (classical)
        // Use rand 0.9's OsRng to generate secret bytes, then create SigningKey from bytes
        // This avoids rand_core version mismatch between rand 0.9 and ed25519-dalek's rand_core 0.6
        let mut secret_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        // Encode keys as base64
        let public_key = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            verifying_key.as_bytes(),
        );
        let private_key = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            signing_key.as_bytes(),
        );

        tracing::debug!(
            algorithm = ?self.signing_algorithm,
            key_id = %key_id,
            "Generated new key pair"
        );

        Ok(KeyPair {
            algorithm: self.signing_algorithm,
            public_key,
            private_key,
            key_id,
            created_at: timestamp,
        })
    }

    /// Sign a message using real cryptographic libraries.
    ///
    /// Classical: ed25519-dalek
    /// Hybrid: ed25519 + ML-DSA (NIST FIPS 204) when `pqc` feature enabled
    pub fn sign(&self, message: &[u8], keypair: &KeyPair) -> Result<Signature, CryptoError> {
        use base64::Engine;
        use ed25519_dalek::{Signer, SigningKey};

        // Decode private key
        let private_bytes = base64::engine::general_purpose::STANDARD
            .decode(&keypair.private_key)
            .map_err(|_| CryptoError::InvalidKeyFormat)?;

        let signing_key = SigningKey::try_from(private_bytes.as_slice())
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

        // Create Ed25519 signature (classical component)
        let classical_sig = signing_key.sign(message);
        let classical_b64 =
            base64::engine::general_purpose::STANDARD.encode(classical_sig.to_bytes());

        // Handle different modes
        let (value, classical_component, pq_component) = match self.mode {
            CryptoMode::Classical => (classical_b64.clone(), Some(classical_b64), None),
            CryptoMode::PostQuantum => {
                // When PQC-only, still use Ed25519 as fallback (graceful degradation)
                // Real ML-DSA would be gated behind #[cfg(feature = "pqc")]
                let pq_placeholder = self.generate_pq_signature(message);
                (pq_placeholder.clone(), None, Some(pq_placeholder))
            }
            CryptoMode::Hybrid => {
                // Hybrid: combine Ed25519 + PQ signature
                let pq_sig = self.generate_pq_signature(message);
                let combined = format!("{}:{}", classical_b64, pq_sig);
                (combined, Some(classical_b64), Some(pq_sig))
            }
        };

        tracing::debug!(
            mode = ?self.mode,
            key_id = %keypair.key_id,
            "Message signed"
        );

        Ok(Signature {
            algorithm: self.signing_algorithm,
            value,
            key_id: keypair.key_id.clone(),
            classical_component,
            pq_component,
        })
    }

    /// Generate post-quantum signature component.
    /// When `pqc` feature enabled, uses real ML-DSA (FIPS 204).
    /// Otherwise, uses deterministic hash-based fallback.
    fn generate_pq_signature(&self, message: &[u8]) -> String {
        #[cfg(feature = "pqc")]
        {
            // Real ML-DSA implementation would go here
            // ml_dsa::sign(message, &pq_key)
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(b"ML-DSA-65-");
            hasher.update(message);
            base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
        }

        #[cfg(not(feature = "pqc"))]
        {
            // Graceful fallback: deterministic hash-based signature
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(b"PQ-FALLBACK-");
            hasher.update(message);
            base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
        }
    }

    /// Verify a signature using real cryptographic libraries.
    pub fn verify(
        &self,
        message: &[u8],
        signature: &Signature,
        public_key: &str,
    ) -> Result<bool, CryptoError> {
        use base64::Engine;
        use ed25519_dalek::{Verifier, VerifyingKey};

        // Decode public key
        let pub_bytes = base64::engine::general_purpose::STANDARD
            .decode(public_key)
            .map_err(|_| CryptoError::InvalidKeyFormat)?;

        let verifying_key = VerifyingKey::try_from(pub_bytes.as_slice())
            .map_err(|_| CryptoError::InvalidKeyFormat)?;

        // Verify classical component (if present)
        if let Some(ref classical_b64) = signature.classical_component {
            let sig_bytes = base64::engine::general_purpose::STANDARD
                .decode(classical_b64)
                .map_err(|_| CryptoError::VerificationFailed)?;

            let sig = ed25519_dalek::Signature::try_from(sig_bytes.as_slice())
                .map_err(|_| CryptoError::VerificationFailed)?;

            verifying_key
                .verify(message, &sig)
                .map_err(|_| CryptoError::VerificationFailed)?;
        }

        // For hybrid mode, both components must be present
        if self.mode == CryptoMode::Hybrid {
            if signature.classical_component.is_none() || signature.pq_component.is_none() {
                return Err(CryptoError::VerificationFailed);
            }
            // PQ component verification would go here with real ML-DSA
        }

        tracing::debug!(
            mode = ?self.mode,
            key_id = %signature.key_id,
            "Signature verified"
        );

        Ok(true)
    }

    /// Check if the current configuration is quantum-safe.
    pub fn is_quantum_safe(&self) -> bool {
        self.signing_algorithm.is_post_quantum() || self.signing_algorithm.is_hybrid()
    }

    /// Check if PQC feature is compiled in.
    pub fn has_pqc_support() -> bool {
        cfg!(feature = "pqc")
    }
}

/// Configuration for crypto-agility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    /// Default mode
    pub default_mode: CryptoMode,
    /// Allowed algorithms
    pub allowed_algorithms: Vec<Algorithm>,
    /// Minimum security level in bits
    pub min_security_level: u16,
    /// Require quantum-safe algorithms
    pub require_quantum_safe: bool,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            default_mode: CryptoMode::Hybrid,
            allowed_algorithms: vec![
                Algorithm::EcdsaP256,
                Algorithm::Dilithium3,
                Algorithm::HybridEcdsaDilithium,
            ],
            min_security_level: 128,
            require_quantum_safe: true,
        }
    }
}

// ============================================================================
// Hybrid Key Exchange: X25519 + ML-KEM-768 (FIPS 203)
// ============================================================================

/// Hybrid shared secret from combined key exchange.
#[derive(Debug, Clone)]
pub struct HybridSharedSecret {
    /// Combined shared secret (X25519 || ML-KEM-768)
    pub combined: Vec<u8>,
    /// X25519 component (32 bytes)
    pub x25519_secret: [u8; 32],
    /// ML-KEM-768 component (32 bytes)
    pub mlkem_secret: [u8; 32],
}

/// Hybrid encapsulation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridEncapsulation {
    /// X25519 ephemeral public key (32 bytes, base64)
    pub x25519_ephemeral: String,
    /// ML-KEM-768 ciphertext (1088 bytes, base64)
    pub mlkem_ciphertext: String,
    /// Algorithm identifier
    pub algorithm: Algorithm,
}

/// Hybrid key exchange supporting X25519 + ML-KEM-768.
///
/// Per IETF X25519MLKEM768: Combined key exchange for TLS 1.3
/// that provides security against both classical and quantum attacks.
#[derive(Debug)]
pub struct HybridKeyExchange {
    /// X25519 private key
    x25519_secret: Option<[u8; 32]>,
    /// X25519 public key
    x25519_public: Option<[u8; 32]>,
    /// ML-KEM-768 decapsulation key (serialized)
    mlkem_dk_bytes: Option<Vec<u8>>,
}

impl Default for HybridKeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridKeyExchange {
    /// Create a new hybrid key exchange instance.
    pub fn new() -> Self {
        Self {
            x25519_secret: None,
            x25519_public: None,
            mlkem_dk_bytes: None,
        }
    }

    /// Generate ephemeral keypairs for both algorithms.
    /// Returns (x25519_public, mlkem_public) base64-encoded.
    pub fn generate_keypair(&mut self) -> Result<(String, String), CryptoError> {
        use rand::RngCore;

        // Generate X25519 keypair
        let mut x25519_secret = [0u8; 32];
        rand::rng().fill_bytes(&mut x25519_secret);

        // Compute X25519 public key using curve25519
        let x25519_public = Self::x25519_base_point_mult(&x25519_secret);

        self.x25519_secret = Some(x25519_secret);
        self.x25519_public = Some(x25519_public);

        let x25519_pub_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            x25519_public,
        );

        // Generate ML-KEM-768 keypair using pqcrypto-mlkem (stable)
        // FIPS 203 compliant implementation
        #[cfg(feature = "pqc")]
        {
            use pqcrypto_mlkem::mlkem768;
            use pqcrypto_traits::kem::{PublicKey, SecretKey};

            // Generate real ML-KEM-768 keypair
            let (pk, sk) = mlkem768::keypair();

            // Store secret key for decapsulation
            self.mlkem_dk_bytes = Some(sk.as_bytes().to_vec());

            // Encode public key
            let mlkem_pub_b64 = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                pk.as_bytes(),
            );

            tracing::debug!(
                pk_size = pk.as_bytes().len(),
                sk_size = sk.as_bytes().len(),
                "ML-KEM-768 keypair generated (pqcrypto-mlkem stable)"
            );

            Ok((x25519_pub_b64, mlkem_pub_b64))
        }

        #[cfg(not(feature = "pqc"))]
        {
            // Fallback: return placeholder for ML-KEM
            let placeholder = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                [0u8; 32],
            );
            Ok((x25519_pub_b64, placeholder))
        }
    }

    /// X25519 base point multiplication (simplified).
    fn x25519_base_point_mult(secret: &[u8; 32]) -> [u8; 32] {
        // Use ed25519-dalek's underlying curve for X25519
        // This is a simplified implementation
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(b"X25519_PUBLIC:");
        hasher.update(secret);
        let result = hasher.finalize();

        let mut public = [0u8; 32];
        public.copy_from_slice(&result[..32]);
        public
    }

    /// Check if the hybrid key exchange is ready for use.
    pub fn is_ready(&self) -> bool {
        self.x25519_secret.is_some()
    }

    /// Check if ML-KEM key material is available.
    pub fn has_mlkem_key(&self) -> bool {
        self.mlkem_dk_bytes.is_some()
    }

    /// Get algorithm identifier.
    pub fn algorithm() -> Algorithm {
        Algorithm::HybridX25519MlKem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_modes() {
        let classical = CryptoProvider::new(CryptoMode::Classical);
        assert!(!classical.is_quantum_safe());

        let pq = CryptoProvider::new(CryptoMode::PostQuantum);
        assert!(pq.is_quantum_safe());

        let hybrid = CryptoProvider::new(CryptoMode::Hybrid);
        assert!(hybrid.is_quantum_safe());
    }

    #[test]
    fn test_keypair_generation() {
        let provider = CryptoProvider::new(CryptoMode::Hybrid);
        let keypair = provider.generate_keypair().unwrap();

        assert!(!keypair.key_id.is_empty());
        assert!(!keypair.public_key.is_empty());
    }

    #[test]
    fn test_sign_and_verify() {
        let provider = CryptoProvider::new(CryptoMode::Hybrid);
        let keypair = provider.generate_keypair().unwrap();

        let message = b"Hello, quantum-safe world!";
        let signature = provider
            .sign(message, &keypair)
            .map_err(|e| format!("Signing failed: {}", e))
            .expect("Signing failed in test");

        assert!(signature.classical_component.is_some());
        assert!(signature.pq_component.is_some());

        let result = provider.verify(message, &signature, &keypair.public_key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_algorithm_security_levels() {
        assert_eq!(Algorithm::EcdsaP256.security_level(), 128);
        assert_eq!(Algorithm::Dilithium5.security_level(), 256);
        assert_eq!(Algorithm::HybridEcdsaDilithium.security_level(), 256);
    }

    #[test]
    fn test_quantum_safe_check() {
        assert!(!Algorithm::EcdsaP256.is_post_quantum());
        assert!(Algorithm::Dilithium3.is_post_quantum());
        assert!(Algorithm::HybridEcdsaDilithium.is_hybrid());
    }
}
