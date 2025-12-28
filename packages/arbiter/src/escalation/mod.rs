//! Escalation System - Human-in-the-Loop for AI Safety
//!
//! Per MANDATE.md Section 6: Autonomous Agent Security
//! Per Strategic Roadmap: Human-in-the-Loop Hub
//!
//! Provides escalation triggers when agents hit trust thresholds,
//! webhook notifications, and human approval workflows.

pub mod approval;
pub mod triggers;
pub mod webhook;

// Re-exports
pub use approval::{ApprovalDecision, ApprovalRequest, ApprovalStatus, ApprovalWorkflow};
pub use triggers::{
    EscalationLevel, EscalationTrigger, TriggerConfig, TriggerResult, TriggerType, TrustThreshold,
};
pub use webhook::{WebhookConfig, WebhookNotifier, WebhookPayload, WebhookResult};
