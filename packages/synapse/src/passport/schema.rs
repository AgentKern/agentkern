//! Memory Passport Schema - Core types and validation
//!
//! Defines the universal format for portable agent state.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

/// Passport version for forward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassportVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl PassportVersion {
    pub const CURRENT: Self = Self { major: 1, minor: 0, patch: 0 };
    
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major
    }
}

impl Default for PassportVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl std::fmt::Display for PassportVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Passport errors.
#[derive(Debug, Error)]
pub enum PassportError {
    #[error("Invalid passport version: {0}")]
    IncompatibleVersion(String),
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
}

/// Agent identity using W3C DID format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// DID URI (e.g., "did:agentkern:agent-123")
    pub did: String,
    /// Public key for verification (base64)
    pub public_key: String,
    /// Key algorithm
    pub algorithm: String,
    /// Creation timestamp (Unix ms)
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
}

/// Sovereignty information for data residency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyInfo {
    /// Original jurisdiction (ISO 3166-1 alpha-2)
    pub origin_region: String,
    /// Current jurisdiction
    pub current_region: String,
    /// Allowed regions for data transfer
    pub allowed_regions: Vec<String>,
    /// Transfer history
    pub transfers: Vec<TransferRecord>,
    /// Data residency constraints
    pub residency_rules: Vec<ResidencyRule>,
}

/// Record of data transfer between regions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRecord {
    /// Source region
    pub from: String,
    /// Destination region
    pub to: String,
    /// Timestamp
    pub timestamp: u64,
    /// Reason for transfer
    pub reason: String,
    /// Was transfer approved?
    pub approved: bool,
}

/// Data residency rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidencyRule {
    /// Rule ID
    pub id: String,
    /// Data category this applies to
    pub category: String,
    /// Allowed regions
    pub regions: Vec<String>,
    /// Minimum retention period (days)
    pub retention_days: Option<u32>,
    /// Regulation source (e.g., "GDPR", "CCPA")
    pub regulation: String,
}

/// Cryptographic provenance chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceChain {
    /// Chain of signatures
    pub signatures: Vec<ProvenanceSignature>,
    /// Hash algorithm used
    pub hash_algorithm: String,
}

/// Single provenance signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceSignature {
    /// Signer DID
    pub signer: String,
    /// Signature value (base64)
    pub signature: String,
    /// Timestamp
    pub timestamp: u64,
    /// Hash of previous state
    pub prev_hash: String,
}

impl ProvenanceChain {
    /// Create empty chain.
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
            hash_algorithm: "SHA-256".to_string(),
        }
    }
    
    /// Verify chain integrity.
    pub fn verify(&self) -> Result<bool, PassportError> {
        // In production, verify each signature cryptographically
        Ok(!self.signatures.is_empty())
    }
}

impl Default for ProvenanceChain {
    fn default() -> Self {
        Self::new()
    }
}

/// The Universal Memory Passport - portable agent state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPassport {
    /// Passport version
    pub version: PassportVersion,
    
    /// Agent identity (DID-based)
    pub identity: AgentIdentity,
    
    /// Cryptographic provenance
    pub provenance: ProvenanceChain,
    
    /// Sovereignty metadata
    pub sovereignty: SovereigntyInfo,
    
    /// Memory layers (may be encrypted)
    pub memory: super::layers::MemoryLayers,
    
    /// Export timestamp
    pub exported_at: u64,
    
    /// Checksum of memory content
    pub checksum: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MemoryPassport {
    /// Create a new passport.
    pub fn new(identity: AgentIdentity, origin_region: impl Into<String>) -> Self {
        let region = origin_region.into();
        Self {
            version: PassportVersion::CURRENT,
            identity,
            provenance: ProvenanceChain::default(),
            sovereignty: SovereigntyInfo {
                origin_region: region.clone(),
                current_region: region,
                allowed_regions: vec![],
                transfers: vec![],
                residency_rules: vec![],
            },
            memory: super::layers::MemoryLayers::default(),
            exported_at: chrono::Utc::now().timestamp_millis() as u64,
            checksum: String::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Validate passport integrity.
    pub fn validate(&self) -> Result<(), PassportError> {
        // Check version compatibility
        if !self.version.is_compatible(&PassportVersion::CURRENT) {
            return Err(PassportError::IncompatibleVersion(
                format!("Got {}, expected {}.x.x", self.version, PassportVersion::CURRENT.major)
            ));
        }
        
        // Verify identity
        if self.identity.did.is_empty() {
            return Err(PassportError::MissingField("identity.did".into()));
        }
        
        // Verify provenance - must have at least one signature
        if !self.provenance.verify()? {
            return Err(PassportError::InvalidSignature);
        }
        
        Ok(())
    }
    
    /// Check if passport can be transferred to region.
    pub fn can_transfer_to(&self, region: &str) -> bool {
        if self.sovereignty.allowed_regions.is_empty() {
            // No restrictions
            return true;
        }
        self.sovereignty.allowed_regions.contains(&region.to_string())
    }
    
    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, PassportError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PassportError::SerializationError(e.to_string()))
    }
    
    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, PassportError> {
        serde_json::from_str(json)
            .map_err(|e| PassportError::SerializationError(e.to_string()))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_identity() -> AgentIdentity {
        AgentIdentity {
            did: "did:agentkern:agent-test-001".to_string(),
            public_key: "base64pubkey".to_string(),
            algorithm: "Ed25519".to_string(),
            created_at: 1700000000000,
            updated_at: 1700000000000,
        }
    }

    #[test]
    fn test_passport_version() {
        let v1 = PassportVersion { major: 1, minor: 0, patch: 0 };
        let v1_1 = PassportVersion { major: 1, minor: 1, patch: 0 };
        let v2 = PassportVersion { major: 2, minor: 0, patch: 0 };
        
        assert!(v1.is_compatible(&v1_1));
        assert!(!v1.is_compatible(&v2));
    }

    #[test]
    fn test_create_passport() {
        let passport = MemoryPassport::new(sample_identity(), "US");
        
        assert_eq!(passport.version, PassportVersion::CURRENT);
        assert_eq!(passport.sovereignty.origin_region, "US");
        assert_eq!(passport.identity.did, "did:agentkern:agent-test-001");
    }

    #[test]
    fn test_passport_validation() {
        let passport = MemoryPassport::new(sample_identity(), "US");
        
        // Should fail because provenance chain is empty
        // In real implementation, we'd sign on creation
        assert!(passport.validate().is_err());
    }

    #[test]
    fn test_transfer_restriction() {
        let mut passport = MemoryPassport::new(sample_identity(), "US");
        passport.sovereignty.allowed_regions = vec!["US".into(), "EU".into()];
        
        assert!(passport.can_transfer_to("US"));
        assert!(passport.can_transfer_to("EU"));
        assert!(!passport.can_transfer_to("CN"));
    }

    #[test]
    fn test_passport_serialization() {
        let passport = MemoryPassport::new(sample_identity(), "US");
        
        let json = passport.to_json().unwrap();
        let restored = MemoryPassport::from_json(&json).unwrap();
        
        assert_eq!(restored.identity.did, passport.identity.did);
        assert_eq!(restored.sovereignty.origin_region, "US");
    }

    #[test]
    fn test_provenance_chain() {
        let mut chain = ProvenanceChain::new();
        chain.signatures.push(ProvenanceSignature {
            signer: "did:agentkern:signer-1".into(),
            signature: "base64sig".into(),
            timestamp: 1700000000000,
            prev_hash: "0000000000".into(),
        });
        
        assert!(chain.verify().unwrap());
    }
}
