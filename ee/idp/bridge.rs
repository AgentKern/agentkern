//! Identity Provider Bridge
//!
//! Maps AgentKern DIDs to enterprise identity provider agent IDs
//! Supports: Entra, Okta, Auth0, Ping Identity, etc.

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// Identity provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// Tenant ID
    pub tenant_id: String,
    /// Client ID
    pub client_id: String,
    /// Client secret reference
    pub client_secret_ref: String,
    /// Graph API endpoint
    pub graph_endpoint: String,
}

impl Default for EntraConfig {
    fn default() -> Self {
        Self {
            tenant_id: String::new(),
            client_id: String::new(),
            client_secret_ref: String::new(),
            graph_endpoint: "https://graph.microsoft.com/v1.0".into(),
        }
    }
}

/// Identity provider bridge trait.
#[async_trait]
pub trait IdentityBridge: Send + Sync {
    /// Register agent in identity provider.
    async fn register_agent(&self, registration: &AgentRegistration) -> Result<ProviderAgentId, IdentityError>;
    
    /// Map AgentKern DID to provider agent ID.
    async fn map_did_to_provider(&self, did: &str) -> Result<ProviderAgentId, IdentityError>;
    
    /// Get agent by ID.
    async fn get_agent(&self, agent_id: &str) -> Result<ProviderAgent, IdentityError>;
    
    /// Update agent lifecycle (enable/disable/retire).
    async fn update_lifecycle(&self, agent_id: &str, status: LifecycleStatus) -> Result<(), IdentityError>;
    
    /// Apply Conditional Access check.
    async fn check_conditional_access(&self, agent_id: &str, resource: &str) -> Result<AccessDecision, IdentityError>;
    
    /// Report trust score to provider.
    async fn report_trust_score(&self, agent_id: &str, score: &super::TrustScore) -> Result<(), IdentityError>;
}

/// Agent registration request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    /// AgentKern DID
    pub did: String,
    /// Display name
    pub display_name: String,
    /// Description
    pub description: Option<String>,
    /// Owner (user principal name)
    pub owner: String,
    /// Agent type
    pub agent_type: AgentType,
    /// Tags
    pub tags: Vec<String>,
}

/// Agent type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Copilot agent
    Copilot,
    /// Custom agent
    Custom,
    /// External agent
    External,
    /// System agent
    System,
}

/// Provider Agent ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAgentId {
    /// Provider object ID
    pub object_id: String,
    /// Application ID
    pub app_id: String,
    /// Service principal ID
    pub sp_id: String,
    /// AgentKern DID (linked)
    pub did: String,
}

/// Provider Agent details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAgent {
    pub id: ProviderAgentId,
    pub display_name: String,
    pub description: Option<String>,
    pub owner: String,
    pub agent_type: AgentType,
    pub lifecycle_status: LifecycleStatus,
    pub created_at: String,
    pub last_activity: Option<String>,
    pub trust_score: Option<f64>,
}

/// Lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStatus {
    Active,
    Disabled,
    Suspended,
    Retired,
}

/// Access decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub allowed: bool,
    pub reason: String,
    pub conditions_met: Vec<String>,
    pub conditions_failed: Vec<String>,
}

/// Identity provider error.
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Conditional Access denied: {0}")]
    ConditionalAccessDenied(String),
    
    #[error("API error: {0}")]
    ApiError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_registration() {
        let reg = AgentRegistration {
            did: "did:agentkern:agent-1".into(),
            display_name: "Test Agent".into(),
            description: None,
            owner: "admin@example.com".into(),
            agent_type: AgentType::Custom,
            tags: vec!["test".into()],
        };
        assert_eq!(reg.agent_type, AgentType::Custom);
    }
}
