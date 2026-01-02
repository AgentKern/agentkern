//! Cost Attribution Dashboard - Track agent spending and billing
//!
//! Per MANDATE.md Section 1: Autonomous agents need spending controls
//! Per $47K Incident: Runaway loops can cause massive costs
//!
//! Features:
//! - Per-agent cost tracking
//! - Real-time budget alerts
//! - Cost breakdown by resource type
//! - Billing export for enterprise

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cost category types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostCategory {
    /// LLM API calls (OpenAI, Anthropic, etc.)
    LlmInference,
    /// Embedding generation
    Embedding,
    /// Vector database queries
    VectorSearch,
    /// Storage costs
    Storage,
    /// Compute (CPU/GPU)
    Compute,
    /// Network egress
    Network,
    /// External API calls
    ExternalApi,
    /// Carbon offset
    CarbonOffset,
    /// Custom category
    Custom(String),
}

/// Single cost event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEvent {
    /// Event ID
    pub id: String,
    /// Agent ID that incurred the cost
    pub agent_id: String,
    /// Timestamp (Unix ms)
    pub timestamp: u64,
    /// Cost category
    pub category: CostCategory,
    /// Amount in USD
    pub amount_usd: f64,
    /// Resource description
    pub resource: String,
    /// Usage quantity
    pub quantity: f64,
    /// Unit (tokens, requests, GB, etc.)
    pub unit: String,
    /// Task/workflow ID
    pub task_id: Option<String>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent cost summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentCostSummary {
    /// Agent ID
    pub agent_id: String,
    /// Total cost in USD
    pub total_usd: f64,
    /// Cost by category
    pub by_category: HashMap<String, f64>,
    /// Number of events
    pub event_count: u64,
    /// First event timestamp
    pub first_event: Option<u64>,
    /// Last event timestamp
    pub last_event: Option<u64>,
}

/// Cost alert threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostThreshold {
    /// Threshold ID
    pub id: String,
    /// Agent ID (or "*" for all)
    pub agent_id: String,
    /// Amount in USD
    pub amount_usd: f64,
    /// Time window (seconds, 0 = total)
    pub window_secs: u64,
    /// Alert level
    pub level: AlertLevel,
    /// Is threshold enabled?
    pub enabled: bool,
}

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Warning,
    Critical,
    Emergency,
}

impl AlertLevel {
    /// Should this level pause agent execution?
    pub fn should_pause(&self) -> bool {
        matches!(self, Self::Emergency)
    }
}

/// Cost alert triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAlert {
    /// Alert ID
    pub id: String,
    /// Threshold that triggered
    pub threshold_id: String,
    /// Agent ID
    pub agent_id: String,
    /// Current amount
    pub current_usd: f64,
    /// Threshold amount
    pub threshold_usd: f64,
    /// Alert level
    pub level: AlertLevel,
    /// Timestamp
    pub timestamp: u64,
    /// Was agent paused?
    pub agent_paused: bool,
}

/// Cost attribution tracker.
pub struct CostTracker {
    events: RwLock<Vec<CostEvent>>,
    thresholds: RwLock<Vec<CostThreshold>>,
    alerts: RwLock<Vec<CostAlert>>,
}

impl CostTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            thresholds: RwLock::new(Vec::new()),
            alerts: RwLock::new(Vec::new()),
        }
    }

    /// Record a cost event.
    pub fn record(&self, event: CostEvent) -> Option<CostAlert> {
        let agent_id = event.agent_id.clone();
        let amount = event.amount_usd;

        self.events.write().push(event);

        // Check thresholds
        self.check_thresholds(&agent_id, amount)
    }

    /// Create a cost event builder.
    pub fn event(&self, agent_id: &str, category: CostCategory) -> CostEventBuilder {
        CostEventBuilder {
            agent_id: agent_id.to_string(),
            category,
            resource: String::new(),
            amount_usd: 0.0,
            quantity: 0.0,
            unit: String::new(),
            task_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Get total cost for an agent.
    pub fn get_agent_total(&self, agent_id: &str) -> f64 {
        self.events
            .read()
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .map(|e| e.amount_usd)
            .sum()
    }

    /// Get agent cost summary.
    pub fn get_agent_summary(&self, agent_id: &str) -> AgentCostSummary {
        let events = self.events.read();
        let agent_events: Vec<_> = events.iter().filter(|e| e.agent_id == agent_id).collect();

        let mut by_category: HashMap<String, f64> = HashMap::new();
        for event in &agent_events {
            let key = format!("{:?}", event.category);
            *by_category.entry(key).or_insert(0.0) += event.amount_usd;
        }

        AgentCostSummary {
            agent_id: agent_id.to_string(),
            total_usd: agent_events.iter().map(|e| e.amount_usd).sum(),
            by_category,
            event_count: agent_events.len() as u64,
            first_event: agent_events.first().map(|e| e.timestamp),
            last_event: agent_events.last().map(|e| e.timestamp),
        }
    }

    /// Get all costs in a time window.
    pub fn get_window(&self, since: u64, until: u64) -> Vec<CostEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| e.timestamp >= since && e.timestamp <= until)
            .cloned()
            .collect()
    }

    /// Get global summary.
    pub fn get_global_summary(&self) -> GlobalCostSummary {
        let events = self.events.read();

        let mut by_agent: HashMap<String, f64> = HashMap::new();
        let mut by_category: HashMap<String, f64> = HashMap::new();

        for event in events.iter() {
            *by_agent.entry(event.agent_id.clone()).or_insert(0.0) += event.amount_usd;
            let cat_key = format!("{:?}", event.category);
            *by_category.entry(cat_key).or_insert(0.0) += event.amount_usd;
        }

        GlobalCostSummary {
            total_usd: events.iter().map(|e| e.amount_usd).sum(),
            by_agent,
            by_category,
            event_count: events.len() as u64,
            alert_count: self.alerts.read().len() as u64,
        }
    }

    /// Add a cost threshold.
    pub fn add_threshold(&self, threshold: CostThreshold) {
        self.thresholds.write().push(threshold);
    }

    /// Check thresholds for an agent.
    fn check_thresholds(&self, agent_id: &str, _new_amount: f64) -> Option<CostAlert> {
        let thresholds = self.thresholds.read();
        let current_total = self.get_agent_total(agent_id);

        for threshold in thresholds.iter() {
            if !threshold.enabled {
                continue;
            }

            // Check if threshold applies
            if threshold.agent_id != "*" && threshold.agent_id != agent_id {
                continue;
            }

            // Check if exceeded
            if current_total >= threshold.amount_usd {
                let alert = CostAlert {
                    id: uuid::Uuid::new_v4().to_string(),
                    threshold_id: threshold.id.clone(),
                    agent_id: agent_id.to_string(),
                    current_usd: current_total,
                    threshold_usd: threshold.amount_usd,
                    level: threshold.level,
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    agent_paused: threshold.level.should_pause(),
                };

                self.alerts.write().push(alert.clone());
                return Some(alert);
            }
        }

        None
    }

    /// Get recent alerts.
    pub fn get_alerts(&self, limit: usize) -> Vec<CostAlert> {
        let alerts = self.alerts.read();
        alerts.iter().rev().take(limit).cloned().collect()
    }

    /// Export costs to CSV.
    pub fn export_csv(&self) -> String {
        let events = self.events.read();
        let mut csv =
            String::from("id,agent_id,timestamp,category,amount_usd,resource,quantity,unit\n");

        for event in events.iter() {
            csv.push_str(&format!(
                "{},{},{},{:?},{:.6},{},{},{}\n",
                event.id,
                event.agent_id,
                event.timestamp,
                event.category,
                event.amount_usd,
                event.resource,
                event.quantity,
                event.unit
            ));
        }

        csv
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for cost events.
pub struct CostEventBuilder {
    agent_id: String,
    category: CostCategory,
    resource: String,
    amount_usd: f64,
    quantity: f64,
    unit: String,
    task_id: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl CostEventBuilder {
    pub fn resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = resource.into();
        self
    }

    pub fn amount(mut self, usd: f64) -> Self {
        self.amount_usd = usd;
        self
    }

    pub fn quantity(mut self, qty: f64, unit: impl Into<String>) -> Self {
        self.quantity = qty;
        self.unit = unit.into();
        self
    }

    pub fn task(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    pub fn meta(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn build(self) -> CostEvent {
        CostEvent {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: self.agent_id,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            category: self.category,
            amount_usd: self.amount_usd,
            resource: self.resource,
            quantity: self.quantity,
            unit: self.unit,
            task_id: self.task_id,
            metadata: self.metadata,
        }
    }
}

/// Global cost summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCostSummary {
    pub total_usd: f64,
    pub by_agent: HashMap<String, f64>,
    pub by_category: HashMap<String, f64>,
    pub event_count: u64,
    pub alert_count: u64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_create() {
        let tracker = CostTracker::new();
        assert_eq!(tracker.events.read().len(), 0);
    }

    #[test]
    fn test_record_event() {
        let tracker = CostTracker::new();

        let event = tracker
            .event("agent-1", CostCategory::LlmInference)
            .resource("gpt-4")
            .amount(0.003)
            .quantity(1000.0, "tokens")
            .build();

        tracker.record(event);

        assert_eq!(tracker.events.read().len(), 1);
    }

    #[test]
    fn test_agent_total() {
        let tracker = CostTracker::new();

        for _i in 0..5 {
            let event = tracker
                .event("agent-1", CostCategory::LlmInference)
                .amount(0.01)
                .build();
            tracker.record(event);
        }

        let total = tracker.get_agent_total("agent-1");
        assert!((total - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_agent_summary() {
        let tracker = CostTracker::new();

        tracker.record(
            tracker
                .event("agent-1", CostCategory::LlmInference)
                .amount(0.01)
                .build(),
        );
        tracker.record(
            tracker
                .event("agent-1", CostCategory::VectorSearch)
                .amount(0.002)
                .build(),
        );

        let summary = tracker.get_agent_summary("agent-1");

        assert!((summary.total_usd - 0.012).abs() < 0.001);
        assert_eq!(summary.event_count, 2);
    }

    #[test]
    fn test_threshold_alert() {
        let tracker = CostTracker::new();

        tracker.add_threshold(CostThreshold {
            id: "t1".into(),
            agent_id: "agent-1".into(),
            amount_usd: 0.05,
            window_secs: 0,
            level: AlertLevel::Warning,
            enabled: true,
        });

        // Record events until threshold exceeded
        for _ in 0..10 {
            let event = tracker
                .event("agent-1", CostCategory::LlmInference)
                .amount(0.01)
                .build();
            tracker.record(event);
        }

        // Should have alerts
        assert!(!tracker.alerts.read().is_empty());
    }

    #[test]
    fn test_global_summary() {
        let tracker = CostTracker::new();

        tracker.record(
            tracker
                .event("agent-1", CostCategory::LlmInference)
                .amount(0.01)
                .build(),
        );
        tracker.record(
            tracker
                .event("agent-2", CostCategory::Storage)
                .amount(0.005)
                .build(),
        );

        let summary = tracker.get_global_summary();

        assert!((summary.total_usd - 0.015).abs() < 0.001);
        assert_eq!(summary.by_agent.len(), 2);
    }

    #[test]
    fn test_export_csv() {
        let tracker = CostTracker::new();

        tracker.record(
            tracker
                .event("agent-1", CostCategory::LlmInference)
                .resource("gpt-4")
                .amount(0.003)
                .quantity(1000.0, "tokens")
                .build(),
        );

        let csv = tracker.export_csv();

        assert!(csv.contains("agent-1"));
        assert!(csv.contains("LlmInference"));
    }

    #[test]
    fn test_alert_level_pause() {
        assert!(!AlertLevel::Warning.should_pause());
        assert!(!AlertLevel::Critical.should_pause());
        assert!(AlertLevel::Emergency.should_pause());
    }
}
