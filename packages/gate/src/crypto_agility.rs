//! VeriMantle-Gate: Crypto-Agility Module
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
//! use verimantle_gate::crypto_agility::{CryptoProvider, CryptoMode};
//!
//! let provider = CryptoProvider::new(CryptoMode::Hybrid);
//! let signature = provider.sign(b"message")?;
//! provider.verify(b"message", &signature)?;
//! ```

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

/// Cryptographic algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Algorithm {
    // Classical algorithms
    EcdsaP256,
    EcdsaP384,
    Ed25519,
    
    // Post-Quantum algorithms (NIST PQC)
    Dilithium2,
    Dilithium3,
    Dilithium5,
    Kyber512,
    Kyber768,
    Kyber1024,
    
    // Hybrid combinations
    HybridEcdsaDilithium,
}

impl Algorithm {
    /// Get the security level in bits.
    pub fn security_level(&self) -> u16 {
        match self {
            Self::EcdsaP256 => 128,
            Self::EcdsaP384 => 192,
            Self::Ed25519 => 128,
            Self::Dilithium2 => 128,
            Self::Dilithium3 => 192,
            Self::Dilithium5 => 256,
            Self::Kyber512 => 128,
            Self::Kyber768 => 192,
            Self::Kyber1024 => 256,
            Self::HybridEcdsaDilithium => 256, // Max of both
        }
    }

    /// Check if this is a post-quantum algorithm.
    pub fn is_post_quantum(&self) -> bool {
        matches!(
            self,
            Self::Dilithium2 | Self::Dilithium3 | Self::Dilithium5 |
            Self::Kyber512 | Self::Kyber768 | Self::Kyber1024
        )
    }

    /// Check if this is a hybrid algorithm.
    pub fn is_hybrid(&self) -> bool {
        matches!(self, Self::HybridEcdsaDilithium)
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

    /// Generate a new key pair.
    pub fn generate_keypair(&self) -> Result<KeyPair, CryptoError> {
        // In production, this would use actual cryptographic libraries
        // For now, we generate placeholder keys
        
        let key_id = uuid::Uuid::new_v4().to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Ok(KeyPair {
            algorithm: self.signing_algorithm,
            public_key: format!("PUB-{}-{}", self.signing_algorithm.security_level(), key_id),
            private_key: format!("PRIV-{}", key_id),
            key_id,
            created_at: timestamp,
        })
    }

    /// Sign a message.
    /// 
    /// Note: This is a placeholder implementation.
    /// Production would use actual cryptographic signing.
    pub fn sign(&self, message: &[u8], keypair: &KeyPair) -> Result<Signature, CryptoError> {
        // Validate algorithm matches
        if keypair.algorithm != self.signing_algorithm {
            return Err(CryptoError::UnsupportedAlgorithm(
                format!("Key uses {:?}, provider uses {:?}", 
                    keypair.algorithm, self.signing_algorithm)
            ));
        }

        // Create signature based on mode
        let (classical, pq) = match self.mode {
            CryptoMode::Classical => (
                Some(format!("CLASSICAL-SIG-{:x}", md5_hash(message))),
                None
            ),
            CryptoMode::PostQuantum => (
                None,
                Some(format!("PQ-SIG-{:x}", md5_hash(message)))
            ),
            CryptoMode::Hybrid => (
                Some(format!("CLASSICAL-SIG-{:x}", md5_hash(message))),
                Some(format!("PQ-SIG-{:x}", md5_hash(message)))
            ),
        };

        Ok(Signature {
            algorithm: self.signing_algorithm,
            value: format!("SIG-{}-{:x}", keypair.key_id, md5_hash(message)),
            key_id: keypair.key_id.clone(),
            classical_component: classical,
            pq_component: pq,
        })
    }

    /// Verify a signature.
    /// 
    /// Note: This is a placeholder implementation.
    /// Production would use actual cryptographic verification.
    pub fn verify(&self, message: &[u8], signature: &Signature, public_key: &str) -> Result<bool, CryptoError> {
        // In production, this would:
        // 1. Parse the public key
        // 2. Verify classical component (if present)
        // 3. Verify post-quantum component (if present)
        // 4. Both must pass for hybrid mode
        
        // Placeholder: check that signature contains expected hash
        let expected_hash = format!("{:x}", md5_hash(message));
        let is_valid = signature.value.contains(&expected_hash);
        
        if !is_valid {
            return Err(CryptoError::VerificationFailed);
        }
        
        // For hybrid, both components must be present
        if self.mode == CryptoMode::Hybrid {
            if signature.classical_component.is_none() || signature.pq_component.is_none() {
                return Err(CryptoError::VerificationFailed);
            }
        }
        
        Ok(true)
    }

    /// Check if the current configuration is quantum-safe.
    pub fn is_quantum_safe(&self) -> bool {
        self.signing_algorithm.is_post_quantum() || 
        self.signing_algorithm.is_hybrid()
    }
}

/// Simple hash for placeholder signatures (NOT cryptographic!).
fn md5_hash(data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
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
        let signature = provider.sign(message, &keypair).unwrap();
        
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
