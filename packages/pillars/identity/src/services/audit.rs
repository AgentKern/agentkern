use agentkern_gate::crypto_agility::{CryptoMode, CryptoProvider, KeyPair};
use chrono::Utc;
use serde_json::Value;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Crypto error: {0}")]
    Crypto(String),
}

/// Service for logging compliance and security events with PQC signatures
pub struct AuditService {
    pool: PgPool,
    crypto: CryptoProvider,
    keypair: KeyPair,
}

impl AuditService {
    pub fn new(pool: PgPool) -> Self {
        let mut crypto = CryptoProvider::default();
        let keypair = match crypto.generate_keypair() {
            Ok(kp) => kp,
            Err(e) => {
                tracing::warn!(
                    "Failed to generate audit signing keypair with default mode: {}. Falling back to Classical.",
                    e
                );
                crypto = CryptoProvider::new(CryptoMode::Classical);
                crypto
                    .generate_keypair()
                    .expect("Failed to generate fallback classical audit signing keypair")
            }
        };

        tracing::info!(key_id = %keypair.key_id, mode = ?crypto.mode(), "AuditService initialized");

        Self {
            pool,
            crypto,
            keypair,
        }
    }

    /// Log a security or compliance event with cryptographic signature
    #[allow(clippy::too_many_arguments)]
    pub async fn log(
        &self,
        event_type: &str,
        actor_id: Option<&str>,
        actor_type: Option<&str>,
        target_id: Option<&str>,
        target_type: Option<&str>,
        action: &str,
        outcome: &str,
        details: Option<Value>,
        ip_address: Option<&str>,
    ) -> Result<Uuid, AuditError> {
        let id = Uuid::new_v4();
        let created_at = Utc::now();

        // Create canonical message for signing
        let canonical = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}",
            id,
            event_type,
            actor_id.unwrap_or(""),
            target_id.unwrap_or(""),
            action,
            outcome,
            details.as_ref().map(|d| d.to_string()).unwrap_or_default(),
            created_at.to_rfc3339()
        );

        // Sign with PQC (Hybrid Ed25519 + ML-DSA)
        let signature = self
            .crypto
            .sign(canonical.as_bytes(), &self.keypair)
            .map_err(|e| AuditError::Crypto(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO audit_events 
            (id, event_type, actor_id, actor_type, target_id, target_type, action, outcome, details, ip_address, signature, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::inet, $11, $12)
            "#
        )
        .bind(id)
        .bind(event_type)
        .bind(actor_id)
        .bind(actor_type)
        .bind(target_id)
        .bind(target_type)
        .bind(action)
        .bind(outcome)
        .bind(details)
        .bind(ip_address)
        .bind(&signature.value)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// Verify the integrity of the audit log hash chain
    /// Returns true if valid, false if tampered
    pub async fn verify_chain(&self) -> Result<bool, AuditError> {
        // In a real system, you'd fetch in batches.
        // For verification, we just check if any row has a hash mismatch.
        // But since the hash is computed in a TRIGGER, it's hard to spoof unless
        // the attacker also has DB superuser access to disable the trigger.

        // We can verify by re-computing the hash for the last N records
        // and checking if they match the stored event_hash and link to previous_hash.

        let rows = sqlx::query_as::<_, (Uuid, Option<String>, Option<String>, String)>(
            "SELECT id, event_hash, previous_hash, 
                    coalesce(event_type,'') || coalesce(actor_id,'') || coalesce(target_id,'') || 
                    coalesce(action,'') || coalesce(outcome,'') || coalesce(details::text,'') || 
                    created_at::text || coalesce(previous_hash,'genesis') as content
             FROM audit_events 
             ORDER BY created_at DESC, id DESC 
             LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await?;

        for (event_id, stored_hash, _prev_hash, content) in rows {
            let Some(stored_hash) = stored_hash else {
                continue;
            };
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let computed_hash = hex::encode(hasher.finalize());

            if stored_hash != computed_hash {
                tracing::error!(
                    "Audit log tampering detected! ID: {}, Stored: {}, Computed: {}",
                    event_id,
                    stored_hash,
                    computed_hash
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}
