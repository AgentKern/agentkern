//! Atomic Transfer Engine
//!
//! Per Market Research: 60% of multi-agent systems fail due to lack of atomic payments.
//! This module implements 2-phase commit for safe agent-to-agent transfers.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::balance::BalanceLedger;
use crate::types::{AgentId, Amount, TransactionId};

/// Transfer request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    /// Sender agent ID
    pub from: AgentId,
    /// Receiver agent ID
    pub to: AgentId,
    /// Amount to transfer
    pub amount: Amount,
    /// Reference/memo
    pub reference: Option<String>,
    /// Idempotency key (prevent duplicate transfers)
    pub idempotency_key: Option<String>,
}

impl TransferRequest {
    /// Create a new transfer request.
    pub fn new(from: impl Into<AgentId>, to: impl Into<AgentId>, amount: Amount) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            amount,
            reference: None,
            idempotency_key: None,
        }
    }

    /// Add a reference.
    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    /// Add an idempotency key.
    pub fn with_idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }
}

/// Transfer status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    /// Transfer initiated, funds held
    Pending,
    /// Transfer completed
    Completed,
    /// Transfer failed, funds released
    Failed,
    /// Transfer cancelled by sender
    Cancelled,
}

/// Transfer result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    /// Transaction ID
    pub transaction_id: TransactionId,
    /// Status
    pub status: TransferStatus,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Error message if failed
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

/// Pending transfer record.
#[derive(Debug, Clone)]
struct PendingTransfer {
    request: TransferRequest,
    transaction_id: TransactionId,
    created_at: DateTime<Utc>,
}

/// Transfer engine with 2-phase commit.
pub struct TransferEngine {
    ledger: Arc<BalanceLedger>,
    pending: Arc<RwLock<HashMap<TransactionId, PendingTransfer>>>,
    completed: Arc<RwLock<HashMap<String, TransactionId>>>, // idempotency cache
}

impl TransferEngine {
    /// Create a new transfer engine.
    pub fn new(ledger: Arc<BalanceLedger>) -> Self {
        Self {
            ledger,
            pending: Arc::new(RwLock::new(HashMap::new())),
            completed: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Execute an atomic transfer.
    pub async fn transfer(&self, request: TransferRequest) -> TransferResult {
        let transaction_id = Uuid::new_v4();

        // Check idempotency
        if let Some(ref key) = request.idempotency_key {
            let completed = self.completed.read();
            if let Some(&existing_id) = completed.get(key) {
                return TransferResult::success(existing_id);
            }
        }

        // Validate request
        if request.from == request.to {
            return TransferResult::failed(transaction_id, "Cannot transfer to self");
        }
        if request.amount.is_zero() || request.amount.is_negative() {
            return TransferResult::failed(transaction_id, "Invalid amount");
        }

        // Phase 1: Hold funds
        if let Err(e) = self.ledger.hold(&request.from, request.amount) {
            return TransferResult::failed(transaction_id, e.to_string());
        }

        // Store pending transfer
        {
            let mut pending = self.pending.write();
            pending.insert(
                transaction_id,
                PendingTransfer {
                    request: request.clone(),
                    transaction_id,
                    created_at: Utc::now(),
                },
            );
        }

        // Phase 2: Commit transfer
        match self
            .ledger
            .commit_transfer(&request.from, &request.to, request.amount)
        {
            Ok(()) => {
                // Remove from pending
                {
                    let mut pending = self.pending.write();
                    pending.remove(&transaction_id);
                }

                // Store in idempotency cache
                if let Some(key) = request.idempotency_key {
                    let mut completed = self.completed.write();
                    completed.insert(key, transaction_id);
                }

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
                // Rollback: release held funds
                let _ = self.ledger.release(&request.from, request.amount);

                // Remove from pending
                {
                    let mut pending = self.pending.write();
                    pending.remove(&transaction_id);
                }

                TransferResult::failed(transaction_id, e.to_string())
            }
        }
    }

    /// Cancel a pending transfer.
    pub async fn cancel(&self, transaction_id: TransactionId) -> Result<(), TransferError> {
        let pending_transfer = {
            let mut pending = self.pending.write();
            pending.remove(&transaction_id)
        };

        match pending_transfer {
            Some(pt) => {
                // Release held funds
                self.ledger
                    .release(&pt.request.from, pt.request.amount)
                    .map_err(|e| TransferError::LedgerError(e.to_string()))?;
                Ok(())
            }
            None => Err(TransferError::NotFound),
        }
    }

    /// Get pending transfers count.
    pub fn pending_count(&self) -> usize {
        self.pending.read().len()
    }

    /// Get details of pending transfers for monitoring/audit.
    /// Returns Vec of (transaction_id, created_at, from, to, amount).
    pub fn pending_transfers(
        &self,
    ) -> Vec<(TransactionId, DateTime<Utc>, AgentId, AgentId, Amount)> {
        self.pending
            .read()
            .values()
            .map(|pt| {
                (
                    pt.transaction_id,
                    pt.created_at,
                    pt.request.from.clone(),
                    pt.request.to.clone(),
                    pt.request.amount,
                )
            })
            .collect()
    }
}

/// Transfer errors.
#[derive(Debug, thiserror::Error)]
pub enum TransferError {
    #[error("Transfer not found")]
    NotFound,
    #[error("Ledger error: {0}")]
    LedgerError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::Currency;

    fn setup() -> TransferEngine {
        let ledger = Arc::new(BalanceLedger::new(Currency::VMC));
        // Pre-fund agent-1
        ledger
            .deposit("agent-1", Amount::from_float(1000.0, 6))
            .unwrap();
        TransferEngine::new(ledger)
    }

    #[tokio::test]
    async fn test_successful_transfer() {
        let engine = setup();

        let request = TransferRequest::new("agent-1", "agent-2", Amount::from_float(100.0, 6));
        let result = engine.transfer(request).await;

        assert_eq!(result.status, TransferStatus::Completed);
    }

    #[tokio::test]
    async fn test_insufficient_funds() {
        let engine = setup();

        let request = TransferRequest::new("agent-1", "agent-2", Amount::from_float(2000.0, 6));
        let result = engine.transfer(request).await;

        assert_eq!(result.status, TransferStatus::Failed);
        assert!(result.error.unwrap().contains("Insufficient"));
    }

    #[tokio::test]
    async fn test_idempotency() {
        let engine = setup();

        let request = TransferRequest::new("agent-1", "agent-2", Amount::from_float(50.0, 6))
            .with_idempotency_key("test-key-1");

        let result1 = engine.transfer(request.clone()).await;
        let result2 = engine.transfer(request).await;

        // Same transaction ID returned
        assert_eq!(result1.transaction_id, result2.transaction_id);
    }

    #[tokio::test]
    async fn test_self_transfer() {
        let engine = setup();

        let request = TransferRequest::new("agent-1", "agent-1", Amount::from_float(50.0, 6));
        let result = engine.transfer(request).await;

        assert_eq!(result.status, TransferStatus::Failed);
    }
}
