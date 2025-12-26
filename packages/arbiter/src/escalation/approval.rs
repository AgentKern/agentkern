//! Approval Workflow - Human approval for high-risk agent actions
//!
//! Manages approval requests, decisions, and audit trails for human-in-the-loop.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::RwLock;
use super::triggers::{TriggerResult, EscalationLevel};

/// Approval status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    /// Awaiting human decision
    Pending,
    /// Approved by human
    Approved,
    /// Rejected by human
    Rejected,
    /// Auto-approved (low risk)
    AutoApproved,
    /// Timed out
    Expired,
}

/// Approval decision with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    /// Decision status
    pub status: ApprovalStatus,
    /// Approver ID (human)
    pub approver: Option<String>,
    /// Decision timestamp
    pub decided_at: Option<u64>,
    /// Reason/comments
    pub reason: Option<String>,
}

/// Approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Request ID
    pub id: String,
    /// Agent ID requesting approval
    pub agent_id: String,
    /// Action requiring approval
    pub action: String,
    /// Action parameters
    pub params: serde_json::Value,
    /// Escalation level
    pub level: EscalationLevel,
    /// Created timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Current status
    pub status: ApprovalStatus,
    /// Decision if made
    pub decision: Option<ApprovalDecision>,
    /// Context from trigger
    pub context: HashMap<String, serde_json::Value>,
}

impl ApprovalRequest {
    /// Check if request has expired.
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        now > self.expires_at
    }
    
    /// Check if request is pending.
    pub fn is_pending(&self) -> bool {
        self.status == ApprovalStatus::Pending && !self.is_expired()
    }
}

/// Approval workflow manager.
pub struct ApprovalWorkflow {
    requests: RwLock<HashMap<String, ApprovalRequest>>,
    auto_approve_levels: Vec<EscalationLevel>,
}

impl ApprovalWorkflow {
    /// Create a new workflow manager.
    pub fn new() -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            auto_approve_levels: vec![], // No auto-approve by default
        }
    }
    
    /// Create with auto-approve for low levels.
    pub fn with_auto_approve(levels: Vec<EscalationLevel>) -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            auto_approve_levels: levels,
        }
    }
    
    /// Create approval request from trigger.
    pub fn request_approval(&self, trigger: &TriggerResult, action: &str, params: serde_json::Value) -> ApprovalRequest {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let timeout = trigger.level.default_timeout_secs() * 1000;
        
        // Check for auto-approval
        let auto_approved = self.auto_approve_levels.contains(&trigger.level);
        
        let request = ApprovalRequest {
            id: id.clone(),
            agent_id: trigger.agent_id.clone(),
            action: action.to_string(),
            params,
            level: trigger.level,
            created_at: now,
            expires_at: now + timeout,
            status: if auto_approved { ApprovalStatus::AutoApproved } else { ApprovalStatus::Pending },
            decision: if auto_approved {
                Some(ApprovalDecision {
                    status: ApprovalStatus::AutoApproved,
                    approver: Some("system".into()),
                    decided_at: Some(now),
                    reason: Some("Auto-approved based on level".into()),
                })
            } else {
                None
            },
            context: trigger.context.clone(),
        };
        
        // Store request
        self.requests.write().insert(id, request.clone());
        
        request
    }
    
    /// Get approval request by ID.
    pub fn get_request(&self, id: &str) -> Option<ApprovalRequest> {
        self.requests.read().get(id).cloned()
    }
    
    /// List pending requests.
    pub fn pending_requests(&self) -> Vec<ApprovalRequest> {
        self.requests.read()
            .values()
            .filter(|r| r.is_pending())
            .cloned()
            .collect()
    }
    
    /// List requests by agent.
    pub fn requests_by_agent(&self, agent_id: &str) -> Vec<ApprovalRequest> {
        self.requests.read()
            .values()
            .filter(|r| r.agent_id == agent_id)
            .cloned()
            .collect()
    }
    
    /// Approve a request.
    pub fn approve(&self, request_id: &str, approver: &str, reason: Option<String>) -> Option<ApprovalRequest> {
        let mut requests = self.requests.write();
        
        if let Some(request) = requests.get_mut(request_id) {
            if request.status == ApprovalStatus::Pending {
                request.status = ApprovalStatus::Approved;
                request.decision = Some(ApprovalDecision {
                    status: ApprovalStatus::Approved,
                    approver: Some(approver.to_string()),
                    decided_at: Some(chrono::Utc::now().timestamp_millis() as u64),
                    reason,
                });
                return Some(request.clone());
            }
        }
        
        None
    }
    
    /// Reject a request.
    pub fn reject(&self, request_id: &str, approver: &str, reason: Option<String>) -> Option<ApprovalRequest> {
        let mut requests = self.requests.write();
        
        if let Some(request) = requests.get_mut(request_id) {
            if request.status == ApprovalStatus::Pending {
                request.status = ApprovalStatus::Rejected;
                request.decision = Some(ApprovalDecision {
                    status: ApprovalStatus::Rejected,
                    approver: Some(approver.to_string()),
                    decided_at: Some(chrono::Utc::now().timestamp_millis() as u64),
                    reason,
                });
                return Some(request.clone());
            }
        }
        
        None
    }
    
    /// Expire old pending requests.
    pub fn expire_stale(&self) -> Vec<String> {
        let mut requests = self.requests.write();
        let mut expired = Vec::new();
        
        for (id, request) in requests.iter_mut() {
            if request.status == ApprovalStatus::Pending && request.is_expired() {
                request.status = ApprovalStatus::Expired;
                request.decision = Some(ApprovalDecision {
                    status: ApprovalStatus::Expired,
                    approver: None,
                    decided_at: Some(chrono::Utc::now().timestamp_millis() as u64),
                    reason: Some("Request expired".into()),
                });
                expired.push(id.clone());
            }
        }
        
        expired
    }
    
    /// Get workflow statistics.
    pub fn stats(&self) -> WorkflowStats {
        let requests = self.requests.read();
        
        let mut stats = WorkflowStats::default();
        for request in requests.values() {
            stats.total += 1;
            match request.status {
                ApprovalStatus::Pending => stats.pending += 1,
                ApprovalStatus::Approved => stats.approved += 1,
                ApprovalStatus::Rejected => stats.rejected += 1,
                ApprovalStatus::AutoApproved => stats.auto_approved += 1,
                ApprovalStatus::Expired => stats.expired += 1,
            }
        }
        
        stats
    }
}

impl Default for ApprovalWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowStats {
    pub total: usize,
    pub pending: usize,
    pub approved: usize,
    pub rejected: usize,
    pub auto_approved: usize,
    pub expired: usize,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::escalation::triggers::TriggerType;

    fn sample_trigger(level: EscalationLevel) -> TriggerResult {
        TriggerResult {
            triggered: true,
            level,
            trigger_type: TriggerType::TrustScore,
            agent_id: "agent-test".into(),
            reason: "Test trigger".into(),
            context: HashMap::new(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }

    #[test]
    fn test_workflow_create() {
        let workflow = ApprovalWorkflow::new();
        assert!(workflow.pending_requests().is_empty());
    }

    #[test]
    fn test_request_approval() {
        let workflow = ApprovalWorkflow::new();
        let trigger = sample_trigger(EscalationLevel::High);
        
        let request = workflow.request_approval(&trigger, "delete_data", serde_json::json!({}));
        
        assert_eq!(request.status, ApprovalStatus::Pending);
        assert_eq!(request.agent_id, "agent-test");
    }

    #[test]
    fn test_auto_approve() {
        let workflow = ApprovalWorkflow::with_auto_approve(vec![EscalationLevel::Low]);
        let trigger = sample_trigger(EscalationLevel::Low);
        
        let request = workflow.request_approval(&trigger, "log_message", serde_json::json!({}));
        
        assert_eq!(request.status, ApprovalStatus::AutoApproved);
    }

    #[test]
    fn test_approve_request() {
        let workflow = ApprovalWorkflow::new();
        let trigger = sample_trigger(EscalationLevel::High);
        
        let request = workflow.request_approval(&trigger, "action", serde_json::json!({}));
        
        let approved = workflow.approve(&request.id, "admin@example.com", Some("Looks good".into()));
        
        assert!(approved.is_some());
        assert_eq!(approved.unwrap().status, ApprovalStatus::Approved);
    }

    #[test]
    fn test_reject_request() {
        let workflow = ApprovalWorkflow::new();
        let trigger = sample_trigger(EscalationLevel::High);
        
        let request = workflow.request_approval(&trigger, "action", serde_json::json!({}));
        
        let rejected = workflow.reject(&request.id, "admin@example.com", Some("Not allowed".into()));
        
        assert!(rejected.is_some());
        assert_eq!(rejected.unwrap().status, ApprovalStatus::Rejected);
    }

    #[test]
    fn test_get_pending() {
        let workflow = ApprovalWorkflow::new();
        let trigger = sample_trigger(EscalationLevel::High);
        
        workflow.request_approval(&trigger, "action1", serde_json::json!({}));
        workflow.request_approval(&trigger, "action2", serde_json::json!({}));
        
        assert_eq!(workflow.pending_requests().len(), 2);
    }

    #[test]
    fn test_stats() {
        let workflow = ApprovalWorkflow::new();
        let trigger = sample_trigger(EscalationLevel::High);
        
        let req1 = workflow.request_approval(&trigger, "action1", serde_json::json!({}));
        workflow.request_approval(&trigger, "action2", serde_json::json!({}));
        
        workflow.approve(&req1.id, "admin", None);
        
        let stats = workflow.stats();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.approved, 1);
        assert_eq!(stats.pending, 1);
    }
}
