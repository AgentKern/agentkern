#![allow(unused)]
#![allow(dead_code)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::derivable_impls)]
//! AgentKern-Arbiter: Conflict Resolution & Coordination Engine
//!
//! Per ARCHITECTURE.md: "The Core (Rust/Hyper-Loop)"
//!
//! Features implemented:
//! - **Thread-per-Core**: Minimal context switching for sub-ms latency
//! - **Raft Consensus**: Strong consistency for Atomic Business Locks
//! - **Priority Preemption**: Higher priority agents can preempt locks
//! - **ISO 42001 Audit Ledger**: Compliance traceability for all actions
//! - **Kill Switch**: Emergency agent termination
//! - **Carbon-Aware**: Sustainable computing
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    AgentKern-Arbiter                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │         Thread-per-Core Runtime (Hyper-Loop)                │
//! │  ┌─────────┐    ┌─────────┐    ┌─────────┐                 │
//! │  │ Core 0  │    │ Core 1  │    │ Core N  │                 │
//! │  │         │    │         │    │         │                 │
//! │  └────┬────┘    └────┬────┘    └────┬────┘                 │
//! │       │              │              │                       │
//! │       └──────────────┼──────────────┘                       │
//! │                      ▼                                      │
//! │           ┌─────────────────────┐                          │
//! │           │ Raft Lock Manager   │                          │
//! │           │ (Strong Consistency)│                          │
//! │           └─────────────────────┘                          │
//! │                      ▼                                      │
//! │           ┌─────────────────────┐                          │
//! │           │   Audit Ledger      │                          │
//! │           │ (ISO 42001 AIMS)    │                          │
//! │           └─────────────────────┘                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod coordinator;
pub mod locks;
pub mod queue;
pub mod types;

// Hyper-Stack modules (per ARCHITECTURE.md)
pub mod raft; // Raft Consensus for Atomic Business Locks
pub mod thread_per_core; // Thread-per-Core for minimal latency

// ISO 42001 Compliance (per GLOBAL_GAPS.md §3)
pub mod audit; // Audit Ledger for compliance traceability
pub mod iso42001; // ISO 42001 AIMS automated reporting

// EXECUTION_MANDATE.md modules
pub mod carbon;
pub mod killswitch; // Kill Switch for agent termination (Section 6) // Carbon-Aware Computing (Section 7)

// Roadmap modules
pub mod antifragile; // Anti-Fragile Self-Healing Engine
pub mod chaos; // Chaos Testing / Fault Injection
pub mod loop_prevention; // Runaway Loop Prevention ($47k incident)

// Phase 2: Human-in-the-Loop Escalation
pub mod escalation; // Escalation triggers, webhooks, approval workflow

// Phase 3: Security Hardening & Compliance
pub mod cost;
pub mod eu_ai_act; // EU AI Act (Aug 2025) compliance export // Cost attribution dashboard

// NOTE: gateway and marketplace moved to agentkern-nexus during consolidation
// See: packages/nexus/src/agent_card.rs, protocols/, marketplace/

// Re-exports
pub use antifragile::{
    AdaptationRate, AntifragileEngine, CircuitBreaker, CircuitState, Failure, FailureCategory,
    FailureClass, FailureSeverity, RecoveryStrategy, RecoveryStrategyType,
};
pub use audit::{AuditLedger, AuditOutcome, AuditRecord, AuditStatistics};
pub use carbon::{CarbonIntensity, CarbonRegion, CarbonScheduler};
pub use chaos::{ChaosConfig, ChaosError, ChaosMonkey, ChaosResult, ChaosStats};
pub use coordinator::Coordinator;
pub use cost::{AlertLevel, CostAlert, CostCategory, CostEvent, CostTracker, GlobalCostSummary};
pub use escalation::{
    ApprovalRequest, ApprovalStatus, ApprovalWorkflow, EscalationLevel, EscalationTrigger,
    TriggerConfig, TriggerResult, TriggerType, WebhookConfig, WebhookNotifier,
};
pub use eu_ai_act::{
    ComplianceReport, EuAiActExporter, OverallStatus, RiskLevel, TechnicalDocumentation,
};
pub use iso42001::{
    AuditEvent, AuditOutcome as Iso42001Outcome, AuditReport, ComplianceLedger, HumanOversight,
    ReportFormat, ReportGenerator,
};
pub use killswitch::{KillReason, KillRecord, KillSwitch, TerminationType};
pub use locks::LockManager;
pub use loop_prevention::{
    LoopPreventer, LoopPreventionConfig, LoopPreventionError, TrackedMessage,
};
pub use queue::PriorityQueue;
pub use raft::{RaftConfig, RaftLockManager, RaftState};
pub use thread_per_core::{ThreadPerCoreConfig, ThreadPerCoreRuntime};
pub use types::{BusinessLock, CoordinationRequest, CoordinationResult, LockType};
