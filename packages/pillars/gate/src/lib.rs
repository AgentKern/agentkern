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

pub mod api;
pub mod dsl;
pub mod engine;
pub mod neural;
pub mod policy;
pub mod types;

// Hyper-Stack modules (per ARCHITECTURE.md)
pub mod metrics; // Production Prometheus metrics
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
pub mod mtls; // Zero-Trust mTLS (Section 5)

#[cfg(feature = "compliance")]
pub use agentkern_governance::industry::finance::pci;
#[cfg(feature = "compliance")]
pub use agentkern_governance::industry::finance::shariah as shariah_compliance;
#[cfg(feature = "compliance")]
pub use agentkern_governance::industry::healthcare::fhir;
#[cfg(feature = "compliance")]
pub use agentkern_governance::industry::healthcare::hipaa;
#[cfg(feature = "compliance")]
pub use agentkern_governance::industry::{finance, healthcare};
pub use agentkern_governance::privacy::global_registry as global_privacy;

// MANDATE.md Section 6: Prompt Defense
#[cfg(feature = "esg")]
pub mod carbon;
pub mod context_guard;
pub mod prompt_guard; // Prompt injection detection // Energy-Aware Veto (ESG) // RAG memory injection protection

// Roadmap modules
pub mod explain; // Explainability Engine
pub mod feature_flags; // Privacy-first feature toggles

// Phase 2: Legacy Bridge Connectors
// Phase 2: Legacy Bridge Connectors
pub mod cache;
pub mod connectors; // Legacy system connectors (SAP, SWIFT, SQL)
pub mod provenance; // Neural Integrity (Phase 10) // Distributed Cache (Phase 20)

#[cfg(feature = "wasm")]
pub mod wasm; // WASM Component Model

// Re-exports
pub use actors::{GateSupervisor, PolicyResult, SupervisorStatus};
pub use budget::{AgentBudget, BudgetConfig, BudgetError};
pub use cache::{CacheLayer, RateLimiter};
#[cfg(feature = "esg")]
pub use carbon::{CarbonCheckResult, CarbonVeto};
pub use connectors::{
    ConnectorConfig, ConnectorHealth, ConnectorProtocol, ConnectorRegistry, LegacyConnector,
    MockConnector, SqlConnector,
};
pub use crypto_agility::{Algorithm, CryptoMode, CryptoProvider};
pub use engine::GateEngine;
pub use explain::{ExplainContext, ExplainabilityEngine, Explanation, ExplanationMethod};
pub use global_privacy::{
    GlobalPrivacyRegistry, Jurisdiction, PrivacyCheckResult, PrivacyError, Regulation,
    TransferStatus,
};
#[cfg(feature = "compliance")]
pub use hipaa::{HipaaError, HipaaRole, HipaaValidator, PhiScanResult};
pub use mtls::{CertificateInfo, CertificateValidator, MtlsConfig};
pub use neural::{
    IntentClass, IntentResult, ModelConfig, NeuralError, NeuralGuard, NeuroSymbolicValidator,
};
#[cfg(feature = "compliance")]
pub use pci::{CardBrand, CardToken, PciError, PciValidator};
pub use policy::{Policy, PolicyAction, PolicyRule};
pub use provenance::{ModelProvenance, ProvenanceError};
pub use runtime::{HyperRuntime, TokioRuntime};
#[cfg(feature = "compliance")]
pub use shariah_compliance::{
    ComplianceResult, ShariahComplianceError, ShariahComplianceValidator,
};
pub use sovereign::{DataTransfer, SovereignController, TransferDecision};
pub use tee::Enclave;
pub use types::{DataRegion, VerificationRequest, VerificationResult};
