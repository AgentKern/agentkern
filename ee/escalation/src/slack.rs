//! Slack Integration
//!
//! Native Slack integration with Block Kit and modal workflows

use serde::{Deserialize, Serialize};

/// Slack configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    /// Bot token (xoxb-...)
    pub bot_token: String,
    /// App token (xapp-...)
    pub app_token: Option<String>,
    /// Signing secret
    pub signing_secret: String,
    /// Default channel for alerts
    pub default_channel: String,
}

/// Slack integration for rich notifications.
pub struct SlackIntegration {
    config: SlackConfig,
}

impl SlackIntegration {
    /// Create new Slack integration.
    pub fn new(config: SlackConfig) -> Result<Self, SlackError> {
        agentkern_connectors_ee::license::check_feature_license("slack")?;
        Ok(Self { config })
    }

    /// Send escalation alert with Block Kit.
    pub fn send_escalation(
        &self,
        escalation: &EscalationAlert,
    ) -> Result<SlackResponse, SlackError> {
        let blocks = self.build_escalation_blocks(escalation);
        self.post_message(
            &escalation
                .channel
                .clone()
                .unwrap_or(self.config.default_channel.clone()),
            &blocks,
        )
    }

    /// Open approval modal.
    pub fn open_approval_modal(
        &self,
        trigger_id: &str,
        request: &ApprovalRequest,
    ) -> Result<(), SlackError> {
        let view = self.build_approval_modal(request);
        self.open_view(trigger_id, &view)
    }

    /// Update message with approval result.
    pub fn update_approval(
        &self,
        channel: &str,
        ts: &str,
        result: &ApprovalResult,
    ) -> Result<(), SlackError> {
        let blocks = self.build_result_blocks(result);
        self.update_message(channel, ts, &blocks)
    }

    fn build_escalation_blocks(&self, escalation: &EscalationAlert) -> Vec<SlackBlock> {
        vec![
            SlackBlock::Header {
                text: format!(":warning: {} Escalation", escalation.level),
            },
            SlackBlock::Section {
                text: escalation.description.clone(),
                fields: vec![
                    ("Agent", escalation.agent_id.clone()),
                    ("Task", escalation.task_id.clone()),
                    ("Level", format!("{:?}", escalation.level)),
                ],
            },
            SlackBlock::Actions {
                elements: vec![
                    SlackElement::Button {
                        text: "Approve".into(),
                        action_id: format!("approve_{}", escalation.request_id),
                        style: Some("primary".into()),
                    },
                    SlackElement::Button {
                        text: "Reject".into(),
                        action_id: format!("reject_{}", escalation.request_id),
                        style: Some("danger".into()),
                    },
                    SlackElement::Button {
                        text: "View Details".into(),
                        action_id: format!("details_{}", escalation.request_id),
                        style: None,
                    },
                ],
            },
        ]
    }

    fn build_approval_modal(&self, _request: &ApprovalRequest) -> SlackView {
        SlackView {
            view_type: "modal".into(),
            title: "Review Escalation".into(),
            submit: Some("Submit".into()),
            close: Some("Cancel".into()),
            blocks: vec![],
        }
    }

    fn build_result_blocks(&self, result: &ApprovalResult) -> Vec<SlackBlock> {
        vec![SlackBlock::Section {
            text: format!(
                "Request {} by {}",
                if result.approved {
                    "approved"
                } else {
                    "rejected"
                },
                result.approver
            ),
            fields: vec![],
        }]
    }

    fn post_message(
        &self,
        channel: &str,
        _blocks: &[SlackBlock],
    ) -> Result<SlackResponse, SlackError> {
        // Would use Slack Web API
        Ok(SlackResponse {
            ok: true,
            ts: chrono::Utc::now().timestamp_millis().to_string(),
            channel: channel.to_string(),
        })
    }

    fn open_view(&self, _trigger_id: &str, _view: &SlackView) -> Result<(), SlackError> {
        // Would use views.open API
        Ok(())
    }

    fn update_message(
        &self,
        _channel: &str,
        _ts: &str,
        _blocks: &[SlackBlock],
    ) -> Result<(), SlackError> {
        // Would use chat.update API
        Ok(())
    }
}

/// Escalation alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationAlert {
    pub request_id: String,
    pub agent_id: String,
    pub task_id: String,
    pub level: EscalationLevel,
    pub description: String,
    pub channel: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EscalationLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for EscalationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Approval request.
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub id: String,
    pub agent_id: String,
    pub action: String,
    pub context: String,
}

/// Approval result.
#[derive(Debug, Clone)]
pub struct ApprovalResult {
    pub request_id: String,
    pub approved: bool,
    pub approver: String,
    pub reason: Option<String>,
}

/// Slack response.
#[derive(Debug, Clone)]
pub struct SlackResponse {
    pub ok: bool,
    pub ts: String,
    pub channel: String,
}

/// Slack Block Kit block.
#[derive(Debug, Clone, Serialize)]
pub enum SlackBlock {
    Header {
        text: String,
    },
    Section {
        text: String,
        fields: Vec<(&'static str, String)>,
    },
    Actions {
        elements: Vec<SlackElement>,
    },
    Divider,
}

/// Slack Block Kit element.
#[derive(Debug, Clone, Serialize)]
pub enum SlackElement {
    Button {
        text: String,
        action_id: String,
        style: Option<String>,
    },
}

/// Slack view (modal).
#[derive(Debug, Clone, Serialize)]
pub struct SlackView {
    pub view_type: String,
    pub title: String,
    pub submit: Option<String>,
    pub close: Option<String>,
    pub blocks: Vec<SlackBlock>,
}

/// Slack errors.
#[derive(Debug, thiserror::Error)]
pub enum SlackError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid token")]
    InvalidToken,

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("License error: {0}")]
    LicenseError(#[from] agentkern_connectors_ee::license::LicenseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escalation_level() {
        let alert = EscalationAlert {
            request_id: "req-1".into(),
            agent_id: "agent-1".into(),
            task_id: "task-1".into(),
            level: EscalationLevel::High,
            description: "High risk action".into(),
            channel: None,
        };
        assert!(matches!(alert.level, EscalationLevel::High));
    }
}
