//! SDK Error Types
//!
//! Comprehensive error handling for all SDK operations.

use thiserror::Error;

/// Result type alias for SDK operations.
pub type SdkResult<T> = Result<T, SdkError>;

/// SDK error types.
#[derive(Debug, Error)]
pub enum SdkError {
    // =========================================================================
    // Key Management Errors
    // =========================================================================
    
    /// Failed to generate keypair
    #[error("Failed to generate keypair: {0}")]
    KeyGenerationFailed(String),
    
    /// Invalid private key format
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    
    /// Invalid public key format
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    
    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    // =========================================================================
    // Signing Errors
    // =========================================================================
    
    /// Signing operation failed
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    
    /// Signature verification failed
    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),
    
    /// Invalid signature format
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    // =========================================================================
    // Proof Errors
    // =========================================================================
    
    /// Proof creation failed
    #[error("Proof creation failed: {0}")]
    ProofCreationFailed(String),
    
    /// Proof validation failed
    #[error("Proof validation failed: {0}")]
    ProofValidationFailed(String),
    
    /// Proof expired
    #[error("Proof expired at {0}")]
    ProofExpired(String),
    
    /// Invalid proof format
    #[error("Invalid proof format: {0}")]
    InvalidProofFormat(String),
    
    /// Missing required claim
    #[error("Missing required claim: {0}")]
    MissingClaim(String),

    // =========================================================================
    // Protocol Errors
    // =========================================================================
    
    /// Message encoding failed
    #[error("Message encoding failed: {0}")]
    EncodingFailed(String),
    
    /// Message decoding failed
    #[error("Message decoding failed: {0}")]
    DecodingFailed(String),
    
    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    // =========================================================================
    // Configuration Errors
    // =========================================================================
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    // =========================================================================
    // External Errors
    // =========================================================================
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Base64 encoding error
    #[error("Base64 error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

impl SdkError {
    /// Create a key generation error
    pub fn key_generation(msg: impl Into<String>) -> Self {
        Self::KeyGenerationFailed(msg.into())
    }

    /// Create a signing error
    pub fn signing(msg: impl Into<String>) -> Self {
        Self::SigningFailed(msg.into())
    }

    /// Create a verification error
    pub fn verification(msg: impl Into<String>) -> Self {
        Self::VerificationFailed(msg.into())
    }

    /// Create a proof error
    pub fn proof(msg: impl Into<String>) -> Self {
        Self::ProofCreationFailed(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SdkError::KeyGenerationFailed("RNG failed".into());
        assert!(err.to_string().contains("RNG failed"));
    }

    #[test]
    fn test_error_helpers() {
        let err = SdkError::key_generation("test error");
        assert!(matches!(err, SdkError::KeyGenerationFailed(_)));
    }
}
