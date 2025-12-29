#![allow(unused)]
#![allow(dead_code)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::type_complexity)]
#![allow(clippy::unwrap_or_default)]
//! AgentKern-Synapse: Graph-based State Ledger with CRDTs
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
//! │                    AgentKern-Synapse                       │
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

pub mod adaptive;
pub mod drift;
pub mod graph; // Graph Vector Database
pub mod intent;
pub mod state;
pub mod types; // Adaptive Query Execution (ENGINEERING_STANDARD Section 2)

// GLOBAL_GAPS.md modules
pub mod embeddings; // Polyglot Embeddings (Section 2)
pub mod mesh; // Global Mesh Sync (Section 1)
pub mod polyglot; // Native Language Support

// COMPETITIVE_LANDSCAPE.md modules
pub mod crdt; // Conflict-Free Replicated Data Types

// Re-exports
pub use adaptive::{AdaptiveExecutor, ExecutionMetrics, ExecutionStrategy};
pub use crdt::{AgentStateCrdt, GCounter, LwwMap, LwwRegister, OrSet, PNCounter};
pub use drift::DriftDetector;
pub use embeddings::{EmbeddingConfig, EmbeddingProvider, PolyglotEmbedder, SynapseRegion};
pub use graph::{EdgeType, GraphEdge, GraphNode, GraphVectorDB, NodeType};
pub use intent::{IntentPath, IntentStep};
pub use mesh::{DataRegion, GeoFence, GlobalMesh, MeshCell, MeshSync};
pub use polyglot::{Language, PolyglotMemory};
pub use state::StateStore;
pub use types::{AgentState, StateQuery, StateUpdate};

// NOTE: Antifragile moved to agentkern-arbiter during consolidation
// See: packages/arbiter/src/antifragile.rs

// Innovation #10: Digital Twin Sandbox
pub mod sandbox;
pub use sandbox::{
    ChaosEvent, EnvironmentSnapshot, Sandbox, SandboxEngine, SandboxMode, TestResult, TestScenario,
};

// Phase 2: Memory Passport for Sovereign Identity
pub mod passport;
pub use passport::{
    GdprExport, MemoryLayers, MemoryPassport, PassportError, PassportExporter, PassportImporter,
    PassportVersion,
};

// AI Security: RAG Context Guard (per AI-Native Audit 2026)
pub mod context_guard;
pub use context_guard::{
    ContextAnalysisResult, ContextGuard, ContextGuardConfig, DetectedThreat, ThreatType,
};
