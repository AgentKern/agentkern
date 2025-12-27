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

pub mod locks;
pub mod queue;
pub mod coordinator;
pub mod types;

// Hyper-Stack modules (per ARCHITECTURE.md)
pub mod raft;              // Raft Consensus for Atomic Business Locks
pub mod thread_per_core;   // Thread-per-Core for minimal latency

// ISO 42001 Compliance (per GLOBAL_GAPS.md §3)
pub mod audit;             // Audit Ledger for compliance traceability
pub mod iso42001;          // ISO 42001 AIMS automated reporting

// EXECUTION_MANDATE.md modules
pub mod killswitch;        // Kill Switch for agent termination (Section 6)
pub mod carbon;            // Carbon-Aware Computing (Section 7)

// Roadmap modules
pub mod antifragile;       // Anti-Fragile Self-Healing Engine
pub mod chaos;             // Chaos Testing / Fault Injection
pub mod loop_prevention;   // Runaway Loop Prevention ($47k incident)

// Phase 2: Human-in-the-Loop Escalation
pub mod escalation;        // Escalation triggers, webhooks, approval workflow

// Phase 3: Security Hardening & Compliance
pub mod eu_ai_act;         // EU AI Act (Aug 2025) compliance export
pub mod cost;              // Cost attribution dashboard

// NOTE: gateway and marketplace moved to agentkern-nexus during consolidation
// See: packages/nexus/src/agent_card.rs, protocols/, marketplace/


// Re-exports
pub use locks::LockManager;
pub use queue::PriorityQueue;
pub use coordinator::Coordinator;
pub use types::{BusinessLock, CoordinationRequest, CoordinationResult, LockType};
pub use raft::{RaftLockManager, RaftConfig, RaftState};
pub use thread_per_core::{ThreadPerCoreRuntime, ThreadPerCoreConfig};
pub use audit::{AuditLedger, AuditRecord, AuditOutcome, AuditStatistics};
pub use killswitch::{KillSwitch, KillReason, KillRecord, TerminationType};
pub use carbon::{CarbonScheduler, CarbonIntensity, CarbonRegion};
pub use antifragile::{
    AntifragileEngine, Failure, FailureClass, RecoveryStrategy, CircuitBreaker, CircuitState,
    FailureSeverity, FailureCategory, AdaptationRate, RecoveryStrategyType,
};
pub use chaos::{ChaosMonkey, ChaosConfig, ChaosError, ChaosResult, ChaosStats};
pub use loop_prevention::{LoopPreventer, LoopPreventionConfig, TrackedMessage, LoopPreventionError};
pub use escalation::{
    EscalationTrigger, TriggerType, TriggerConfig, TriggerResult, EscalationLevel,
    WebhookNotifier, WebhookConfig, ApprovalWorkflow, ApprovalRequest, ApprovalStatus,
};
pub use eu_ai_act::{
    EuAiActExporter, TechnicalDocumentation, ComplianceReport, RiskLevel, OverallStatus,
};
pub use cost::{
    CostTracker, CostEvent, CostCategory, CostAlert, AlertLevel, GlobalCostSummary,
};
pub use iso42001::{
    ComplianceLedger, AuditEvent, HumanOversight, AuditOutcome as Iso42001Outcome,
    AuditReport, ReportFormat, ReportGenerator,
};
