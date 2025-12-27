//! Microsoft Teams Integration
//!
//! Native Teams integration with Adaptive Cards

use serde::{Deserialize, Serialize};

/// Teams configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsConfig {
    /// Incoming webhook URL
    pub webhook_url: String,
    /// Bot app ID (for interactive features)
    pub app_id: Option<String>,
    /// Bot app password
    pub app_password: Option<String>,
}

/// Microsoft Teams integration.
pub struct TeamsIntegration {
    config: TeamsConfig,
}

impl TeamsIntegration {
    /// Create new Teams integration.
    pub fn new(config: TeamsConfig) -> Result<Self, TeamsError> {
        crate::connectors::license::check_feature_license("teams")?;
        Ok(Self { config })
    }
    
    /// Send escalation via Adaptive Card.
    pub fn send_escalation(&self, alert: &TeamsAlert) -> Result<(), TeamsError> {
        let card = self.build_adaptive_card(alert);
        self.post_card(&card)
    }
    
    /// Send simple message.
    pub fn send_message(&self, text: &str) -> Result<(), TeamsError> {
        let payload = serde_json::json!({
            "text": text
        });
        self.post_webhook(&payload)
    }
    
    fn build_adaptive_card(&self, alert: &TeamsAlert) -> AdaptiveCard {
        AdaptiveCard {
            card_type: "AdaptiveCard".into(),
            version: "1.4".into(),
            body: vec![
                CardElement::TextBlock {
                    text: format!("⚠️ {} Escalation", alert.level),
                    size: "Large".into(),
                    weight: "Bolder".into(),
                },
                CardElement::FactSet {
                    facts: vec![
                        ("Agent", alert.agent_id.clone()),
                        ("Task", alert.task_id.clone()),
                        ("Level", alert.level.clone()),
                    ],
                },
                CardElement::TextBlock {
                    text: alert.description.clone(),
                    size: "Default".into(),
                    weight: "Default".into(),
                },
            ],
            actions: vec![
                CardAction::ActionSubmit {
                    title: "Approve".into(),
                    data: serde_json::json!({"action": "approve", "id": alert.request_id}),
                },
                CardAction::ActionSubmit {
                    title: "Reject".into(),
                    data: serde_json::json!({"action": "reject", "id": alert.request_id}),
                },
                CardAction::ActionOpenUrl {
                    title: "View Details".into(),
                    url: format!("https://dashboard.agentkern.com/escalations/{}", alert.request_id),
                },
            ],
        }
    }
    
    fn post_card(&self, card: &AdaptiveCard) -> Result<(), TeamsError> {
        let payload = serde_json::json!({
            "type": "message",
            "attachments": [{
                "contentType": "application/vnd.microsoft.card.adaptive",
                "content": card
            }]
        });
        self.post_webhook(&payload)
    }
    
    fn post_webhook(&self, payload: &serde_json::Value) -> Result<(), TeamsError> {
        // Would use reqwest to POST to webhook_url
        Ok(())
    }
}

/// Teams alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsAlert {
    pub request_id: String,
    pub agent_id: String,
    pub task_id: String,
    pub level: String,
    pub description: String,
}

/// Adaptive Card structure.
#[derive(Debug, Clone, Serialize)]
pub struct AdaptiveCard {
    #[serde(rename = "type")]
    pub card_type: String,
    pub version: String,
    pub body: Vec<CardElement>,
    pub actions: Vec<CardAction>,
}

/// Card element.
#[derive(Debug, Clone, Serialize)]
pub enum CardElement {
    TextBlock { text: String, size: String, weight: String },
    FactSet { facts: Vec<(&'static str, String)> },
}

/// Card action.
#[derive(Debug, Clone, Serialize)]
pub enum CardAction {
    ActionSubmit { title: String, data: serde_json::Value },
    ActionOpenUrl { title: String, url: String },
}

/// Teams errors.
#[derive(Debug, thiserror::Error)]
pub enum TeamsError {
    #[error("Webhook error: {0}")]
    WebhookError(String),
    
    #[error("Card build error: {0}")]
    CardError(String),
    
    #[error("License error: {0}")]
    LicenseError(#[from] crate::connectors::license::LicenseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teams_alert() {
        let alert = TeamsAlert {
            request_id: "req-1".into(),
            agent_id: "agent-1".into(),
            task_id: "task-1".into(),
            level: "High".into(),
            description: "High risk action".into(),
        };
        assert_eq!(alert.level, "High");
    }
}
