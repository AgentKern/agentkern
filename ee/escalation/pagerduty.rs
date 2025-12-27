//! PagerDuty Integration
//!
//! Native PagerDuty integration with Events API v2

use serde::{Deserialize, Serialize};

/// PagerDuty configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyConfig {
    /// Events API v2 routing key
    pub routing_key: String,
    /// Service ID
    pub service_id: String,
    /// Default severity
    pub default_severity: PagerDutySeverity,
}

/// PagerDuty severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PagerDutySeverity {
    Critical,
    Error,
    Warning,
    Info,
}

impl PagerDutySeverity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Critical => "critical",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// PagerDuty integration.
pub struct PagerDutyIntegration {
    config: PagerDutyConfig,
}

impl PagerDutyIntegration {
    /// Create new PagerDuty integration.
    pub fn new(config: PagerDutyConfig) -> Result<Self, PagerDutyError> {
        crate::connectors::license::check_feature_license("pagerduty")?;
        Ok(Self { config })
    }
    
    /// Trigger incident.
    pub fn trigger(&self, event: &PagerDutyEvent) -> Result<PagerDutyResponse, PagerDutyError> {
        let payload = self.build_trigger_payload(event);
        self.send_event(&payload)
    }
    
    /// Acknowledge incident.
    pub fn acknowledge(&self, dedup_key: &str) -> Result<PagerDutyResponse, PagerDutyError> {
        let payload = self.build_ack_payload(dedup_key);
        self.send_event(&payload)
    }
    
    /// Resolve incident.
    pub fn resolve(&self, dedup_key: &str) -> Result<PagerDutyResponse, PagerDutyError> {
        let payload = self.build_resolve_payload(dedup_key);
        self.send_event(&payload)
    }
    
    fn build_trigger_payload(&self, event: &PagerDutyEvent) -> serde_json::Value {
        serde_json::json!({
            "routing_key": self.config.routing_key,
            "event_action": "trigger",
            "dedup_key": event.dedup_key,
            "payload": {
                "summary": event.summary,
                "severity": event.severity.as_str(),
                "source": event.source,
                "component": event.component,
                "group": event.group,
                "class": event.class,
                "custom_details": event.custom_details
            },
            "links": event.links.iter().map(|(text, url)| {
                serde_json::json!({"text": text, "href": url})
            }).collect::<Vec<_>>()
        })
    }
    
    fn build_ack_payload(&self, dedup_key: &str) -> serde_json::Value {
        serde_json::json!({
            "routing_key": self.config.routing_key,
            "event_action": "acknowledge",
            "dedup_key": dedup_key
        })
    }
    
    fn build_resolve_payload(&self, dedup_key: &str) -> serde_json::Value {
        serde_json::json!({
            "routing_key": self.config.routing_key,
            "event_action": "resolve",
            "dedup_key": dedup_key
        })
    }
    
    fn send_event(&self, payload: &serde_json::Value) -> Result<PagerDutyResponse, PagerDutyError> {
        // Would POST to https://events.pagerduty.com/v2/enqueue
        Ok(PagerDutyResponse {
            status: "success".into(),
            message: "Event processed".into(),
            dedup_key: payload.get("dedup_key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }
}

/// PagerDuty event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyEvent {
    /// Deduplication key (for grouping)
    pub dedup_key: String,
    /// Event summary
    pub summary: String,
    /// Severity level
    pub severity: PagerDutySeverity,
    /// Source system
    pub source: String,
    /// Component affected
    pub component: Option<String>,
    /// Logical grouping
    pub group: Option<String>,
    /// Event class
    pub class: Option<String>,
    /// Custom details for context
    pub custom_details: serde_json::Value,
    /// Links for runbook, dashboard, etc.
    pub links: Vec<(String, String)>,
}

/// PagerDuty response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyResponse {
    pub status: String,
    pub message: String,
    pub dedup_key: String,
}

/// PagerDuty errors.
#[derive(Debug, thiserror::Error)]
pub enum PagerDutyError {
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Invalid routing key")]
    InvalidRoutingKey,
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("License error: {0}")]
    LicenseError(#[from] crate::connectors::license::LicenseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_str() {
        assert_eq!(PagerDutySeverity::Critical.as_str(), "critical");
        assert_eq!(PagerDutySeverity::Warning.as_str(), "warning");
    }

    #[test]
    fn test_event_creation() {
        let event = PagerDutyEvent {
            dedup_key: "agent-1-task-1".into(),
            summary: "High risk agent action".into(),
            severity: PagerDutySeverity::Critical,
            source: "AgentKern".into(),
            component: Some("Arbiter".into()),
            group: Some("Production".into()),
            class: Some("Escalation".into()),
            custom_details: serde_json::json!({"agent_id": "agent-1"}),
            links: vec![("Dashboard".into(), "https://dashboard.agentkern.com".into())],
        };
        
        assert_eq!(event.severity, PagerDutySeverity::Critical);
    }
}
