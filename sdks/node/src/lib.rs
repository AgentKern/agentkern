//! AgentKern SDK Node.js Bindings
//!
//! N-API bindings for sdk-core, enabling TypeScript/JavaScript consumption.
//! Generated with napi-rs for native performance.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

use agentkern_sdk_core::{
    agent::{Agent as CoreAgent, AgentConfig as CoreAgentConfig},
    proof::LiabilityProof as CoreLiabilityProof,
    protocol::{A2AMessage as CoreA2AMessage, MessageType as CoreMessageType},
};

/// AgentKern SDK version
#[napi]
pub const VERSION: &str = agentkern_sdk_core::VERSION;

// ============================================================================
// AGENT
// ============================================================================

/// Agent configuration options.
#[napi(object)]
#[derive(Clone)]
pub struct AgentConfig {
    /// Agent name/label
    pub name: String,
    /// Default proof expiry in seconds
    pub proof_expiry_seconds: Option<u32>,
    /// Issuer identifier (DID or domain)
    pub issuer: Option<String>,
}

/// Agent - The core identity unit in AgentKern.
#[napi]
pub struct Agent {
    inner: CoreAgent,
}

#[napi]
impl Agent {
    /// Generate a new Agent with a random Ed25519 keypair.
    #[napi(factory)]
    pub fn generate(name: String) -> Result<Agent> {
        let inner = CoreAgent::generate(&name).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Agent { inner })
    }

    /// Generate a new Agent with custom configuration.
    #[napi(factory)]
    pub fn generate_with_config(config: AgentConfig) -> Result<Agent> {
        let core_config = CoreAgentConfig {
            name: config.name,
            proof_expiry_seconds: config.proof_expiry_seconds.unwrap_or(300) as u64,
            issuer: config.issuer,
            allowed_actions: vec![],
        };
        let inner = CoreAgent::generate_with_config(core_config)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Agent { inner })
    }

    /// Restore an Agent from an existing keypair seed (base64url encoded).
    #[napi(factory)]
    pub fn from_seed(name: String, seed_base64: String) -> Result<Agent> {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        let seed = URL_SAFE_NO_PAD
            .decode(&seed_base64)
            .map_err(|e| Error::from_reason(format!("Invalid base64: {}", e)))?;
        let inner =
            CoreAgent::from_seed(&name, &seed).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Agent { inner })
    }

    /// Get the agent's unique ID (DID format).
    #[napi(getter)]
    pub fn id(&self) -> String {
        self.inner.id().clone()
    }

    /// Get the agent's name.
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get the agent's public key (base64url encoded).
    #[napi(getter)]
    pub fn public_key(&self) -> String {
        self.inner.public_key().to_base64()
    }

    /// Get the keypair seed for persistence (base64url encoded).
    /// WARNING: This is sensitive key material. Store securely.
    #[napi(getter)]
    pub fn seed(&self) -> String {
        self.inner.seed_base64()
    }

    /// Sign arbitrary data (returns base64url signature).
    #[napi]
    pub fn sign(&self, data: Buffer) -> String {
        self.inner.sign(&data)
    }

    /// Sign a string message (returns base64url signature).
    #[napi]
    pub fn sign_message(&self, message: String) -> String {
        self.inner.sign_message(&message)
    }

    /// Create a Liability Proof for an action.
    #[napi]
    pub fn create_proof(&self, action: String) -> Result<LiabilityProof> {
        let proof = self
            .inner
            .create_proof(&action)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(LiabilityProof::from_core(proof))
    }

    /// Create a Liability Proof with custom expiry (seconds).
    #[napi]
    pub fn create_proof_with_expiry(
        &self,
        action: String,
        expiry_seconds: i64,
    ) -> Result<LiabilityProof> {
        let proof = self
            .inner
            .create_proof_with_options(
                &action,
                None,
                Some(chrono::Duration::seconds(expiry_seconds)),
            )
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(LiabilityProof::from_core(proof))
    }

    /// Verify a Liability Proof (static method).
    #[napi]
    pub fn verify_proof(proof: LiabilityProof) -> Result<bool> {
        // Prefer parsing the JWT and failing loudly rather than using a silent fallback
        let core_proof = CoreLiabilityProof::from_jwt(&proof.jwt)
            .map_err(|e| Error::from_reason(format!("Invalid JWT proof: {}", e)))?;

        CoreAgent::verify_proof(&core_proof).map_err(|e| Error::from_reason(e.to_string()))
    }
}

// ============================================================================
// LIABILITY PROOF
// ============================================================================

/// Liability Proof - A signed JWT proving authorization.
#[napi(object)]
#[derive(Clone)]
pub struct LiabilityProof {
    /// JWT algorithm
    pub alg: String,
    /// JWT type
    pub typ: String,
    /// Key ID (public key)
    pub kid: String,
    /// Issuer (DID)
    pub issuer: String,
    /// Subject (agent DID)
    pub subject: String,
    /// Authorized action
    pub action: String,
    /// Issued at (Unix timestamp)
    pub issued_at: i64,
    /// Expiration (Unix timestamp)
    pub expires_at: i64,
    /// JWT ID
    pub jti: String,
    /// Full raw JWT string
    pub jwt: String,
}

impl LiabilityProof {
    fn from_core(proof: CoreLiabilityProof) -> Self {
        Self {
            alg: proof.header.alg.clone(),
            typ: proof.header.typ.clone(),
            kid: proof.header.kid.clone(),
            issuer: proof.claims.iss.clone(),
            subject: proof.claims.sub.clone(),
            action: proof.claims.action.clone(),
            issued_at: proof.claims.iat,
            expires_at: proof.claims.exp,
            jti: proof.claims.jti.clone(),
            jwt: proof.raw.clone(),
        }
    }

    fn to_core(&self) -> CoreLiabilityProof {
        CoreLiabilityProof::from_jwt(&self.jwt).unwrap_or_else(|_| {
            // Fallback: construct from fields
            CoreLiabilityProof {
                header: agentkern_sdk_core::proof::ProofHeader {
                    alg: self.alg.clone(),
                    typ: self.typ.clone(),
                    kid: self.kid.clone(),
                },
                claims: agentkern_sdk_core::proof::ProofClaims {
                    iss: self.issuer.clone(),
                    sub: self.subject.clone(),
                    aud: None,
                    iat: self.issued_at,
                    exp: self.expires_at,
                    jti: self.jti.clone(),
                    action: self.action.clone(),
                    scope: vec![],
                },
                signature: String::new(),
                raw: self.jwt.clone(),
            }
        })
    }
}

/// Parse a JWT string into a LiabilityProof.
#[napi]
pub fn parse_proof(jwt: String) -> Result<LiabilityProof> {
    let proof =
        CoreLiabilityProof::from_jwt(&jwt).map_err(|e| Error::from_reason(e.to_string()))?;
    Ok(LiabilityProof::from_core(proof))
}

/// Check if a proof is expired.
#[napi]
pub fn is_proof_expired(proof: LiabilityProof) -> bool {
    let now = chrono::Utc::now().timestamp();
    proof.expires_at < now
}

// ============================================================================
// A2A PROTOCOL
// ============================================================================

/// A2A Message types.
#[napi(string_enum)]
pub enum MessageType {
    Request,
    Response,
    Notification,
    Error,
    Ping,
    Pong,
    Capabilities,
}

impl From<CoreMessageType> for MessageType {
    fn from(mt: CoreMessageType) -> Self {
        match mt {
            CoreMessageType::Request => MessageType::Request,
            CoreMessageType::Response => MessageType::Response,
            CoreMessageType::Notification => MessageType::Notification,
            CoreMessageType::Error => MessageType::Error,
            CoreMessageType::Ping => MessageType::Ping,
            CoreMessageType::Pong => MessageType::Pong,
            CoreMessageType::Capabilities => MessageType::Capabilities,
        }
    }
}

/// Create an A2A request message.
#[napi]
pub fn create_a2a_request(from: String, to: String, payload: serde_json::Value) -> Result<String> {
    let msg = CoreA2AMessage::request(&from, &to, payload);
    msg.to_json().map_err(|e| Error::from_reason(e.to_string()))
}

/// Create an A2A notification message.
#[napi]
pub fn create_a2a_notification(
    from: String,
    to: String,
    payload: serde_json::Value,
) -> Result<String> {
    let msg = CoreA2AMessage::notification(&from, &to, payload);
    msg.to_json().map_err(|e| Error::from_reason(e.to_string()))
}

/// Parse an A2A message from JSON.
#[napi]
pub fn parse_a2a_message(json: String) -> Result<serde_json::Value> {
    let msg = CoreA2AMessage::from_json(&json).map_err(|e| Error::from_reason(e.to_string()))?;
    serde_json::to_value(&msg).map_err(|e| Error::from_reason(e.to_string()))
}
