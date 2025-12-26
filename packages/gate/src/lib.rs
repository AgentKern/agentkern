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
//! - `crypto`: Quantum-safe cryptography
//! - `mtls`: Zero-trust mTLS
//! - `hipaa`: HIPAA healthcare compliance
//! - `pci`: PCI-DSS payment compliance

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

// EXECUTION_MANDATE.md modules
pub mod budget;            // Gas Limits & Budgets (Section 6)
pub mod crypto_agility;    // Quantum-Safe Crypto (Section 3)
pub mod takaful;           // Takaful Compliance (Section 2)
pub mod mtls;              // Zero-Trust mTLS (Section 5)
pub mod hipaa;             // HIPAA Healthcare Compliance (Section 2)
pub mod pci;               // PCI-DSS Payment Compliance (Section 2)
pub mod fhir;              // FHIR R4 Healthcare Integration (Section 2)

// MANDATE.md Section 6: Prompt Defense
pub mod prompt_guard;      // Prompt injection detection

// Roadmap modules
pub mod explain;           // Explainability Engine

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
pub use budget::{AgentBudget, BudgetConfig, BudgetError};
pub use crypto_agility::{CryptoProvider, CryptoMode, Algorithm};
pub use takaful::{TakafulValidator, TakafulError, ComplianceResult};
pub use mtls::{CertificateValidator, MtlsConfig, CertificateInfo};
pub use hipaa::{HipaaValidator, HipaaError, PhiScanResult, HipaaRole};
pub use pci::{PciValidator, PciError, CardToken, CardBrand};
pub use explain::{ExplainabilityEngine, Explanation, ExplainContext, ExplanationMethod};



