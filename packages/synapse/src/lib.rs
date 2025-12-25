//! VeriMantle-Synapse: Graph-based State Ledger
//!
//! The "Memory" for autonomous AI agents.
//!
//! Per ARCHITECTURE.md:
//! - Uses CRDTs for eventual consistency
//! - Tracks "Intent Paths" to prevent goal drift
//! - Stores agent state as a graph, not just key-value
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    VeriMantle-Synapse                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────┐    ┌─────────┐    ┌─────────┐                 │
//! │  │ Agent A │───►│ Intent  │───►│ State   │                 │
//! │  └─────────┘    │ Path    │    │ Graph   │                 │
//! │                 └─────────┘    └─────────┘                 │
//! │                      │              │                       │
//! │                      ▼              ▼                       │
//! │                 Drift            CRDT                       │
//! │                 Detector         Merge                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod state;
pub mod intent;
pub mod drift;
pub mod types;

// Re-exports
pub use state::StateStore;
pub use intent::{IntentPath, IntentStep};
pub use drift::DriftDetector;
pub use types::{AgentState, StateQuery, StateUpdate};
