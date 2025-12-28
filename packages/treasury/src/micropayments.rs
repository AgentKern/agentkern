//! Micropayment Aggregator
//!
//! Per Market Research: Traditional payments fail at sub-cent transactions.
//! This module aggregates micropayments for efficient batch settlement.

use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::{AgentId, Amount, TransactionId};

/// Pending micropayment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPayment {
    /// Unique ID
    pub id: TransactionId,
    /// Sender
    pub from: AgentId,
    /// Receiver
    pub to: AgentId,
    /// Amount
    pub amount: Amount,
    /// Reference
    pub reference: Option<String>,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Aggregated payment batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentBatch {
    /// Batch ID
    pub id: TransactionId,
    /// Payments in this batch
    pub payments: Vec<PendingPayment>,
    /// Total amount
    pub total: Amount,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Aggregator configuration.
#[derive(Debug, Clone)]
pub struct AggregatorConfig {
    /// Minimum amount for immediate settlement
    pub immediate_threshold: Amount,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Maximum batch age before forced settlement
    pub max_batch_age: Duration,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            immediate_threshold: Amount::from_float(1.0, 6), // $1 threshold
            max_batch_size: 100,
            max_batch_age: Duration::minutes(5),
        }
    }
}

/// Micropayment aggregator.
pub struct MicropaymentAggregator {
    config: AggregatorConfig,
    /// Pending payments by receiver
    pending: Arc<RwLock<HashMap<AgentId, Vec<PendingPayment>>>>,
    /// Pending batches ready for settlement
    ready_batches: Arc<RwLock<Vec<PaymentBatch>>>,
}

impl Default for MicropaymentAggregator {
    fn default() -> Self {
        Self::new(AggregatorConfig::default())
    }
}

impl MicropaymentAggregator {
    /// Create a new aggregator.
    pub fn new(config: AggregatorConfig) -> Self {
        Self {
            config,
            pending: Arc::new(RwLock::new(HashMap::new())),
            ready_batches: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a micropayment.
    /// Returns Some(batch) if a batch is ready for settlement.
    pub fn add_payment(&self, payment: PendingPayment) -> Option<PaymentBatch> {
        // If above threshold, return for immediate settlement
        if payment.amount.value >= self.config.immediate_threshold.value {
            return Some(PaymentBatch {
                id: Uuid::new_v4(),
                payments: vec![payment.clone()],
                total: payment.amount,
                created_at: Utc::now(),
            });
        }

        let mut pending = self.pending.write();
        let receiver_payments = pending.entry(payment.to.clone()).or_default();
        receiver_payments.push(payment.clone());

        // Check if we should batch
        let total: i64 = receiver_payments.iter().map(|p| p.amount.value).sum();
        let oldest = receiver_payments
            .first()
            .map(|p| p.created_at)
            .unwrap_or(Utc::now());
        let age = Utc::now() - oldest;

        let should_batch = total >= self.config.immediate_threshold.value
            || receiver_payments.len() >= self.config.max_batch_size
            || age >= self.config.max_batch_age;

        if should_batch {
            let payments = std::mem::take(receiver_payments);
            let batch = PaymentBatch {
                id: Uuid::new_v4(),
                payments: payments.clone(),
                total: Amount::new(total, payment.amount.decimals),
                created_at: Utc::now(),
            };

            // Store ready batch
            let mut ready = self.ready_batches.write();
            ready.push(batch.clone());

            return Some(batch);
        }

        None
    }

    /// Get pending payment count for a receiver.
    pub fn pending_count(&self, receiver: &str) -> usize {
        let pending = self.pending.read();
        pending.get(receiver).map(|v| v.len()).unwrap_or(0)
    }

    /// Get total pending amount for a receiver.
    pub fn pending_amount(&self, receiver: &str, decimals: u8) -> Amount {
        let pending = self.pending.read();
        let total: i64 = pending
            .get(receiver)
            .map(|v| v.iter().map(|p| p.amount.value).sum())
            .unwrap_or(0);
        Amount::new(total, decimals)
    }

    /// Get ready batches for settlement.
    pub fn get_ready_batches(&self) -> Vec<PaymentBatch> {
        let ready = self.ready_batches.read();
        ready.clone()
    }

    /// Mark a batch as settled.
    pub fn mark_settled(&self, batch_id: TransactionId) {
        let mut ready = self.ready_batches.write();
        ready.retain(|b| b.id != batch_id);
    }

    /// Force batch creation for all pending payments (for shutdown/cleanup).
    pub fn flush_all(&self) -> Vec<PaymentBatch> {
        let mut pending = self.pending.write();
        let mut batches = Vec::new();

        for (receiver, payments) in pending.drain() {
            if payments.is_empty() {
                continue;
            }

            let total: i64 = payments.iter().map(|p| p.amount.value).sum();
            let decimals = payments.first().map(|p| p.amount.decimals).unwrap_or(6);

            batches.push(PaymentBatch {
                id: Uuid::new_v4(),
                payments,
                total: Amount::new(total, decimals),
                created_at: Utc::now(),
            });
        }

        batches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immediate_settlement() {
        let aggregator = MicropaymentAggregator::default();

        let payment = PendingPayment {
            id: Uuid::new_v4(),
            from: "agent-1".to_string(),
            to: "agent-2".to_string(),
            amount: Amount::from_float(5.0, 6), // Above threshold
            reference: None,
            created_at: Utc::now(),
        };

        let batch = aggregator.add_payment(payment);
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().payments.len(), 1);
    }

    #[test]
    fn test_aggregation() {
        let config = AggregatorConfig {
            immediate_threshold: Amount::from_float(1.0, 6),
            max_batch_size: 3,
            max_batch_age: Duration::minutes(5),
        };
        let aggregator = MicropaymentAggregator::new(config);

        // Add micropayments below threshold
        for i in 0..2 {
            let payment = PendingPayment {
                id: Uuid::new_v4(),
                from: format!("agent-{}", i),
                to: "receiver".to_string(),
                amount: Amount::from_float(0.1, 6),
                reference: None,
                created_at: Utc::now(),
            };
            let batch = aggregator.add_payment(payment);
            assert!(batch.is_none());
        }

        // Third payment should trigger batch
        let payment = PendingPayment {
            id: Uuid::new_v4(),
            from: "agent-2".to_string(),
            to: "receiver".to_string(),
            amount: Amount::from_float(0.1, 6),
            reference: None,
            created_at: Utc::now(),
        };
        let batch = aggregator.add_payment(payment);
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().payments.len(), 3);
    }

    #[test]
    fn test_flush_all() {
        let aggregator = MicropaymentAggregator::default();

        for i in 0..5 {
            let payment = PendingPayment {
                id: Uuid::new_v4(),
                from: format!("sender-{}", i),
                to: "receiver".to_string(),
                amount: Amount::from_float(0.01, 6),
                reference: None,
                created_at: Utc::now(),
            };
            aggregator.add_payment(payment);
        }

        let batches = aggregator.flush_all();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].payments.len(), 5);
    }
}
