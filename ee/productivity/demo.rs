//! Demo Productivity Platform Implementation
//!
//! Works without credentials - returns realistic demo data
//! Simulates any productivity platform (Microsoft 365, Google Workspace, etc.)

use super::outlook::*;
use super::sharepoint::*;
use crate::core::{ConnectionMode, ConnectionStatus, GracefulService};
use async_trait::async_trait;

/// Demo productivity platform that works without credentials.
pub struct DemoProductivity {
    mode: ConnectionMode,
}

impl DemoProductivity {
    pub fn new() -> Self {
        Self {
            mode: ConnectionMode::detect("productivity"),
        }
    }
}

impl Default for DemoProductivity {
    fn default() -> Self {
        Self::new()
    }
}

impl GracefulService for DemoProductivity {
    fn mode(&self) -> ConnectionMode {
        self.mode
    }
    
    fn status(&self) -> ConnectionStatus {
        ConnectionStatus::new("productivity")
    }
}

#[async_trait]
impl OutlookConnector for DemoProductivity {
    async fn send_email(&self, email: &EmailMessage) -> Result<String, OutlookError> {
        Ok(format!("DEMO-EMAIL-{}", uuid::Uuid::new_v4()))
    }
    
    async fn get_unread(&self, _limit: u32) -> Result<Vec<EmailMessage>, OutlookError> {
        Ok(vec![
            EmailMessage {
                id: Some("demo-1".into()),
                subject: "[Demo] Weekly Report".into(),
                body: "This is demo data. Set VERIMANTLE_PRODUCTIVITY_API_KEY for live.".into(),
                body_type: BodyType::Text,
                from: Some("demo@example.com".into()),
                to: vec!["you@example.com".into()],
                cc: vec![],
                received_at: Some(chrono::Utc::now().to_rfc3339()),
                is_read: false,
                importance: Importance::Normal,
            }
        ])
    }
    
    async fn search(&self, query: &str) -> Result<Vec<EmailMessage>, OutlookError> {
        Ok(vec![
            EmailMessage {
                id: Some("demo-search-1".into()),
                subject: format!("[Demo] Search result for: {}", query),
                body: "Demo search result".into(),
                body_type: BodyType::Text,
                from: Some("demo@example.com".into()),
                to: vec![],
                cc: vec![],
                received_at: Some(chrono::Utc::now().to_rfc3339()),
                is_read: true,
                importance: Importance::Normal,
            }
        ])
    }
    
    async fn create_event(&self, _event: &CalendarEvent) -> Result<String, OutlookError> {
        Ok(format!("DEMO-EVENT-{}", uuid::Uuid::new_v4()))
    }
    
    async fn get_upcoming(&self, _days: u32) -> Result<Vec<CalendarEvent>, OutlookError> {
        Ok(vec![
            CalendarEvent {
                id: Some("demo-event-1".into()),
                subject: "[Demo] Team Standup".into(),
                body: Some("Demo meeting".into()),
                start: chrono::Utc::now().to_rfc3339(),
                end: (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339(),
                location: Some("Demo Room".into()),
                attendees: vec![],
                is_online_meeting: true,
                online_meeting_url: Some("https://demo.teams.microsoft.com/meeting".into()),
            }
        ])
    }
    
    async fn triage_meetings(&self, _instructions: &str) -> Result<TriageResult, OutlookError> {
        Ok(TriageResult {
            accepted: vec!["demo-meeting-1".into()],
            declined: vec![],
            requires_attention: vec!["demo-meeting-2".into()],
        })
    }
}

#[async_trait]
impl SharePointConnector for DemoProductivity {
    async fn search(&self, query: &str, _limit: u32) -> Result<Vec<SearchResult>, SharePointError> {
        Ok(vec![
            SearchResult {
                document: Document {
                    id: "demo-doc-1".into(),
                    name: format!("[Demo] {}.docx", query),
                    path: "/Demo Documents/".into(),
                    web_url: "https://demo.sharepoint.com/doc".into(),
                    size_bytes: 1024,
                    mime_type: "application/docx".into(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    modified_at: chrono::Utc::now().to_rfc3339(),
                    created_by: Some("demo@example.com".into()),
                    modified_by: None,
                },
                relevance_score: 0.95,
                highlights: vec!["...demo match...".into()],
            }
        ])
    }
    
    async fn get_document(&self, doc_id: &str) -> Result<Document, SharePointError> {
        Ok(Document {
            id: doc_id.to_string(),
            name: "[Demo] Document.docx".into(),
            path: "/Demo Documents/".into(),
            web_url: "https://demo.sharepoint.com/doc".into(),
            size_bytes: 1024,
            mime_type: "application/docx".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            created_by: Some("demo@example.com".into()),
            modified_by: None,
        })
    }
    
    async fn get_content(&self, _doc_id: &str) -> Result<String, SharePointError> {
        Ok("[Demo Content] This is demo data. Set VERIMANTLE_M365_API_KEY for live content.".into())
    }
    
    async fn list_folder(&self, folder_path: &str) -> Result<Vec<Document>, SharePointError> {
        Ok(vec![
            Document {
                id: "demo-doc-folder-1".into(),
                name: "[Demo] File1.docx".into(),
                path: format!("{}/File1.docx", folder_path),
                web_url: "https://demo.sharepoint.com/doc1".into(),
                size_bytes: 1024,
                mime_type: "application/docx".into(),
                created_at: chrono::Utc::now().to_rfc3339(),
                modified_at: chrono::Utc::now().to_rfc3339(),
                created_by: None,
                modified_by: None,
            }
        ])
    }
    
    async fn upload(&self, _folder_path: &str, name: &str, _content: &[u8]) -> Result<String, SharePointError> {
        Ok(format!("demo-uploaded-{}", name))
    }
    
    async fn get_recent(&self, _limit: u32) -> Result<Vec<Document>, SharePointError> {
        self.list_folder("/Recent").await
    }
}

/// Factory to get productivity platform with graceful fallback.
pub struct ProductivityFactory;

impl ProductivityFactory {
    /// Get productivity platform.
    pub fn get() -> DemoProductivity {
        DemoProductivity::new()
    }
    
    /// Get connection status.
    pub fn status() -> ConnectionStatus {
        ConnectionStatus::new("productivity")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_productivity_works_without_credentials() {
        let platform = DemoProductivity::new();
        assert!(platform.is_available());
    }

    #[tokio::test]
    async fn test_demo_outlook_get_unread() {
        let platform = DemoProductivity::new();
        let emails = platform.get_unread(10).await;
        assert!(emails.is_ok());
    }

    #[tokio::test]
    async fn test_demo_sharepoint_search() {
        let platform = DemoProductivity::new();
        let results = platform.search("test", 10).await;
        assert!(results.is_ok());
    }
}
