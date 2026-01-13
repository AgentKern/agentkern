//! PostgreSQL-backed Transfer Engine
//!
//! Production-grade implementation that replaces in-memory state
//! with database transactions for high availability.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{AgentId, Amount, TransactionId};

/// Transfer request
#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub from: AgentId,
    pub to: AgentId,
    pub amount: Amount,
    pub reference: Option<String>,
    pub idempotency_key: Option<String>,
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

/// Transfer result
#[derive(Debug, Clone)]
pub struct TransferResult {
    pub transaction_id: TransactionId,
    pub status: TransferStatus,
    pub timestamp: DateTime<Utc>,
    pub error: Option<String>,
}

impl TransferResult {
    fn success(transaction_id: TransactionId) -> Self {
        Self {
            transaction_id,
            status: TransferStatus::Completed,
            timestamp: Utc::now(),
            error: None,
        }
    }

    fn failed(transaction_id: TransactionId, error: impl Into<String>) -> Self {
        Self {
            transaction_id,
            status: TransferStatus::Failed,
            timestamp: Utc::now(),
            error: Some(error.into()),
        }
    }
}

/// Transfer errors
#[derive(Debug, thiserror::Error)]
pub enum TransferError {
    #[error("Transfer not found")]
    NotFound,
    #[error("Insufficient funds: available {available}, requested {requested}")]
    InsufficientFunds { available: i64, requested: i64 },
    #[error("Cannot transfer to self")]
    SelfTransfer,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// PostgreSQL-backed transfer engine with proper distributed locking
pub struct PgTransferEngine {
    pool: PgPool,
}

impl PgTransferEngine {
    /// Create a new transfer engine
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Execute an atomic transfer using database transactions
    pub async fn transfer(&self, request: TransferRequest) -> TransferResult {
        let transaction_id = Uuid::new_v4();

        // Validate request
        if request.from == request.to {
            return TransferResult::failed(transaction_id, "Cannot transfer to self");
        }
        if request.amount.is_zero() || request.amount.is_negative() {
            return TransferResult::failed(transaction_id, "Invalid amount");
        }

        // Check idempotency
        if let Some(ref key) = request.idempotency_key {
            match self.check_idempotency(key).await {
                Ok(Some(existing_id)) => return TransferResult::success(existing_id),
                Err(e) => return TransferResult::failed(transaction_id, e.to_string()),
                _ => {}
            }
        }

        // Execute transfer in transaction with row-level locking
        match self.execute_transfer(&request, transaction_id).await {
            Ok(()) => {
                tracing::info!(
                    transaction_id = %transaction_id,
                    from = %request.from,
                    to = %request.to,
                    amount = %request.amount,
                    "Transfer completed"
                );
                TransferResult::success(transaction_id)
            }
            Err(e) => {
                tracing::warn!(
                    transaction_id = %transaction_id,
                    error = %e,
                    "Transfer failed"
                );
                TransferResult::failed(transaction_id, e.to_string())
            }
        }
    }

    /// Check if this idempotency key was already used
    async fn check_idempotency(&self, key: &str) -> Result<Option<TransactionId>, TransferError> {
        let result = sqlx::query_scalar::<_, Uuid>(
            "SELECT transaction_id FROM completed_transfers WHERE idempotency_key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Execute the transfer with proper locking
    async fn execute_transfer(
        &self,
        request: &TransferRequest,
        transaction_id: TransactionId,
    ) -> Result<(), TransferError> {
        let amount_micros = request.amount.as_micros();

        // Start transaction
        let mut tx = self.pool.begin().await?;

        // Lock sender's balance row (SELECT FOR UPDATE)
        let sender_balance = sqlx::query_as::<_, (i64, i64)>(
            "SELECT balance, held FROM agent_balances WHERE agent_id = $1 FOR UPDATE",
        )
        .bind(&request.from)
        .fetch_optional(&mut *tx)
        .await?
        .unwrap_or((0, 0));

        let available = sender_balance.0 - sender_balance.1;
        if available < amount_micros {
            return Err(TransferError::InsufficientFunds {
                available,
                requested: amount_micros,
            });
        }

        // Ensure receiver exists (create if not)
        sqlx::query(
            "INSERT INTO agent_balances (agent_id, balance, currency) 
             VALUES ($1, 0, 'VMC') 
             ON CONFLICT (agent_id) DO NOTHING",
        )
        .bind(&request.to)
        .execute(&mut *tx)
        .await?;

        // Deduct from sender
        sqlx::query(
            "UPDATE agent_balances SET balance = balance - $1, updated_at = NOW() WHERE agent_id = $2"
        )
        .bind(amount_micros)
        .bind(&request.from)
        .execute(&mut *tx)
        .await?;

        // Credit to receiver
        sqlx::query(
            "UPDATE agent_balances SET balance = balance + $1, updated_at = NOW() WHERE agent_id = $2"
        )
        .bind(amount_micros)
        .bind(&request.to)
        .execute(&mut *tx)
        .await?;

        // Record completed transfer
        sqlx::query(
            "INSERT INTO completed_transfers 
             (transaction_id, from_agent, to_agent, amount, reference, idempotency_key, completed_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW())"
        )
        .bind(transaction_id)
        .bind(&request.from)
        .bind(&request.to)
        .bind(amount_micros)
        .bind(&request.reference)
        .bind(&request.idempotency_key)
        .execute(&mut *tx)
        .await?;

        // Commit transaction
        tx.commit().await?;

        Ok(())
    }

    /// Get balance for an agent
    pub async fn get_balance(&self, agent_id: &str) -> Result<i64, TransferError> {
        let balance =
            sqlx::query_scalar::<_, i64>("SELECT balance FROM agent_balances WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_optional(&self.pool)
                .await?
                .unwrap_or(0);

        Ok(balance)
    }

    /// Deposit funds to an agent (for testing/initial funding)
    pub async fn deposit(&self, agent_id: &str, amount: i64) -> Result<(), TransferError> {
        sqlx::query(
            "INSERT INTO agent_balances (agent_id, balance, currency)
             VALUES ($1, $2, 'VMC')
             ON CONFLICT (agent_id) DO UPDATE SET balance = agent_balances.balance + $2, updated_at = NOW()"
        )
        .bind(agent_id)
        .bind(amount)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Cleanup expired pending transfers
    pub async fn cleanup_expired(&self) -> Result<u64, TransferError> {
        let result = sqlx::query(
            "DELETE FROM pending_transfers WHERE expires_at < NOW() AND status = 'pending'",
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // Tests require a database - run with:
    // DATABASE_URL=postgres://... cargo test -p agentkern-treasury
}
