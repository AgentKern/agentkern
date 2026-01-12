use sqlx::PgPool;
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;
use chrono::Utc;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Service for logging compliance and security events
pub struct AuditService {
    pool: PgPool,
}

impl AuditService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Log a security or compliance event
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
        
        sqlx::query(
            r#"
            INSERT INTO audit_events 
            (id, event_type, actor_id, actor_type, target_id, target_type, action, outcome, details, ip_address, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::inet, $11)
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
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(id)
    }
}
