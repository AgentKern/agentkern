#![warn(unused)] // Production: warn on unused code
#![warn(dead_code)] // Production: warn on dead code
//! AgentKern-Treasury: Agent Payment Infrastructure
//!
//! Per MANIFESTO.md: "Agents can pay each other for services via micropayment rails"

pub mod api;
pub mod balance;
pub mod budget;
pub mod db; // PostgreSQL-backed distributed state
pub mod lock;
pub mod micropayments;
pub mod transfer;
pub mod types;
pub mod verification;

// Re-exports
pub use balance::{AgentBalance, BalanceLedger, Currency};
pub use budget::{BudgetManager, BudgetPeriod, SpendingLimit};
pub use lock::{LockConfig, LockError, LockGuard, LockManager, LockMode};
pub use micropayments::{MicropaymentAggregator, PendingPayment};
pub use transfer::{TransferEngine, TransferRequest, TransferResult, TransferStatus};
pub use types::{AgentId, Amount, TransactionId};
