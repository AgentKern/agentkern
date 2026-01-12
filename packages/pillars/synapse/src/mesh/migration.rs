//! Synapse Cross-Cloud Migration
//!
//! Mobility mechanism for moving agent memory between regions.
//! Per GLOBAL_GAPS.md: "Cloud-Agnostic Hibernate"

use super::{DataRegion, MeshError};
use crate::encryption::{EncryptedEnvelope, EncryptionEngine};
use crate::passport::MemoryPassport;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Migration ticket for resuming state in a new region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationTicket {
    pub ticket_id: String,
    pub agent_id: String,
    pub source_region: DataRegion,
    pub target_region: DataRegion,
    pub expires_at: u64,
    /// Encrypted payload containing the passport (optional, or stored in blob)
    pub payload: Option<EncryptedEnvelope>,
}

/// Cross-cloud migration engine.
pub struct MigrationManager {
    encryption: EncryptionEngine,
}

impl MigrationManager {
    /// Create new migration manager.
    pub fn new(encryption: EncryptionEngine) -> Self {
        Self { encryption }
    }

    /// Hibernate an agent: encrypt state and prepare migration ticket.
    pub fn hibernate(
        &self,
        passport: MemoryPassport,
        target_region: DataRegion,
    ) -> Result<MigrationTicket, MeshError> {
        let agent_id = passport.identity.did.clone();

        // Encrypt the passport for migration
        let envelope = self
            .encryption
            .encrypt_value(&passport)
            .map_err(|e| MeshError::SyncFailed(format!("Encryption failed: {}", e)))?;

        Ok(MigrationTicket {
            ticket_id: Uuid::new_v4().to_string(),
            agent_id,
            source_region: DataRegion::Global, // Simplified for now
            target_region,
            expires_at: chrono::Utc::now().timestamp() as u64 + 3600, // 1h expiry
            payload: Some(envelope),
        })
    }

    /// Wakeup an agent: verify ticket and decrypt state.
    pub fn wakeup(
        &self,
        ticket: MigrationTicket,
        current_region: DataRegion,
    ) -> Result<MemoryPassport, MeshError> {
        // Verify target region matches
        if ticket.target_region != current_region && ticket.target_region != DataRegion::Global {
            return Err(MeshError::GeoFenceBlocked {
                reason: "Migration ticket intended for different region".into(),
            });
        }

        // Verify expiry
        if ticket.expires_at < chrono::Utc::now().timestamp() as u64 {
            return Err(MeshError::SyncFailed("Migration ticket expired".into()));
        }

        let envelope = ticket
            .payload
            .ok_or_else(|| MeshError::SyncFailed("Migration ticket missing payload".into()))?;

        // Decrypt passport
        let passport: MemoryPassport = self
            .encryption
            .decrypt_value(&envelope)
            .map_err(|e| MeshError::SyncFailed(format!("Decryption failed: {}", e)))?;

        Ok(passport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::EncryptionEngine;
    use crate::mesh::DataRegion;
    use crate::passport::{AgentIdentity, MemoryPassport};

    fn sample_identity() -> AgentIdentity {
        AgentIdentity {
            did: "did:agentkern:agent-x".to_string(),
            public_key: "base64pubkey".to_string(),
            algorithm: "Ed25519".to_string(),
            created_at: 1700000000000,
            updated_at: 1700000000000,
        }
    }

    #[test]
    fn test_migration_roundtrip() {
        let encryption = EncryptionEngine::new();
        let manager = MigrationManager::new(encryption);

        let passport = MemoryPassport::new(sample_identity(), "US".to_string());

        // Hibernate
        let ticket = manager
            .hibernate(passport.clone(), DataRegion::EuFrankfurt)
            .unwrap();
        assert_eq!(ticket.agent_id, "did:agentkern:agent-x");
        assert!(ticket.payload.is_some());

        // Wakeup
        let restored = manager.wakeup(ticket, DataRegion::EuFrankfurt).unwrap();
        assert_eq!(restored.identity.did, "did:agentkern:agent-x");
    }

    #[test]
    fn test_migration_region_mismatch() {
        let encryption = EncryptionEngine::new();
        let manager = MigrationManager::new(encryption);
        let passport = MemoryPassport::new(sample_identity(), "US".to_string());

        let ticket = manager.hibernate(passport, DataRegion::MenaRiyadh).unwrap();

        // Attempt wakeup in wrong region
        let result = manager.wakeup(ticket, DataRegion::UsEast);
        assert!(matches!(result, Err(MeshError::GeoFenceBlocked { .. })));
    }
}
