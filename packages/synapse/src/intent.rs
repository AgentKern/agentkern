//! VeriMantle-Synapse: Intent Path Tracking
//!
//! Tracks the agent's goal progression to prevent "Intent Drift".
//!
//! Per ARCHITECTURE.md:
//! - Stores "Intent Paths" not just vectors
//! - Anchors agents to original business goals

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A step in an intent path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentStep {
    /// Step number (1-indexed)
    pub step: u32,
    /// Action taken
    pub action: String,
    /// Result of the action
    pub result: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Semantic embedding of the action (for drift detection)
    pub embedding: Option<Vec<f32>>,
}

impl IntentStep {
    pub fn new(step: u32, action: impl Into<String>) -> Self {
        Self {
            step,
            action: action.into(),
            result: None,
            timestamp: Utc::now(),
            embedding: None,
        }
    }

    pub fn with_result(mut self, result: impl Into<String>) -> Self {
        self.result = Some(result.into());
        self
    }
}

/// An intent path tracking an agent's goal progression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentPath {
    /// Unique path identifier
    pub id: Uuid,
    /// Agent who owns this path
    pub agent_id: String,
    /// Original intent/goal description
    pub original_intent: String,
    /// Semantic embedding of original intent
    pub intent_embedding: Option<Vec<f32>>,
    /// Current step (0 = not started)
    pub current_step: u32,
    /// Expected total steps
    pub expected_steps: u32,
    /// History of steps taken
    pub history: Vec<IntentStep>,
    /// Has drift been detected?
    pub drift_detected: bool,
    /// Drift score (0-100, higher = more drift)
    pub drift_score: u8,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl IntentPath {
    /// Create a new intent path.
    pub fn new(agent_id: impl Into<String>, intent: impl Into<String>, expected_steps: u32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_id: agent_id.into(),
            original_intent: intent.into(),
            intent_embedding: None,
            current_step: 0,
            expected_steps,
            history: Vec::new(),
            drift_detected: false,
            drift_score: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Record a step in the path.
    pub fn record_step(&mut self, action: impl Into<String>, result: Option<String>) -> &IntentStep {
        self.current_step += 1;
        let mut step = IntentStep::new(self.current_step, action);
        if let Some(r) = result {
            step = step.with_result(r);
        }
        self.history.push(step);
        self.updated_at = Utc::now();
        self.history.last().unwrap()
    }

    /// Check if the path is complete.
    pub fn is_complete(&self) -> bool {
        self.current_step >= self.expected_steps
    }

    /// Check if the path has exceeded expected steps.
    pub fn is_overrun(&self) -> bool {
        self.current_step > self.expected_steps
    }

    /// Get progress as a percentage.
    pub fn progress_percent(&self) -> f32 {
        if self.expected_steps == 0 {
            return 100.0;
        }
        (self.current_step as f32 / self.expected_steps as f32 * 100.0).min(100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_path_new() {
        let path = IntentPath::new("agent-1", "Process customer order", 5);
        
        assert_eq!(path.agent_id, "agent-1");
        assert_eq!(path.original_intent, "Process customer order");
        assert_eq!(path.expected_steps, 5);
        assert_eq!(path.current_step, 0);
        assert!(!path.is_complete());
    }

    #[test]
    fn test_intent_path_record_step() {
        let mut path = IntentPath::new("agent-1", "Test", 3);
        
        path.record_step("validate_input", Some("success".to_string()));
        assert_eq!(path.current_step, 1);
        assert_eq!(path.history.len(), 1);
        assert_eq!(path.history[0].action, "validate_input");
        
        path.record_step("process_data", None);
        assert_eq!(path.current_step, 2);
    }

    #[test]
    fn test_intent_path_completion() {
        let mut path = IntentPath::new("agent-1", "Test", 2);
        
        assert!(!path.is_complete());
        path.record_step("step1", None);
        assert!(!path.is_complete());
        path.record_step("step2", None);
        assert!(path.is_complete());
        assert!(!path.is_overrun());
        
        path.record_step("step3", None);
        assert!(path.is_overrun());
    }

    #[test]
    fn test_progress_percent() {
        let mut path = IntentPath::new("agent-1", "Test", 4);
        
        assert_eq!(path.progress_percent(), 0.0);
        path.record_step("step1", None);
        assert_eq!(path.progress_percent(), 25.0);
        path.record_step("step2", None);
        assert_eq!(path.progress_percent(), 50.0);
    }
}
