//! Cryptographic Signing Module
//!
//! Ed25519 key generation, signing, and verification using the `ring` crate.
//! ring is based on AWS libcrypto (BoringSSL) - production-grade and FIPS-ready.

use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, KeyPair as RingKeyPair, UnparsedPublicKey, ED25519},
};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::error::{SdkError, SdkResult};

/// Ed25519 key pair for signing operations.
pub struct KeyPair {
    inner: Ed25519KeyPair,
    seed: Vec<u8>,
}

impl KeyPair {
    /// Generate a new random Ed25519 keypair.
    pub fn generate() -> SdkResult<Self> {
        let rng = SystemRandom::new();
        let seed = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| SdkError::key_generation(format!("PKCS8 generation failed: {e}")))?;
        
        let inner = Ed25519KeyPair::from_pkcs8(seed.as_ref())
            .map_err(|e| SdkError::key_generation(format!("KeyPair creation failed: {e}")))?;
        
        Ok(Self {
            inner,
            seed: seed.as_ref().to_vec(),
        })
    }

    /// Create keypair from existing seed (PKCS8 format).
    pub fn from_seed(seed: &[u8]) -> SdkResult<Self> {
        let inner = Ed25519KeyPair::from_pkcs8(seed)
            .map_err(|e| SdkError::InvalidPrivateKey(format!("Invalid PKCS8: {e}")))?;
        
        Ok(Self {
            inner,
            seed: seed.to_vec(),
        })
    }

    /// Create keypair from base64url-encoded seed.
    pub fn from_base64(encoded: &str) -> SdkResult<Self> {
        let seed = URL_SAFE_NO_PAD.decode(encoded)?;
        Self::from_seed(&seed)
    }

    /// Get the public key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            bytes: self.inner.public_key().as_ref().to_vec(),
        }
    }

    /// Get the seed for serialization (PKCS8 format).
    pub fn seed(&self) -> &[u8] {
        &self.seed
    }

    /// Get the seed as base64url-encoded string.
    pub fn seed_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode(&self.seed)
    }

    /// Sign data and return a signature.
    pub fn sign(&self, data: &[u8]) -> Signature {
        let sig = self.inner.sign(data);
        Signature {
            bytes: sig.as_ref().to_vec(),
        }
    }
}

/// Ed25519 public key (32 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    bytes: Vec<u8>,
}

impl PublicKey {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> SdkResult<Self> {
        if bytes.len() != 32 {
            return Err(SdkError::InvalidPublicKey(format!(
                "Expected 32 bytes, got {}",
                bytes.len()
            )));
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Create from base64url-encoded string.
    pub fn from_base64(encoded: &str) -> SdkResult<Self> {
        let bytes = URL_SAFE_NO_PAD.decode(encoded)?;
        Self::from_bytes(&bytes)
    }

    /// Get raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Encode as base64url string.
    pub fn to_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode(&self.bytes)
    }

    /// Verify a signature against this public key.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> SdkResult<bool> {
        let key = UnparsedPublicKey::new(&ED25519, &self.bytes);
        match key.verify(message, &signature.bytes) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false), // Invalid signature is not an error, just false
        }
    }
}

/// Ed25519 signature (64 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    bytes: Vec<u8>,
}

impl Signature {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> SdkResult<Self> {
        if bytes.len() != 64 {
            return Err(SdkError::InvalidSignature(format!(
                "Expected 64 bytes, got {}",
                bytes.len()
            )));
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Create from base64url-encoded string.
    pub fn from_base64(encoded: &str) -> SdkResult<Self> {
        let bytes = URL_SAFE_NO_PAD.decode(encoded)?;
        Self::from_bytes(&bytes)
    }

    /// Get raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Encode as base64url string.
    pub fn to_base64(&self) -> String {
        URL_SAFE_NO_PAD.encode(&self.bytes)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate().expect("KeyPair generation failed");
        assert_eq!(kp.public_key().as_bytes().len(), 32);
    }

    #[test]
    fn test_keypair_from_seed() {
        let kp1 = KeyPair::generate().unwrap();
        let seed = kp1.seed().to_vec();
        
        let kp2 = KeyPair::from_seed(&seed).unwrap();
        assert_eq!(kp1.public_key(), kp2.public_key());
    }

    #[test]
    fn test_sign_verify() {
        let kp = KeyPair::generate().unwrap();
        let message = b"Hello, AgentKern!";
        
        let signature = kp.sign(message);
        assert_eq!(signature.as_bytes().len(), 64);
        
        let is_valid = kp.public_key().verify(message, &signature).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_wrong_message() {
        let kp = KeyPair::generate().unwrap();
        let signature = kp.sign(b"original message");
        
        let is_valid = kp.public_key().verify(b"different message", &signature).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_verify_wrong_key() {
        let kp1 = KeyPair::generate().unwrap();
        let kp2 = KeyPair::generate().unwrap();
        
        let signature = kp1.sign(b"message");
        let is_valid = kp2.public_key().verify(b"message", &signature).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_base64_roundtrip() {
        let kp = KeyPair::generate().unwrap();
        let pk_b64 = kp.public_key().to_base64();
        
        let pk2 = PublicKey::from_base64(&pk_b64).unwrap();
        assert_eq!(kp.public_key(), pk2);
    }

    #[test]
    fn test_signature_base64() {
        let kp = KeyPair::generate().unwrap();
        let sig = kp.sign(b"test");
        let sig_b64 = sig.to_base64();
        
        let sig2 = Signature::from_base64(&sig_b64).unwrap();
        assert_eq!(sig, sig2);
    }
}
