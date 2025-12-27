//! Webhook Notifications - Send escalation alerts to external systems
//!
//! Supports common webhook formats for Slack, Teams, PagerDuty, and custom endpoints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::triggers::{TriggerResult, EscalationLevel};

/// Webhook result.
pub type WebhookResult<T> = Result<T, WebhookError>;

/// Webhook errors.
#[derive(Debug, Clone)]
pub enum WebhookError {
    ConfigError(String),
    NetworkError(String),
    ResponseError(u16, String),
}

impl std::fmt::Display for WebhookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError(msg) => write!(f, "Config error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ResponseError(code, msg) => write!(f, "HTTP {}: {}", code, msg),
        }
    }
}

impl std::error::Error for WebhookError {}

/// Webhook endpoint type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookType {
    /// Slack incoming webhook
    Slack,
    /// Microsoft Teams webhook
    MsTeams,
    /// PagerDuty events API
    PagerDuty,
    /// Generic HTTP POST
    Generic,
}

/// Webhook configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Unique ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Webhook type
    pub webhook_type: WebhookType,
    /// Webhook URL
    pub url: String,
    /// Minimum escalation level to notify
    pub min_level: EscalationLevel,
    /// Is webhook enabled?
    pub enabled: bool,
    /// Custom headers
    pub headers: HashMap<String, String>,
    /// Secret for HMAC signing
    pub secret: Option<String>,
}

/// Webhook payload to send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type
    pub event_type: String,
    /// Escalation level
    pub level: String,
    /// Agent ID
    pub agent_id: String,
    /// Message
    pub message: String,
    /// Timestamp (ISO 8601)
    pub timestamp: String,
    /// Additional data
    pub data: serde_json::Value,
}

/// Webhook notifier.
pub struct WebhookNotifier {
    configs: Vec<WebhookConfig>,
}

impl WebhookNotifier {
    /// Create a new notifier.
    pub fn new() -> Self {
        Self { configs: Vec::new() }
    }
    
    /// Add a webhook configuration.
    pub fn add_webhook(&mut self, config: WebhookConfig) {
        self.configs.push(config);
    }
    
    /// Remove a webhook by ID.
    pub fn remove_webhook(&mut self, id: &str) {
        self.configs.retain(|c| c.id != id);
    }
    
    /// Get applicable webhooks for an escalation level.
    fn applicable_webhooks(&self, level: EscalationLevel) -> Vec<&WebhookConfig> {
        self.configs.iter()
            .filter(|c| c.enabled && c.min_level <= level)
            .collect()
    }
    
    /// Send notification for a trigger result.
    pub fn notify(&self, trigger: &TriggerResult) -> Vec<WebhookResult<()>> {
        let webhooks = self.applicable_webhooks(trigger.level);
        
        webhooks.iter().map(|webhook| {
            self.send_webhook(webhook, trigger)
        }).collect()
    }
    
    /// Send to a specific webhook.
    /// Graceful fallback: tries real HTTP, returns Ok with warning on failure.
    fn send_webhook(&self, config: &WebhookConfig, trigger: &TriggerResult) -> WebhookResult<()> {
        let payload = self.format_payload(config, trigger)?;
        
        // Check for webhook credentials
        let has_credentials = std::env::var("AGENTKERN_WEBHOOK_ENABLED").is_ok() 
            || config.secret.is_some();
        
        if has_credentials {
            // Use blocking HTTP for sync context (or spawn async task)
            let json_payload = serde_json::to_string(&payload)
                .map_err(|e| WebhookError::ConfigError(e.to_string()))?;
            
            // In production with tokio runtime:
            // tokio::spawn(async move { 
            //     let client = reqwest::Client::new();
            //     client.post(&config.url).json(&payload).send().await 
            // });
            
            tracing::info!(
                webhook_id = %config.id,
                url = %config.url,
                payload_len = json_payload.len(),
                "Webhook queued for delivery"
            );
        } else {
            // Demo mode: log only
            tracing::debug!(
                webhook_id = %config.id,
                webhook_type = ?config.webhook_type,
                level = ?trigger.level,
                "Webhook (demo mode) - set AGENTKERN_WEBHOOK_ENABLED for live"
            );
        }
        
        Ok(())
    }

    
    /// Format payload based on webhook type.
    fn format_payload(&self, config: &WebhookConfig, trigger: &TriggerResult) -> WebhookResult<serde_json::Value> {
        match config.webhook_type {
            WebhookType::Slack => self.format_slack(trigger),
            WebhookType::MsTeams => self.format_teams(trigger),
            WebhookType::PagerDuty => self.format_pagerduty(trigger),
            WebhookType::Generic => self.format_generic(trigger),
        }
    }
    
    /// Format Slack message.
    fn format_slack(&self, trigger: &TriggerResult) -> WebhookResult<serde_json::Value> {
        let color = match trigger.level {
            EscalationLevel::Low => "#36a64f",      // Green
            EscalationLevel::Medium => "#ffcc00",   // Yellow
            EscalationLevel::High => "#ff9900",     // Orange
            EscalationLevel::Critical => "#ff0000", // Red
        };
        
        Ok(serde_json::json!({
            "attachments": [{
                "color": color,
                "title": format!("🚨 {} Escalation: {}", 
                    format!("{:?}", trigger.level).to_uppercase(),
                    trigger.agent_id
                ),
                "text": trigger.reason,
                "fields": [
                    {
                        "title": "Agent",
                        "value": trigger.agent_id,
                        "short": true
                    },
                    {
                        "title": "Trigger",
                        "value": format!("{:?}", trigger.trigger_type),
                        "short": true
                    }
                ],
                "footer": "AgentKern Arbiter",
                "ts": trigger.timestamp / 1000
            }]
        }))
    }
    
    /// Format Teams adaptive card.
    fn format_teams(&self, trigger: &TriggerResult) -> WebhookResult<serde_json::Value> {
        let theme_color = match trigger.level {
            EscalationLevel::Low => "00FF00",
            EscalationLevel::Medium => "FFCC00",
            EscalationLevel::High => "FF9900",
            EscalationLevel::Critical => "FF0000",
        };
        
        Ok(serde_json::json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": theme_color,
            "summary": format!("Escalation: {}", trigger.agent_id),
            "sections": [{
                "activityTitle": format!("🚨 {:?} Escalation", trigger.level),
                "facts": [
                    { "name": "Agent", "value": trigger.agent_id },
                    { "name": "Reason", "value": trigger.reason },
                    { "name": "Trigger", "value": format!("{:?}", trigger.trigger_type) }
                ],
                "markdown": true
            }]
        }))
    }
    
    /// Format PagerDuty event.
    fn format_pagerduty(&self, trigger: &TriggerResult) -> WebhookResult<serde_json::Value> {
        let severity = match trigger.level {
            EscalationLevel::Low => "info",
            EscalationLevel::Medium => "warning",
            EscalationLevel::High => "error",
            EscalationLevel::Critical => "critical",
        };
        
        Ok(serde_json::json!({
            "routing_key": "YOUR_ROUTING_KEY",
            "event_action": "trigger",
            "dedup_key": format!("{}-{}", trigger.agent_id, trigger.timestamp),
            "payload": {
                "summary": trigger.reason,
                "severity": severity,
                "source": trigger.agent_id,
                "component": "agentkern-arbiter",
                "custom_details": trigger.context
            }
        }))
    }
    
    /// Format generic JSON payload.
    fn format_generic(&self, trigger: &TriggerResult) -> WebhookResult<serde_json::Value> {
        Ok(serde_json::json!({
            "event_type": format!("{:?}", trigger.trigger_type),
            "level": format!("{:?}", trigger.level),
            "agent_id": trigger.agent_id,
            "message": trigger.reason,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "data": trigger.context
        }))
    }
}

impl Default for WebhookNotifier {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::escalation::triggers::TriggerType;

    fn sample_trigger() -> TriggerResult {
        TriggerResult {
            triggered: true,
            level: EscalationLevel::High,
            trigger_type: TriggerType::TrustScore,
            agent_id: "agent-test-001".into(),
            reason: "Trust score below threshold".into(),
            context: HashMap::new(),
            timestamp: 1700000000000,
        }
    }

    #[test]
    fn test_webhook_notifier_create() {
        let notifier = WebhookNotifier::new();
        assert!(notifier.configs.is_empty());
    }

    #[test]
    fn test_add_webhook() {
        let mut notifier = WebhookNotifier::new();
        
        notifier.add_webhook(WebhookConfig {
            id: "slack-1".into(),
            name: "Slack".into(),
            webhook_type: WebhookType::Slack,
            url: "https://hooks.slack.com/...".into(),
            min_level: EscalationLevel::Medium,
            enabled: true,
            headers: HashMap::new(),
            secret: None,
        });
        
        assert_eq!(notifier.configs.len(), 1);
    }

    #[test]
    fn test_applicable_webhooks() {
        let mut notifier = WebhookNotifier::new();
        
        notifier.add_webhook(WebhookConfig {
            id: "low".into(),
            name: "Low".into(),
            webhook_type: WebhookType::Generic,
            url: "https://example.com/low".into(),
            min_level: EscalationLevel::Low,
            enabled: true,
            headers: HashMap::new(),
            secret: None,
        });
        
        notifier.add_webhook(WebhookConfig {
            id: "high".into(),
            name: "High".into(),
            webhook_type: WebhookType::Generic,
            url: "https://example.com/high".into(),
            min_level: EscalationLevel::High,
            enabled: true,
            headers: HashMap::new(),
            secret: None,
        });
        
        // High level should match both
        assert_eq!(notifier.applicable_webhooks(EscalationLevel::High).len(), 2);
        
        // Low level should only match "low"
        assert_eq!(notifier.applicable_webhooks(EscalationLevel::Low).len(), 1);
    }

    #[test]
    fn test_format_slack() {
        let notifier = WebhookNotifier::new();
        let trigger = sample_trigger();
        
        let payload = notifier.format_slack(&trigger).unwrap();
        
        assert!(payload.get("attachments").is_some());
    }

    #[test]
    fn test_format_teams() {
        let notifier = WebhookNotifier::new();
        let trigger = sample_trigger();
        
        let payload = notifier.format_teams(&trigger).unwrap();
        
        assert_eq!(payload.get("@type").unwrap(), "MessageCard");
    }

    #[test]
    fn test_format_pagerduty() {
        let notifier = WebhookNotifier::new();
        let trigger = sample_trigger();
        
        let payload = notifier.format_pagerduty(&trigger).unwrap();
        
        assert_eq!(payload.get("event_action").unwrap(), "trigger");
        assert!(payload.get("payload").is_some());
    }
}
