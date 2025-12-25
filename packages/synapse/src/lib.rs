//! VeriMantle-Synapse: Graph-based State Ledger with CRDTs
//!
//! Per ARCHITECTURE.md Section 3: "The Speed of Light"
//! Per ENGINEERING_STANDARD.md Section 2: "Adaptive Execution"
//!
//! Features implemented:
//! - **CRDTs**: Conflict-free Replicated Data Types for eventual consistency
//! - **Graph Vector DB**: State stored as graph with vector embeddings
//! - **Adaptive Query**: Arrow/Polars for profile-guided optimization
//! - **Intent Tracking**: Monitor goal progression and detect drift
//! - **Polyglot Embeddings**: Region-specific embedding models
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    VeriMantle-Synapse                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │           Graph Vector Database                      │   │
//! │  │  ┌────────┐    ┌────────┐    ┌────────┐            │   │
//! │  │  │ Agent  │───►│ Intent │───►│ State  │            │   │
//! │  │  │ Node   │    │ Node   │    │ Node   │            │   │
//! │  │  └────────┘    └────────┘    └────────┘            │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! │                          │                                  │
//! │        ┌─────────────────┴─────────────────┐               │
//! │        │      Adaptive Query Executor      │               │
//! │        │  Standard ←→ Vectorized ←→ Stream │               │
//! │        └───────────────────────────────────┘               │
//! │                          │                                  │
//! │                    CRDT Replication                         │
//! │              (US ← → EU ← → Asia ← → Africa)                │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod state;
pub mod intent;
pub mod drift;
pub mod types;
pub mod graph;       // Graph Vector Database
pub mod adaptive;    // Adaptive Query Execution (ENGINEERING_STANDARD Section 2)

// GLOBAL_GAPS.md modules
pub mod embeddings;  // Polyglot Embeddings (Section 2)

// Re-exports
pub use state::StateStore;
pub use intent::{IntentPath, IntentStep};
pub use drift::DriftDetector;
pub use types::{AgentState, StateQuery, StateUpdate};
pub use graph::{GraphVectorDB, GraphNode, GraphEdge, NodeType, EdgeType};
pub use adaptive::{AdaptiveExecutor, ExecutionStrategy, ExecutionMetrics};
pub use embeddings::{EmbeddingConfig, EmbeddingProvider, PolyglotEmbedder, SynapseRegion};

