//! VeriMantle-Gate: Neuro-Symbolic Verification Engine
//!
//! The "Guardrails" for autonomous AI agents.
//!
//! Per ENGINEERING_STANDARD.md:
//! - **Fast Path (Symbolic)**: Deterministic policy checks (<1ms)
//! - **Safety Path (Neural)**: Semantic malice scoring (<20ms)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    VeriMantle-Gate                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Request ──► Policy Parser ──► Symbolic Engine ──► Result   │
//! │                    │                  │                      │
//! │                    ▼                  ▼                      │
//! │              YAML/DSL             Fast Path                  │
//! │                                      │                       │
//! │                              (if risk > threshold)           │
//! │                                      ▼                       │
//! │                              Neural Scorer                   │
//! │                              (ONNX Model)                    │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod policy;
pub mod engine;
pub mod dsl;
pub mod neural;
pub mod types;

// Re-exports
pub use policy::{Policy, PolicyRule, PolicyAction};
pub use engine::GateEngine;
pub use types::{VerificationRequest, VerificationResult};
