//! Escalation System - Human-in-the-Loop for AI Safety
//!
//! Per MANDATE.md Section 6: Autonomous Agent Security
//! Per Strategic Roadmap: Human-in-the-Loop Hub
//!
//! Provides escalation triggers when agents hit trust thresholds,
//! webhook notifications, and human approval workflows.

pub mod triggers;
pub mod webhook;
pub mod approval;

// Re-exports
pub use triggers::{
    EscalationTrigger, TriggerType, TriggerConfig, TriggerResult,
    TrustThreshold, EscalationLevel,
};
pub use webhook::{
    WebhookNotifier, WebhookConfig, WebhookPayload, WebhookResult,
};
pub use approval::{
    ApprovalWorkflow, ApprovalRequest, ApprovalDecision, ApprovalStatus,
};
