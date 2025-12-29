//! Memory Passport: Zero-Trust Agent State Encryption
//!
//! Per Zero-Trust Security Model Migration:
//! - Memory Passports encapsulate agent state with field-level encryption
//! - DID-anchored verification for cross-agent access control
//! - Prevents "Intrapersonal Spying" between agents
//!
//! This addresses the P1 priority: Synapse ledger field-level encryption.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::encryption::{EncryptedEnvelope, EncryptionConfig, EncryptionEngine, EncryptionError};
use crate::types::AgentState;

// ============================================================================
// MEMORY PASSPORT TYPES
// ============================================================================

/// Sensitivity level for state fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldSensitivity {
    /// Public - no encryption needed
    Public,
    /// Internal - encrypted, same-agent access
    Internal,
    /// Confidential - encrypted, explicit grants only
    Confidential,
    /// Secret - encrypted, TEE-only access
    Secret,
}

impl Default for FieldSensitivity {
    fn default() -> Self {
        Self::Internal
    }
}

/// Access grant for cross-agent state sharing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessGrant {
    /// Grantee agent DID (e.g., did:key:z6Mk...)
    pub grantee_did: String,
    /// Fields accessible
    pub fields: Vec<String>,
    /// Grant expiration
    pub expires_at: Option<DateTime<Utc>>,
    /// Allow read access
    pub can_read: bool,
    /// Allow write access
    pub can_write: bool,
}

impl AccessGrant {
    /// Create a read-only grant for specific fields.
    pub fn read_only(grantee_did: impl Into<String>, fields: Vec<String>) -> Self {
        Self {
            grantee_did: grantee_did.into(),
            fields,
            expires_at: None,
            can_read: true,
            can_write: false,
        }
    }

    /// Check if grant is still valid.
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expires) => Utc::now() < expires,
            None => true,
        }
    }
}

/// Encrypted field container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField {
    /// Original field name
    pub name: String,
    /// Sensitivity level
    pub sensitivity: FieldSensitivity,
    /// Encrypted envelope (if encrypted)
    pub envelope: Option<EncryptedEnvelope>,
    /// Plaintext value (if public)
    pub plaintext: Option<serde_json::Value>,
}

/// Memory Passport: Encrypted agent state container.
///
/// Wraps AgentState with field-level encryption based on sensitivity.
/// Enables Zero-Trust cross-agent access control via DID grants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurePassport {
    /// Unique passport ID
    pub id: String,
    /// Owner agent DID
    pub owner_did: String,
    /// Encrypted fields
    pub fields: HashMap<String, EncryptedField>,
    /// Access grants for other agents
    pub grants: Vec<AccessGrant>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,
    /// Version for conflict resolution
    pub version: u64,
}

impl SecurePassport {
    /// Create a new empty Memory Passport.
    pub fn new(owner_did: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            owner_did: owner_did.into(),
            fields: HashMap::new(),
            grants: Vec::new(),
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// Create from an existing AgentState with encryption.
    pub fn from_agent_state(
        state: &AgentState,
        owner_did: impl Into<String>,
        engine: &EncryptionEngine,
        sensitivity_map: &HashMap<String, FieldSensitivity>,
    ) -> Result<Self, EncryptionError> {
        let mut passport = Self::new(owner_did);
        passport.id = format!("mp:{}", state.agent_id);

        for (key, value) in &state.state {
            let sensitivity = sensitivity_map
                .get(key)
                .copied()
                .unwrap_or(FieldSensitivity::Internal);

            let field = match sensitivity {
                FieldSensitivity::Public => EncryptedField {
                    name: key.clone(),
                    sensitivity,
                    envelope: None,
                    plaintext: Some(value.clone()),
                },
                _ => {
                    let envelope = engine.encrypt_value(value)?;
                    EncryptedField {
                        name: key.clone(),
                        sensitivity,
                        envelope: Some(envelope),
                        plaintext: None,
                    }
                }
            };

            passport.fields.insert(key.clone(), field);
        }

        passport.version = state.version;
        passport.updated_at = state.updated_at;

        Ok(passport)
    }

    /// Decrypt and extract to AgentState.
    pub fn to_agent_state(
        &self,
        agent_id: impl Into<String>,
        engine: &EncryptionEngine,
    ) -> Result<AgentState, EncryptionError> {
        let mut state = AgentState::new(agent_id);

        for (key, field) in &self.fields {
            let value = match field.sensitivity {
                FieldSensitivity::Public => {
                    field.plaintext.clone().unwrap_or(serde_json::Value::Null)
                }
                _ => {
                    if let Some(envelope) = &field.envelope {
                        engine.decrypt_value(envelope)?
                    } else {
                        serde_json::Value::Null
                    }
                }
            };
            state.state.insert(key.clone(), value);
        }

        state.version = self.version;
        state.updated_at = self.updated_at;

        Ok(state)
    }

    /// Grant access to another agent.
    pub fn grant_access(&mut self, grant: AccessGrant) {
        // Remove any existing grant for the same grantee
        self.grants.retain(|g| g.grantee_did != grant.grantee_did);
        self.grants.push(grant);
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// Revoke access for an agent.
    pub fn revoke_access(&mut self, grantee_did: &str) {
        self.grants.retain(|g| g.grantee_did != grantee_did);
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// Check if an agent has read access to a field.
    pub fn can_read(&self, requester_did: &str, field_name: &str) -> bool {
        // Owner always has access
        if requester_did == self.owner_did {
            return true;
        }

        // Check grants
        for grant in &self.grants {
            if grant.grantee_did == requester_did
                && grant.can_read
                && grant.is_valid()
                && (grant.fields.is_empty() || grant.fields.contains(&field_name.to_string()))
            {
                return true;
            }
        }

        false
    }

    /// Get a decrypted field value (with access check).
    pub fn get_field(
        &self,
        field_name: &str,
        requester_did: &str,
        engine: &EncryptionEngine,
    ) -> Result<Option<serde_json::Value>, SecurePassportError> {
        if !self.can_read(requester_did, field_name) {
            return Err(SecurePassportError::AccessDenied {
                requester: requester_did.to_string(),
                field: field_name.to_string(),
            });
        }

        let field = match self.fields.get(field_name) {
            Some(f) => f,
            None => return Ok(None),
        };

        let value = match field.sensitivity {
            FieldSensitivity::Public => field.plaintext.clone(),
            _ => {
                if let Some(envelope) = &field.envelope {
                    Some(engine.decrypt_value(envelope).map_err(SecurePassportError::from)?)
                } else {
                    None
                }
            }
        };

        Ok(value)
    }

    /// Set a field value with encryption.
    pub fn set_field(
        &mut self,
        field_name: impl Into<String>,
        value: serde_json::Value,
        sensitivity: FieldSensitivity,
        engine: &EncryptionEngine,
    ) -> Result<(), EncryptionError> {
        let name = field_name.into();

        let field = match sensitivity {
            FieldSensitivity::Public => EncryptedField {
                name: name.clone(),
                sensitivity,
                envelope: None,
                plaintext: Some(value),
            },
            _ => {
                let envelope = engine.encrypt_value(&value)?;
                EncryptedField {
                    name: name.clone(),
                    sensitivity,
                    envelope: Some(envelope),
                    plaintext: None,
                }
            }
        };

        self.fields.insert(name, field);
        self.updated_at = Utc::now();
        self.version += 1;

        Ok(())
    }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Memory Passport errors.
#[derive(Debug, thiserror::Error)]
pub enum SecurePassportError {
    #[error("Access denied: {requester} cannot access field {field}")]
    AccessDenied { requester: String, field: String },
    #[error("Encryption error: {0}")]
    Encryption(#[from] EncryptionError),
    #[error("Invalid passport format")]
    InvalidFormat,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_passport_new() {
        let passport = SecurePassport::new("did:key:z6MkOwner");
        assert_eq!(passport.owner_did, "did:key:z6MkOwner");
        assert!(passport.fields.is_empty());
        assert!(passport.grants.is_empty());
    }

    #[test]
    fn test_field_encryption_roundtrip() {
        let engine = EncryptionEngine::new();
        let mut passport = SecurePassport::new("did:key:z6MkOwner");

        passport
            .set_field(
                "secret_key",
                serde_json::json!("my-secret-value"),
                FieldSensitivity::Secret,
                &engine,
            )
            .unwrap();

        // Field should be encrypted
        let field = passport.fields.get("secret_key").unwrap();
        assert!(field.envelope.is_some());
        assert!(field.plaintext.is_none());

        // Owner can decrypt
        let value = passport
            .get_field("secret_key", "did:key:z6MkOwner", &engine)
            .unwrap();
        assert_eq!(value, Some(serde_json::json!("my-secret-value")));
    }

    #[test]
    fn test_public_field_no_encryption() {
        let engine = EncryptionEngine::new();
        let mut passport = SecurePassport::new("did:key:z6MkOwner");

        passport
            .set_field(
                "public_name",
                serde_json::json!("Agent XYZ"),
                FieldSensitivity::Public,
                &engine,
            )
            .unwrap();

        let field = passport.fields.get("public_name").unwrap();
        assert!(field.envelope.is_none());
        assert_eq!(field.plaintext, Some(serde_json::json!("Agent XYZ")));
    }

    #[test]
    fn test_access_control() {
        let engine = EncryptionEngine::new();
        let mut passport = SecurePassport::new("did:key:z6MkOwner");

        passport
            .set_field(
                "secret",
                serde_json::json!("hidden"),
                FieldSensitivity::Confidential,
                &engine,
            )
            .unwrap();

        // Unauthorized agent cannot access
        let result = passport.get_field("secret", "did:key:z6MkUnauthorized", &engine);
        assert!(matches!(result, Err(SecurePassportError::AccessDenied { .. })));

        // Grant access
        passport.grant_access(AccessGrant::read_only(
            "did:key:z6MkAuthorized",
            vec!["secret".to_string()],
        ));

        // Now authorized
        let value = passport
            .get_field("secret", "did:key:z6MkAuthorized", &engine)
            .unwrap();
        assert_eq!(value, Some(serde_json::json!("hidden")));
    }

    #[test]
    fn test_from_agent_state() {
        let engine = EncryptionEngine::new();

        let mut state = AgentState::new("agent-1");
        state.state.insert("name".to_string(), serde_json::json!("TestAgent"));
        state.state.insert("api_key".to_string(), serde_json::json!("secret-key"));

        let mut sensitivity = HashMap::new();
        sensitivity.insert("name".to_string(), FieldSensitivity::Public);
        sensitivity.insert("api_key".to_string(), FieldSensitivity::Secret);

        let passport = SecurePassport::from_agent_state(
            &state,
            "did:key:z6MkTest",
            &engine,
            &sensitivity,
        )
        .unwrap();

        // Name should be public
        let name_field = passport.fields.get("name").unwrap();
        assert_eq!(name_field.sensitivity, FieldSensitivity::Public);

        // API key should be encrypted
        let key_field = passport.fields.get("api_key").unwrap();
        assert_eq!(key_field.sensitivity, FieldSensitivity::Secret);
        assert!(key_field.envelope.is_some());
    }

    #[test]
    fn test_revoke_access() {
        let mut passport = SecurePassport::new("did:key:z6MkOwner");

        passport.grant_access(AccessGrant::read_only(
            "did:key:z6MkGrantee",
            vec!["field1".to_string()],
        ));
        assert_eq!(passport.grants.len(), 1);

        passport.revoke_access("did:key:z6MkGrantee");
        assert!(passport.grants.is_empty());
    }
}
