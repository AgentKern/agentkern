//! VeriMantle-Gate: Zero-Trust mTLS Module
//!
//! Per EXECUTION_MANDATE.md §5: "Zero-Trust Security & Agent Identity"
//!
//! Features:
//! - Mutual TLS (mTLS) enforcement
//! - Certificate validation
//! - Agent identity verification
//! - Just-in-Time credential issuance
//!
//! # Example
//!
//! ```rust,ignore
//! use verimantle_gate::mtls::{MtlsConfig, CertificateValidator};
//!
//! let validator = CertificateValidator::new(MtlsConfig::strict());
//! validator.validate_certificate(&cert)?;
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// mTLS errors.
#[derive(Debug, Error)]
pub enum MtlsError {
    #[error("Certificate expired")]
    CertificateExpired,
    #[error("Certificate not yet valid")]
    CertificateNotYetValid,
    #[error("Invalid certificate chain")]
    InvalidChain,
    #[error("Certificate revoked")]
    CertificateRevoked,
    #[error("Missing client certificate")]
    MissingClientCert,
    #[error("Untrusted issuer: {issuer}")]
    UntrustedIssuer { issuer: String },
    #[error("Agent identity mismatch")]
    IdentityMismatch,
}

/// mTLS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsConfig {
    /// Require client certificates
    pub require_client_cert: bool,
    /// Trusted CAs (base64 encoded)
    pub trusted_ca_certs: Vec<String>,
    /// Certificate revocation list URL
    pub crl_url: Option<String>,
    /// OCSP responder URL
    pub ocsp_url: Option<String>,
    /// Maximum certificate validity (days)
    pub max_cert_validity_days: u32,
    /// Allow self-signed (dev only!)
    pub allow_self_signed: bool,
}

impl Default for MtlsConfig {
    fn default() -> Self {
        Self {
            require_client_cert: true,
            trusted_ca_certs: vec![],
            crl_url: None,
            ocsp_url: None,
            max_cert_validity_days: 365,
            allow_self_signed: false,
        }
    }
}

impl MtlsConfig {
    /// Strict configuration for production.
    pub fn strict() -> Self {
        Self {
            require_client_cert: true,
            trusted_ca_certs: vec![],
            crl_url: Some("https://crl.verimantle.com/crl.pem".to_string()),
            ocsp_url: Some("https://ocsp.verimantle.com".to_string()),
            max_cert_validity_days: 90,
            allow_self_signed: false,
        }
    }

    /// Development configuration (allow self-signed).
    pub fn development() -> Self {
        Self {
            require_client_cert: true,
            trusted_ca_certs: vec![],
            crl_url: None,
            ocsp_url: None,
            max_cert_validity_days: 365,
            allow_self_signed: true,
        }
    }
}

/// Parsed certificate information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// Subject (agent identity)
    pub subject: String,
    /// Issuer
    pub issuer: String,
    /// Serial number
    pub serial: String,
    /// Not before (Unix timestamp)
    pub not_before: u64,
    /// Not after (Unix timestamp)
    pub not_after: u64,
    /// Key type
    pub key_type: KeyType,
    /// Key size in bits
    pub key_bits: u16,
    /// Is CA certificate
    pub is_ca: bool,
    /// Fingerprint (SHA256)
    pub fingerprint: String,
}

/// Key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    Rsa,
    EcdsaP256,
    EcdsaP384,
    Ed25519,
}

impl CertificateInfo {
    /// Check if certificate is currently valid.
    pub fn is_valid_now(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now >= self.not_before && now <= self.not_after
    }

    /// Get remaining validity in days.
    pub fn days_until_expiry(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        ((self.not_after as i64) - (now as i64)) / 86400
    }
}

/// Certificate validator for mTLS.
#[derive(Debug)]
pub struct CertificateValidator {
    config: MtlsConfig,
    revoked_serials: Vec<String>,
}

impl CertificateValidator {
    /// Create a new certificate validator.
    pub fn new(config: MtlsConfig) -> Self {
        Self {
            config,
            revoked_serials: vec![],
        }
    }

    /// Add a revoked certificate serial.
    pub fn revoke(&mut self, serial: String) {
        self.revoked_serials.push(serial);
    }

    /// Validate a certificate.
    pub fn validate(&self, cert: &CertificateInfo) -> Result<(), MtlsError> {
        // Check expiry
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now < cert.not_before {
            return Err(MtlsError::CertificateNotYetValid);
        }
        
        if now > cert.not_after {
            return Err(MtlsError::CertificateExpired);
        }

        // Check revocation
        if self.revoked_serials.contains(&cert.serial) {
            return Err(MtlsError::CertificateRevoked);
        }

        // Check validity period
        let validity_days = (cert.not_after - cert.not_before) / 86400;
        if validity_days > self.config.max_cert_validity_days as u64 {
            // Log warning but don't reject
            tracing::warn!(
                cert_days = validity_days,
                max_days = self.config.max_cert_validity_days,
                "Certificate validity exceeds recommended maximum"
            );
        }

        Ok(())
    }

    /// Validate an mTLS connection.
    pub fn validate_connection(
        &self,
        client_cert: Option<&CertificateInfo>,
        expected_agent_id: Option<&str>,
    ) -> Result<(), MtlsError> {
        // Check if client cert is required
        if self.config.require_client_cert {
            let cert = client_cert.ok_or(MtlsError::MissingClientCert)?;
            
            // Validate the certificate
            self.validate(cert)?;
            
            // Check agent identity if specified
            if let Some(expected_id) = expected_agent_id {
                if cert.subject != expected_id && !cert.subject.contains(expected_id) {
                    return Err(MtlsError::IdentityMismatch);
                }
            }
        }

        Ok(())
    }
}

/// Just-in-Time credential issuer.
#[derive(Debug)]
pub struct JitCredentialIssuer {
    /// Default credential TTL
    default_ttl: Duration,
}

impl Default for JitCredentialIssuer {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl JitCredentialIssuer {
    /// Create a new issuer with custom TTL.
    pub fn with_ttl(ttl: Duration) -> Self {
        Self { default_ttl: ttl }
    }

    /// Issue ephemeral credentials for an agent.
    pub fn issue(&self, agent_id: &str, scope: &str) -> EphemeralCredential {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        EphemeralCredential {
            credential_id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            scope: scope.to_string(),
            issued_at: now,
            expires_at: now + self.default_ttl.as_secs(),
            token: format!("JIT-{}-{}", agent_id, uuid::Uuid::new_v4()),
        }
    }
}

/// Ephemeral JIT credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralCredential {
    pub credential_id: String,
    pub agent_id: String,
    pub scope: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub token: String,
}

impl EphemeralCredential {
    /// Check if credential is still valid.
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now >= self.issued_at && now <= self.expires_at
    }

    /// Get remaining TTL in seconds.
    pub fn ttl_secs(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        (self.expires_at as i64) - now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_cert() -> CertificateInfo {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        CertificateInfo {
            subject: "CN=agent-123".to_string(),
            issuer: "CN=VeriMantle CA".to_string(),
            serial: "SN-12345".to_string(),
            not_before: now - 86400,
            not_after: now + 86400 * 30,
            key_type: KeyType::EcdsaP256,
            key_bits: 256,
            is_ca: false,
            fingerprint: "SHA256:abc123".to_string(),
        }
    }

    #[test]
    fn test_valid_certificate() {
        let validator = CertificateValidator::new(MtlsConfig::default());
        let cert = make_valid_cert();
        
        assert!(validator.validate(&cert).is_ok());
    }

    #[test]
    fn test_expired_certificate() {
        let validator = CertificateValidator::new(MtlsConfig::default());
        let mut cert = make_valid_cert();
        cert.not_after = 0; // Expired in 1970
        
        let result = validator.validate(&cert);
        assert!(matches!(result, Err(MtlsError::CertificateExpired)));
    }

    #[test]
    fn test_revoked_certificate() {
        let mut validator = CertificateValidator::new(MtlsConfig::default());
        let cert = make_valid_cert();
        
        validator.revoke(cert.serial.clone());
        
        let result = validator.validate(&cert);
        assert!(matches!(result, Err(MtlsError::CertificateRevoked)));
    }

    #[test]
    fn test_missing_client_cert() {
        let validator = CertificateValidator::new(MtlsConfig::strict());
        
        let result = validator.validate_connection(None, None);
        assert!(matches!(result, Err(MtlsError::MissingClientCert)));
    }

    #[test]
    fn test_jit_credentials() {
        let issuer = JitCredentialIssuer::default();
        let cred = issuer.issue("agent-456", "database:read");
        
        assert!(cred.is_valid());
        assert!(cred.ttl_secs() > 0);
        assert!(cred.token.contains("agent-456"));
    }
}
