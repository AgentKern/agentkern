//! Liability Proof Module
//!
//! JWT-based Liability Proofs that cryptographically prove:
//! 1. A specific human authorized a specific AI agent action
//! 2. The authorization was made via a hardware-bound credential
//! 3. The authorizer explicitly accepts liability

use serde::{Deserialize, Serialize};

/// Liability Proof - A signed JWT proving authorization and liability.
///
/// Format: `header.claims.signature` (JWT-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilityProof {
    /// JWT header
    pub header: ProofHeader,
    /// JWT claims (payload)
    pub claims: ProofClaims,
    /// Base64url-encoded signature
    pub signature: String,
    /// Full raw JWT string
    pub raw: String,
}

impl LiabilityProof {
    /// Get the full JWT string.
    pub fn to_jwt(&self) -> &str {
        &self.raw
    }

    /// Parse a JWT string into a LiabilityProof.
    pub fn from_jwt(jwt: &str) -> Result<Self, crate::error::SdkError> {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(crate::error::SdkError::InvalidProofFormat(
                "Expected 3 parts (header.claims.signature)".into(),
            ));
        }

        // Decode header
        let header_bytes = URL_SAFE_NO_PAD.decode(parts[0])?;
        let header: ProofHeader = serde_json::from_slice(&header_bytes)?;

        // Decode claims
        let claims_bytes = URL_SAFE_NO_PAD.decode(parts[1])?;
        let claims: ProofClaims = serde_json::from_slice(&claims_bytes)?;

        Ok(Self {
            header,
            claims,
            signature: parts[2].to_string(),
            raw: jwt.to_string(),
        })
    }

    /// Get the issuer (who created this proof).
    pub fn issuer(&self) -> &str {
        &self.claims.iss
    }

    /// Get the subject (the agent being authorized).
    pub fn subject(&self) -> &str {
        &self.claims.sub
    }

    /// Get the authorized action.
    pub fn action(&self) -> &str {
        &self.claims.action
    }

    /// Check if the proof is expired.
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.claims.exp < now
    }

    /// Get expiration timestamp.
    pub fn expires_at(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(self.claims.exp, 0)
            .unwrap_or_else(chrono::Utc::now)
    }

    /// Get the unique proof ID.
    pub fn jti(&self) -> &str {
        &self.claims.jti
    }
}

/// JWT Header for Liability Proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofHeader {
    /// Algorithm (EdDSA for Ed25519)
    pub alg: String,
    /// Type (LIABILITY+jwt)
    pub typ: String,
    /// Key ID (base64url public key)
    pub kid: String,
}

impl Default for ProofHeader {
    fn default() -> Self {
        Self {
            alg: "EdDSA".to_string(),
            typ: "LIABILITY+jwt".to_string(),
            kid: String::new(),
        }
    }
}

/// JWT Claims for Liability Proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofClaims {
    /// Issuer (DID or domain)
    pub iss: String,
    /// Subject (agent DID)
    pub sub: String,
    /// Audience (optional target)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration (Unix timestamp)
    pub exp: i64,
    /// JWT ID (unique identifier for this proof)
    pub jti: String,
    /// Authorized action
    pub action: String,
    /// Authorized scopes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scope: Vec<String>,
}

impl ProofClaims {
    /// Check if a specific action is authorized.
    pub fn authorizes(&self, action: &str) -> bool {
        // Exact match
        if self.action == action {
            return true;
        }
        
        // Wildcard match (e.g., "payment:*" matches "payment:transfer")
        if self.action.ends_with(":*") {
            let prefix = &self.action[..self.action.len() - 1];
            if action.starts_with(prefix) {
                return true;
            }
        }
        
        // Check scopes
        self.scope.iter().any(|s| s == action || s == "*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_from_jwt() {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
        
        let header = ProofHeader {
            alg: "EdDSA".to_string(),
            typ: "LIABILITY+jwt".to_string(),
            kid: "test-key".to_string(),
        };
        
        let claims = ProofClaims {
            iss: "did:key:zTest".to_string(),
            sub: "did:key:zTest".to_string(),
            aud: None,
            iat: 1000,
            exp: 9999999999,
            jti: "test-jti".to_string(),
            action: "test:action".to_string(),
            scope: vec![],
        };
        
        let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&header).unwrap());
        let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_string(&claims).unwrap());
        let sig_b64 = URL_SAFE_NO_PAD.encode(vec![0u8; 64]);
        
        let jwt = format!("{}.{}.{}", header_b64, claims_b64, sig_b64);
        let proof = LiabilityProof::from_jwt(&jwt).unwrap();
        
        assert_eq!(proof.issuer(), "did:key:zTest");
        assert_eq!(proof.action(), "test:action");
    }

    #[test]
    fn test_authorizes_exact() {
        let claims = ProofClaims {
            iss: "issuer".into(),
            sub: "subject".into(),
            aud: None,
            iat: 0,
            exp: 9999999999,
            jti: "jti".into(),
            action: "payment:transfer".into(),
            scope: vec![],
        };
        
        assert!(claims.authorizes("payment:transfer"));
        assert!(!claims.authorizes("payment:withdraw"));
    }

    #[test]
    fn test_authorizes_wildcard() {
        let claims = ProofClaims {
            iss: "issuer".into(),
            sub: "subject".into(),
            aud: None,
            iat: 0,
            exp: 9999999999,
            jti: "jti".into(),
            action: "payment:*".into(),
            scope: vec![],
        };
        
        assert!(claims.authorizes("payment:transfer"));
        assert!(claims.authorizes("payment:withdraw"));
        assert!(!claims.authorizes("data:read"));
    }

    #[test]
    fn test_is_expired() {
        let claims = ProofClaims {
            iss: "issuer".into(),
            sub: "subject".into(),
            aud: None,
            iat: 0,
            exp: 9999999999,
            jti: "jti".into(),
            action: "test".into(),
            scope: vec![],
        };
        
        let proof = LiabilityProof {
            header: ProofHeader::default(),
            claims: claims.clone(),
            signature: String::new(),
            raw: String::new(),
        };
        
        assert!(!proof.is_expired());
    }
}
