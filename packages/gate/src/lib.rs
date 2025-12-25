//! VeriMantle-Gate: Neuro-Symbolic Verification Engine
//!
//! Per ARCHITECTURE.md: "The Core (Rust/Hyper-Loop)"
//! Per ENGINEERING_STANDARD.md: "Bio-Digital Pragmatism"
//!
//! Features implemented:
//! - `io_uring`: Native Tokio io_uring for zero-copy I/O
//! - `wasm`: WASM Component Model for policy nano-isolation
//! - `neural`: ONNX Runtime for neuro-symbolic guards
//! - `actors`: Actix for dynamic supervision with hot-swap
//! - `sovereign`: Data sovereignty and geo-fencing

pub mod policy;
pub mod dsl;
pub mod neural;
pub mod engine;
pub mod types;

// Hyper-Stack modules (per ARCHITECTURE.md)
pub mod runtime;           // Native Tokio io_uring runtime
pub mod tee;               // Hardware Enclaves (TDX/SEV)
pub mod observability;     // eBPF-compatible tracing

// ENGINEERING_STANDARD.md modules
pub mod actors;            // Dynamic Supervision (Section 1)

// GLOBAL_GAPS.md modules
pub mod sovereign;         // Data Sovereignty & Geo-Fencing (Section 1)

#[cfg(feature = "wasm")]
pub mod wasm;              // WASM Component Model

// Re-exports
pub use engine::GateEngine;
pub use policy::{Policy, PolicyRule, PolicyAction};
pub use types::{VerificationRequest, VerificationResult, DataRegion};
pub use runtime::{HyperRuntime, TokioRuntime};
pub use tee::Enclave;
pub use observability::{ObservabilityPlane, GateMetrics};
pub use actors::{GateSupervisor, PolicyResult, SupervisorStatus};
pub use sovereign::{SovereignController, DataTransfer, TransferDecision};

