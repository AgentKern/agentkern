//! AgentKern-Treasury: Agent Payment Infrastructure
//!
//! Per MANIFESTO.md: "Agents can pay each other for services via micropayment rails"
//! Per Market Research: No one has solved agent-to-agent payments properly
//!
//! This is AgentKern's "Blue Ocean" - the 5th Pillar.
//!
//! Features:
//! - Agent balance management
//! - Atomic transfers with 2-phase commit
//! - Spending budgets and limits
//! - Micropayment aggregation
//! - Transaction history and audit
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_treasury::{Treasury, TransferRequest};
//!
//! let treasury = Treasury::new();
//!
//! // Agent A pays Agent B for a service
//! let result = treasury.transfer(TransferRequest {
//!     from: "agent-a",
//!     to: "agent-b",
//!     amount: 0.001, // $0.001 micropayment
//!     reference: "api-call-12345",
//! }).await?;
//! ```
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    AgentKern-Treasury                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │              Balance Ledger                         │   │
//! │  │  Agent A: $100.00   Agent B: $50.00   Agent C: $0   │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! │                          │                                  │
//! │        ┌─────────────────┴─────────────────┐               │
//! │        │      Atomic Transfer Engine       │               │
//! │        │  (2-Phase Commit for Safety)      │               │
//! │        └───────────────────────────────────┘               │
//! │                          │                                  │
//! │  ┌───────────────┬───────────────┬───────────────┐        │
//! │  │ Budget Manager│ Micropayments │ Audit Ledger  │        │
//! │  │ (Limits)      │ (Aggregation) │ (Compliance)  │        │
//! │  └───────────────┴───────────────┴───────────────┘        │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod balance;
pub mod transfer;
pub mod budget;
pub mod micropayments;
pub mod types;
pub mod carbon;  // Innovation #8: Carbon Footprint Ledger
pub mod lock;    // Per Code Quality Audit: Distributed locking

// Re-exports
pub use balance::{BalanceLedger, AgentBalance, Currency};
pub use transfer::{TransferEngine, TransferRequest, TransferResult, TransferStatus};
pub use budget::{BudgetManager, SpendingLimit, BudgetPeriod};
pub use micropayments::{MicropaymentAggregator, PendingPayment};
pub use types::{Amount, TransactionId, AgentId};
pub use carbon::{CarbonLedger, CarbonFootprint, CarbonBudget, CarbonRegion, ComputeType, CarbonUsage};
pub use lock::{LockManager, LockGuard, LockConfig, LockMode, LockError};
