//! AgentKern Python SDK Bindings
//!
//! PyO3 bindings for sdk-core, enabling Python consumption.
//! Built with maturin for easy wheel distribution.

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

use agentkern_sdk_core::{
    agent::{Agent as CoreAgent, AgentConfig as CoreAgentConfig},
    proof::LiabilityProof as CoreLiabilityProof,
    protocol::A2AMessage as CoreA2AMessage,
};

/// SDK Version
#[pyfunction]
fn version() -> &'static str {
    agentkern_sdk_core::VERSION
}

// ============================================================================
// AGENT
// ============================================================================

/// Agent - The core identity unit in AgentKern.
///
/// Holds an Ed25519 keypair and can:
/// - Sign arbitrary data
/// - Create Liability Proofs
/// - Verify other agents' proofs
#[pyclass]
#[derive(Clone)]
pub struct Agent {
    inner: AgentWrapper,
}

// Wrapper to handle non-Clone CoreAgent
#[derive(Clone)]
struct AgentWrapper {
    id: String,
    name: String,
    public_key: String,
    seed: Vec<u8>,
}

impl AgentWrapper {
    fn from_core(agent: &CoreAgent) -> Self {
        Self {
            id: agent.id().clone(),
            name: agent.name().to_string(),
            public_key: agent.public_key().to_base64(),
            seed: agent.seed().to_vec(),
        }
    }

    fn to_core(&self) -> Result<CoreAgent, PyErr> {
        CoreAgent::from_seed(&self.name, &self.seed)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[pymethods]
impl Agent {
    /// Generate a new Agent with a random Ed25519 keypair.
    #[staticmethod]
    fn generate(name: &str) -> PyResult<Self> {
        let core = CoreAgent::generate(name)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            inner: AgentWrapper::from_core(&core),
        })
    }

    /// Generate a new Agent with custom configuration.
    #[staticmethod]
    #[pyo3(signature = (name, proof_expiry_seconds=None, issuer=None))]
    fn generate_with_config(
        name: &str,
        proof_expiry_seconds: Option<u64>,
        issuer: Option<&str>,
    ) -> PyResult<Self> {
        let config = CoreAgentConfig {
            name: name.to_string(),
            proof_expiry_seconds: proof_expiry_seconds.unwrap_or(300),
            issuer: issuer.map(|s| s.to_string()),
            allowed_actions: vec![],
        };
        let core = CoreAgent::generate_with_config(config)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            inner: AgentWrapper::from_core(&core),
        })
    }

    /// Restore an Agent from an existing keypair seed (base64url encoded).
    #[staticmethod]
    fn from_seed(name: &str, seed_base64: &str) -> PyResult<Self> {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        let seed = URL_SAFE_NO_PAD.decode(seed_base64)
            .map_err(|e| PyValueError::new_err(format!("Invalid base64: {}", e)))?;
        let core = CoreAgent::from_seed(name, &seed)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            inner: AgentWrapper::from_core(&core),
        })
    }

    /// Agent's unique ID (DID format).
    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }

    /// Agent's name.
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    /// Agent's public key (base64url encoded).
    #[getter]
    fn public_key(&self) -> &str {
        &self.inner.public_key
    }

    /// Keypair seed for persistence (base64url encoded) - SENSITIVE.
    #[getter]
    fn seed(&self) -> String {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        URL_SAFE_NO_PAD.encode(&self.inner.seed)
    }

    /// Sign arbitrary data bytes (returns base64url signature).
    fn sign(&self, data: &[u8]) -> PyResult<String> {
        let core = self.inner.to_core()?;
        Ok(core.sign(data))
    }

    /// Sign a string message (returns base64url signature).
    fn sign_message(&self, message: &str) -> PyResult<String> {
        self.sign(message.as_bytes())
    }

    /// Create a Liability Proof for an action.
    fn create_proof(&self, action: &str) -> PyResult<LiabilityProof> {
        let core = self.inner.to_core()?;
        let proof = core.create_proof(action)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(LiabilityProof::from_core(proof))
    }

    /// Verify a Liability Proof (static method).
    #[staticmethod]
    fn verify_proof(proof: &LiabilityProof) -> PyResult<bool> {
        let core_proof = proof.to_core()?;
        CoreAgent::verify_proof(&core_proof)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn __repr__(&self) -> String {
        format!("Agent(id='{}', name='{}')", self.inner.id, self.inner.name)
    }
}

// ============================================================================
// LIABILITY PROOF
// ============================================================================

/// Liability Proof - A signed JWT proving authorization.
#[pyclass]
#[derive(Clone)]
pub struct LiabilityProof {
    /// Issuer (DID)
    #[pyo3(get)]
    pub issuer: String,
    /// Subject (agent DID)
    #[pyo3(get)]
    pub subject: String,
    /// Authorized action
    #[pyo3(get)]
    pub action: String,
    /// Issued at (Unix timestamp)
    #[pyo3(get)]
    pub issued_at: i64,
    /// Expiration (Unix timestamp)
    #[pyo3(get)]
    pub expires_at: i64,
    /// JWT ID
    #[pyo3(get)]
    pub jti: String,
    /// Full raw JWT string
    #[pyo3(get)]
    pub jwt: String,
}

impl LiabilityProof {
    fn from_core(proof: CoreLiabilityProof) -> Self {
        Self {
            issuer: proof.claims.iss.clone(),
            subject: proof.claims.sub.clone(),
            action: proof.claims.action.clone(),
            issued_at: proof.claims.iat,
            expires_at: proof.claims.exp,
            jti: proof.claims.jti.clone(),
            jwt: proof.raw.clone(),
        }
    }

    fn to_core(&self) -> PyResult<CoreLiabilityProof> {
        CoreLiabilityProof::from_jwt(&self.jwt)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[pymethods]
impl LiabilityProof {
    /// Check if this proof is expired.
    fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at < now
    }

    fn __repr__(&self) -> String {
        format!(
            "LiabilityProof(action='{}', expires_at={})",
            self.action, self.expires_at
        )
    }
}

/// Parse a JWT string into a LiabilityProof.
#[pyfunction]
fn parse_proof(jwt: &str) -> PyResult<LiabilityProof> {
    let proof = CoreLiabilityProof::from_jwt(jwt)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(LiabilityProof::from_core(proof))
}

// ============================================================================
// A2A PROTOCOL
// ============================================================================

/// Create an A2A request message.
#[pyfunction]
fn create_a2a_request(from: &str, to: &str, payload: &str) -> PyResult<String> {
    let payload_json: serde_json::Value = serde_json::from_str(payload)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
    let msg = CoreA2AMessage::request(from, to, payload_json);
    msg.to_json().map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Create an A2A notification message.
#[pyfunction]
fn create_a2a_notification(from: &str, to: &str, payload: &str) -> PyResult<String> {
    let payload_json: serde_json::Value = serde_json::from_str(payload)
        .map_err(|e| PyValueError::new_err(format!("Invalid JSON: {}", e)))?;
    let msg = CoreA2AMessage::notification(from, to, payload_json);
    msg.to_json().map_err(|e| PyValueError::new_err(e.to_string()))
}

// ============================================================================
// MODULE
// ============================================================================

/// AgentKern Python SDK
#[pymodule]
fn agentkern(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", agentkern_sdk_core::VERSION)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(parse_proof, m)?)?;
    m.add_function(wrap_pyfunction!(create_a2a_request, m)?)?;
    m.add_function(wrap_pyfunction!(create_a2a_notification, m)?)?;
    m.add_class::<Agent>()?;
    m.add_class::<LiabilityProof>()?;
    Ok(())
}
