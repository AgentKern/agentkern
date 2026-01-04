//! SharePoint Connector
//!
//! Document scanning and search for productivity agents

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// SharePoint connector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharePointConfig {
    /// Tenant ID
    pub tenant_id: String,
    /// Client ID
    pub client_id: String,
    /// Client secret reference
    pub client_secret_ref: String,
    /// Site URL
    pub site_url: Option<String>,
}

/// SharePoint connector trait.
#[async_trait]
pub trait SharePointConnector: Send + Sync {
    /// Search documents.
    async fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchResult>, SharePointError>;
    
    /// Get document by ID.
    async fn get_document(&self, doc_id: &str) -> Result<Document, SharePointError>;
    
    /// Get document content.
    async fn get_content(&self, doc_id: &str) -> Result<String, SharePointError>;
    
    /// List documents in folder.
    async fn list_folder(&self, folder_path: &str) -> Result<Vec<Document>, SharePointError>;
    
    /// Upload document.
    async fn upload(&self, folder_path: &str, name: &str, content: &[u8]) -> Result<String, SharePointError>;
    
    /// Get recent documents.
    async fn get_recent(&self, limit: u32) -> Result<Vec<Document>, SharePointError>;
}

/// Search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: Document,
    pub relevance_score: f64,
    pub highlights: Vec<String>,
}

/// Document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub path: String,
    pub web_url: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub created_at: String,
    pub modified_at: String,
    pub created_by: Option<String>,
    pub modified_by: Option<String>,
}

/// SharePoint error.
#[derive(Debug, thiserror::Error)]
pub enum SharePointError {
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("API error: {0}")]
    ApiError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document() {
        let doc = Document {
            id: "doc1".into(),
            name: "report.docx".into(),
            path: "/Documents/report.docx".into(),
            web_url: "https://example.sharepoint.com/doc".into(),
            size_bytes: 1024,
            mime_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            modified_at: "2025-01-02T00:00:00Z".into(),
            created_by: Some("user@example.com".into()),
            modified_by: None,
        };
        assert_eq!(doc.name, "report.docx");
    }
}
