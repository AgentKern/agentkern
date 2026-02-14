#![warn(unused)] // Production: warn on unused code
#![warn(dead_code)] // Production: warn on dead code
#![allow(clippy::collapsible_if)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::type_complexity)]
#![allow(clippy::unwrap_or_default)]
//! AgentKern-Synapse: Graph-based State Ledger with CRDTs
//!
//! Per ARCHITECTURE.md Section 3: "The Speed of Light"
//! Per ENGINEERING_STANDARD.md Section 2: "Adaptive Execution"

pub mod adaptive;
pub mod api;
pub mod drift;
pub mod graph; // Graph Vector Database
pub mod intent;
pub mod state;
pub mod types;

// GLOBAL_GAPS.md modules
pub mod embeddings; // Polyglot Embeddings (Section 2)
pub mod mesh; // Global Mesh Sync (Section 1)
pub mod polyglot; // Native Language Support

// COMPETITIVE_LANDSCAPE.md modules
pub mod crdt; // Conflict-Free Replicated Data Types
pub mod state_snapshot; // Chain-anchored immutable state backups

// Re-exports
pub use adaptive::{AdaptiveExecutor, ExecutionMetrics, ExecutionStrategy};
pub use crdt::{AgentStateCrdt, GCounter, LwwMap, LwwRegister, OrSet, PNCounter};
pub use drift::DriftDetector;
pub use embeddings::{EmbeddingConfig, EmbeddingProvider, PolyglotEmbedder, SynapseRegion};
pub use graph::{EdgeType, GraphEdge, GraphNode, GraphVectorDB, NodeType};
pub use intent::{IntentPath, IntentStep};
pub use mesh::{DataRegion, GlobalMesh, MeshCell, MeshOrchestrator, MeshSync, MigrationReason};
pub use polyglot::{Language, PolyglotMemory};
pub use state::StateStore;
pub use types::{AgentState, StateQuery, StateUpdate};

// AI Security: RAG Context Guard (per AI-Native Audit 2026)
pub mod context_guard;
pub use context_guard::{
    ContextAnalysisResult, ContextGuard, ContextGuardConfig, DetectedThreat, ThreatType,
};

// Phase 2: Memory Passport
pub mod passport;
pub use passport::{
    GdprExport, MemoryLayers, MemoryPassport, PassportError, PassportExporter, PassportImporter,
    PassportVersion,
};
