//! Outlook Connector
//!
//! Email and calendar integration for productivity agents

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Outlook connector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlookConfig {
    /// Tenant ID
    pub tenant_id: String,
    /// Client ID
    pub client_id: String,
    /// Client secret reference
    pub client_secret_ref: String,
    /// User principal name
    pub user_principal_name: Option<String>,
}

/// Outlook connector trait.
#[async_trait]
pub trait OutlookConnector: Send + Sync {
    /// Send email.
    async fn send_email(&self, email: &EmailMessage) -> Result<String, OutlookError>;
    
    /// Get unread emails.
    async fn get_unread(&self, limit: u32) -> Result<Vec<EmailMessage>, OutlookError>;
    
    /// Search emails.
    async fn search(&self, query: &str) -> Result<Vec<EmailMessage>, OutlookError>;
    
    /// Create calendar event.
    async fn create_event(&self, event: &CalendarEvent) -> Result<String, OutlookError>;
    
    /// Get upcoming events.
    async fn get_upcoming(&self, days: u32) -> Result<Vec<CalendarEvent>, OutlookError>;
    
    /// Delegate meeting triage.
    async fn triage_meetings(&self, instructions: &str) -> Result<TriageResult, OutlookError>;
}

/// Email message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: Option<String>,
    pub subject: String,
    pub body: String,
    pub body_type: BodyType,
    pub from: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub received_at: Option<String>,
    pub is_read: bool,
    pub importance: Importance,
}

/// Body type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BodyType {
    Text,
    Html,
}

/// Importance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Importance {
    Low,
    Normal,
    High,
}

/// Calendar event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Option<String>,
    pub subject: String,
    pub body: Option<String>,
    pub start: String,
    pub end: String,
    pub location: Option<String>,
    pub attendees: Vec<Attendee>,
    pub is_online_meeting: bool,
    pub online_meeting_url: Option<String>,
}

/// Attendee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attendee {
    pub email: String,
    pub name: Option<String>,
    pub response: AttendeeResponse,
}

/// Attendee response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttendeeResponse {
    None,
    Accepted,
    Declined,
    Tentative,
}

/// Triage result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageResult {
    pub accepted: Vec<String>,
    pub declined: Vec<String>,
    pub requires_attention: Vec<String>,
}

/// Outlook error.
#[derive(Debug, thiserror::Error)]
pub enum OutlookError {
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_message() {
        let email = EmailMessage {
            id: None,
            subject: "Test".into(),
            body: "Hello".into(),
            body_type: BodyType::Text,
            from: None,
            to: vec!["test@example.com".into()],
            cc: vec![],
            received_at: None,
            is_read: false,
            importance: Importance::Normal,
        };
        assert_eq!(email.subject, "Test");
    }
}
