#![allow(unused)]
#![allow(dead_code)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::manual_range_contains)]
//! AgentKern-Gate: Neuro-Symbolic Verification Engine
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

pub mod dsl;
pub mod engine;
pub mod neural;
pub mod policy;
pub mod types;

// Hyper-Stack modules (per ARCHITECTURE.md)
pub mod observability;
pub mod runtime; // Native Tokio io_uring runtime
pub mod tee; // Hardware Enclaves (TDX/SEV) // eBPF-compatible tracing

// ENGINEERING_STANDARD.md modules
pub mod actors; // Dynamic Supervision (Section 1)

// GLOBAL_GAPS.md modules
pub mod sovereign; // Data Sovereignty & Geo-Fencing (Section 1)

// EXECUTION_MANDATE.md modules
pub mod budget; // Gas Limits & Budgets (Section 6)
pub mod crypto_agility; // Quantum-Safe Crypto (Section 3)
pub mod fhir;
pub mod hipaa; // HIPAA Healthcare Compliance (Section 2)
pub mod mtls; // Zero-Trust mTLS (Section 5)
pub mod pci; // PCI-DSS Payment Compliance (Section 2)
pub mod takaful; // Takaful Compliance (Section 2) // FHIR R4 Healthcare Integration (Section 2)

// MANDATE.md Section 6: Prompt Defense
pub mod carbon;
pub mod prompt_guard; // Prompt injection detection // Energy-Aware Veto (ESG)

// Roadmap modules
pub mod explain; // Explainability Engine

// Phase 2: Legacy Bridge Connectors
pub mod connectors; // Legacy system connectors (SAP, SWIFT, SQL)

#[cfg(feature = "wasm")]
pub mod wasm; // WASM Component Model

// Re-exports
pub use actors::{GateSupervisor, PolicyResult, SupervisorStatus};
pub use budget::{AgentBudget, BudgetConfig, BudgetError};
pub use carbon::{CarbonCheckResult, CarbonVeto};
pub use connectors::{
    ConnectorConfig, ConnectorHealth, ConnectorProtocol, ConnectorRegistry, LegacyConnector,
    MockConnector, SqlConnector,
};
pub use crypto_agility::{Algorithm, CryptoMode, CryptoProvider};
pub use engine::GateEngine;
pub use explain::{ExplainContext, ExplainabilityEngine, Explanation, ExplanationMethod};
pub use hipaa::{HipaaError, HipaaRole, HipaaValidator, PhiScanResult};
pub use mtls::{CertificateInfo, CertificateValidator, MtlsConfig};
pub use observability::{GateMetrics, ObservabilityPlane};
pub use pci::{CardBrand, CardToken, PciError, PciValidator};
pub use policy::{Policy, PolicyAction, PolicyRule};
pub use runtime::{HyperRuntime, TokioRuntime};
pub use sovereign::{DataTransfer, SovereignController, TransferDecision};
pub use takaful::{ComplianceResult, TakafulError, TakafulValidator};
pub use tee::Enclave;
pub use types::{DataRegion, VerificationRequest, VerificationResult};
