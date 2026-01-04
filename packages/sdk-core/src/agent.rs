//! Agent Module
//!
//! Core Agent identity with keypair management and Liability Proof creation.
//! An Agent is the fundamental unit of identity in AgentKern.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{SdkError, SdkResult};
use crate::proof::{LiabilityProof, ProofClaims, ProofHeader};
use crate::signing::{KeyPair, PublicKey};

/// Unique agent identifier (DID format).
pub type AgentId = String;

/// Agent configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent name/label
    pub name: String,
    /// Default proof expiry in seconds
    pub proof_expiry_seconds: u64,
    /// Issuer identifier (DID or domain)
    pub issuer: Option<String>,
    /// Allowed actions (empty = all allowed)
    pub allowed_actions: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "unnamed-agent".to_string(),
            proof_expiry_seconds: 300, // 5 minutes
            issuer: None,
            allowed_actions: vec![],
        }
    }
}

impl AgentConfig {
    /// Create a new config with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set proof expiry.
    pub fn with_expiry(mut self, seconds: u64) -> Self {
        self.proof_expiry_seconds = seconds;
        self
    }

    /// Set issuer.
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }
}

/// Agent - The core identity unit in AgentKern.
///
/// An Agent holds a cryptographic keypair and can:
/// - Sign arbitrary data
/// - Create Liability Proofs
/// - Verify other agents' proofs
pub struct Agent {
    /// Unique agent ID (DID format)
    id: AgentId,
    /// Agent configuration
    config: AgentConfig,
    /// Ed25519 keypair
    keypair: KeyPair,
    /// Creation timestamp
    created_at: DateTime<Utc>,
}

impl Agent {
    /// Generate a new Agent with a random keypair.
    ///
    /// # Arguments
    /// * `name` - Human-readable agent name
    ///
    /// # Example
    /// ```rust,ignore
    /// let agent = Agent::generate("my-agent")?;
    /// ```
    pub fn generate(name: impl Into<String>) -> SdkResult<Self> {
        let keypair = KeyPair::generate()?;
        let config = AgentConfig::new(name);
        let id = Self::generate_did(&keypair.public_key());
        
        Ok(Self {
            id,
            config,
            keypair,
            created_at: Utc::now(),
        })
    }

    /// Generate agent with custom configuration.
    pub fn generate_with_config(config: AgentConfig) -> SdkResult<Self> {
        let keypair = KeyPair::generate()?;
        let id = Self::generate_did(&keypair.public_key());
        
        Ok(Self {
            id,
            config,
            keypair,
            created_at: Utc::now(),
        })
    }

    /// Restore an Agent from an existing keypair seed.
    pub fn from_seed(name: impl Into<String>, seed: &[u8]) -> SdkResult<Self> {
        let keypair = KeyPair::from_seed(seed)?;
        let config = AgentConfig::new(name);
        let id = Self::generate_did(&keypair.public_key());
        
        Ok(Self {
            id,
            config,
            keypair,
            created_at: Utc::now(),
        })
    }

    /// Get the agent's unique ID (DID format).
    pub fn id(&self) -> &AgentId {
        &self.id
    }

    /// Get the agent's name.
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get the agent's public key.
    pub fn public_key(&self) -> PublicKey {
        self.keypair.public_key()
    }

    /// Get the keypair seed for persistence (PKCS8 format).
    /// 
    /// **WARNING**: This is sensitive key material. Store securely.
    pub fn seed(&self) -> &[u8] {
        self.keypair.seed()
    }

    /// Get the keypair seed as base64url.
    pub fn seed_base64(&self) -> String {
        self.keypair.seed_base64()
    }

    /// Get agent creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    // =========================================================================
    // Signing Operations
    // =========================================================================

    /// Sign arbitrary data.
    ///
    /// Returns a base64url-encoded signature.
    pub fn sign(&self, data: &[u8]) -> String {
        self.keypair.sign(data).to_base64()
    }

    /// Sign a string message.
    pub fn sign_message(&self, message: &str) -> String {
        self.sign(message.as_bytes())
    }

    // =========================================================================
    // Liability Proof Operations
    // =========================================================================

    /// Create a Liability Proof for an action.
    ///
    /// # Arguments
    /// * `action` - The action being authorized (e.g., "payment:transfer:100")
    ///
    /// # Example
    /// ```rust,ignore
    /// let proof = agent.create_proof("payment:transfer:100")?;
    /// println!("Proof: {}", proof.to_jwt());
    /// ```
    pub fn create_proof(&self, action: &str) -> SdkResult<LiabilityProof> {
        self.create_proof_with_options(action, None, None)
    }

    /// Create a Liability Proof with custom options.
    pub fn create_proof_with_options(
        &self,
        action: &str,
        audience: Option<&str>,
        expires_in: Option<Duration>,
    ) -> SdkResult<LiabilityProof> {
        let now = Utc::now();
        let expiry = expires_in.unwrap_or(Duration::seconds(
            self.config.proof_expiry_seconds as i64,
        ));
        let exp = now + expiry;

        let header = ProofHeader {
            alg: "EdDSA".to_string(),
            typ: "LIABILITY+jwt".to_string(),
            kid: self.public_key().to_base64(),
        };

        let claims = ProofClaims {
            iss: self.config.issuer.clone().unwrap_or_else(|| self.id.clone()),
            sub: self.id.clone(),
            aud: audience.map(|s| s.to_string()),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            jti: Uuid::new_v4().to_string(),
            action: action.to_string(),
            scope: self.config.allowed_actions.clone(),
        };

        // Serialize header and claims
        let header_json = serde_json::to_string(&header)?;
        let claims_json = serde_json::to_string(&claims)?;

        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        let header_b64 = URL_SAFE_NO_PAD.encode(header_json.as_bytes());
        let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json.as_bytes());

        // Sign header.claims
        let signing_input = format!("{}.{}", header_b64, claims_b64);
        let signature = self.keypair.sign(signing_input.as_bytes());
        let sig_b64 = signature.to_base64();

        Ok(LiabilityProof {
            header,
            claims,
            signature: sig_b64.clone(),
            raw: format!("{}.{}.{}", header_b64, claims_b64, sig_b64),
        })
    }

    // =========================================================================
    // Verification Operations
    // =========================================================================

    /// Verify a Liability Proof.
    ///
    /// Validates:
    /// 1. Signature is valid
    /// 2. Proof is not expired
    /// 3. Claims are present
    pub fn verify_proof(proof: &LiabilityProof) -> SdkResult<bool> {
        // 1. Check expiration
        let now = Utc::now().timestamp();
        if proof.claims.exp < now {
            return Err(SdkError::ProofExpired(
                DateTime::from_timestamp(proof.claims.exp, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| "unknown".to_string()),
            ));
        }

        // 2. Extract public key from header
        let public_key = PublicKey::from_base64(&proof.header.kid)?;

        // 3. Reconstruct signing input
        let parts: Vec<&str> = proof.raw.split('.').collect();
        if parts.len() != 3 {
            return Err(SdkError::InvalidProofFormat(
                "Expected 3 parts (header.claims.signature)".into(),
            ));
        }

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let signature = crate::signing::Signature::from_base64(parts[2])?;

        // 4. Verify signature
        public_key.verify(signing_input.as_bytes(), &signature)
    }

    // =========================================================================
    // Private Helpers
    // =========================================================================

    /// Generate a DID from public key (did:key format).
    fn generate_did(public_key: &PublicKey) -> AgentId {
        // Use did:key format with multibase encoding
        let pk_b64 = public_key.to_base64();
        format!("did:key:z{}", pk_b64)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_generation() {
        let agent = Agent::generate("test-agent").unwrap();
        assert!(agent.id().starts_with("did:key:z"));
        assert_eq!(agent.name(), "test-agent");
    }

    #[test]
    fn test_agent_from_seed() {
        let agent1 = Agent::generate("agent1").unwrap();
        let seed = agent1.seed().to_vec();
        
        let agent2 = Agent::from_seed("agent2", &seed).unwrap();
        
        // Same keypair = same public key = same DID
        assert_eq!(agent1.id(), agent2.id());
        assert_eq!(agent1.public_key(), agent2.public_key());
    }

    #[test]
    fn test_agent_sign() {
        let agent = Agent::generate("signer").unwrap();
        let sig1 = agent.sign(b"message");
        let sig2 = agent.sign(b"message");
        
        // Same message = same signature (Ed25519 is deterministic)
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_create_proof() {
        let agent = Agent::generate("prover").unwrap();
        let proof = agent.create_proof("test:action").unwrap();
        
        assert_eq!(proof.claims.action, "test:action");
        assert!(proof.claims.exp > Utc::now().timestamp());
        assert!(!proof.raw.is_empty());
    }

    #[test]
    fn test_verify_proof() {
        let agent = Agent::generate("prover").unwrap();
        let proof = agent.create_proof("test:action").unwrap();
        
        let is_valid = Agent::verify_proof(&proof).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_expired_proof() {
        let config = AgentConfig::new("expirer")
            .with_expiry(0); // Expires immediately
        let agent = Agent::generate_with_config(config).unwrap();
        
        let proof = agent.create_proof_with_options(
            "test:action",
            None,
            Some(Duration::seconds(-10)), // Already expired
        ).unwrap();
        
        let result = Agent::verify_proof(&proof);
        assert!(matches!(result, Err(SdkError::ProofExpired(_))));
    }

    #[test]
    fn test_proof_jwt_format() {
        let agent = Agent::generate("jwt-test").unwrap();
        let proof = agent.create_proof("test").unwrap();
        
        // JWT should have 3 parts
        let parts: Vec<&str> = proof.raw.split('.').collect();
        assert_eq!(parts.len(), 3);
        
        // Each part should be valid base64url
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        for part in parts {
            assert!(URL_SAFE_NO_PAD.decode(part).is_ok());
        }
    }
}
