//! VeriMantle-Arbiter: Conflict Resolution & Coordination Engine
//!
//! The "Traffic Control" for autonomous AI agents.
//!
//! Per ARCHITECTURE.md:
//! - Implements Atomic Business Locks
//! - Uses Raft for distributed consensus
//! - Priority-based conflict resolution
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    VeriMantle-Arbiter                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────┐    ┌─────────┐    ┌─────────┐                 │
//! │  │ Request │───►│ Lock    │───►│ Queue   │                 │
//! │  │ Handler │    │ Manager │    │ Manager │                 │
//! │  └─────────┘    └─────────┘    └─────────┘                 │
//! │                      │              │                       │
//! │                      ▼              ▼                       │
//! │                 Priority        Fairness                    │
//! │                 Scheduler       Enforcer                    │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod locks;
pub mod queue;
pub mod coordinator;
pub mod types;

// Re-exports
pub use locks::LockManager;
pub use queue::PriorityQueue;
pub use coordinator::Coordinator;
pub use types::{BusinessLock, CoordinationRequest, CoordinationResult};
