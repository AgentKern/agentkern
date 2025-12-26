//! Demo Identity Provider Implementation
//!
//! Works without credentials - returns realistic demo data
//! Simulates any identity provider (Entra, Okta, Auth0, etc.)

use super::bridge::*;
use super::trust::*;
use crate::core::{ConnectionMode, ConnectionStatus, GracefulService};
use async_trait::async_trait;

/// Demo identity provider that works without credentials.
pub struct DemoIdentity {
    mode: ConnectionMode,
    trust_provider: TrustScoreProvider,
}

impl DemoIdentity {
    pub fn new() -> Self {
        Self {
            mode: ConnectionMode::detect("identity"),
            trust_provider: TrustScoreProvider::new(),
        }
    }
    
    /// Get trust score provider.
    pub fn trust_provider(&self) -> &TrustScoreProvider {
        &self.trust_provider
    }
}

impl Default for DemoIdentity {
    fn default() -> Self {
        Self::new()
    }
}

impl GracefulService for DemoIdentity {
    fn mode(&self) -> ConnectionMode {
        self.mode
    }
    
    fn status(&self) -> ConnectionStatus {
        ConnectionStatus::new("identity")
    }
}

#[async_trait]
impl IdentityBridge for DemoIdentity {
    async fn register_agent(&self, registration: &AgentRegistration) -> Result<ProviderAgentId, IdentityError> {
        Ok(ProviderAgentId {
            object_id: format!("demo-obj-{}", uuid::Uuid::new_v4()),
            app_id: format!("demo-app-{}", uuid::Uuid::new_v4()),
            sp_id: format!("demo-sp-{}", uuid::Uuid::new_v4()),
            did: registration.did.clone(),
        })
    }
    
    async fn map_did_to_provider(&self, did: &str) -> Result<ProviderAgentId, IdentityError> {
        Ok(ProviderAgentId {
            object_id: format!("demo-obj-{}", &did[..8.min(did.len())]),
            app_id: format!("demo-app-{}", &did[..8.min(did.len())]),
            sp_id: format!("demo-sp-{}", &did[..8.min(did.len())]),
            did: did.to_string(),
        })
    }
    
    async fn get_agent(&self, agent_id: &str) -> Result<ProviderAgent, IdentityError> {
        Ok(ProviderAgent {
            id: ProviderAgentId {
                object_id: agent_id.to_string(),
                app_id: format!("app-{}", agent_id),
                sp_id: format!("sp-{}", agent_id),
                did: format!("did:verimantle:{}", agent_id),
            },
            display_name: "[Demo] Test Agent".into(),
            description: Some("Demo agent - set VERIMANTLE_IDENTITY_API_KEY for live".into()),
            owner: "demo@example.com".into(),
            agent_type: AgentType::Custom,
            lifecycle_status: LifecycleStatus::Active,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_activity: Some(chrono::Utc::now().to_rfc3339()),
            trust_score: Some(0.85),
        })
    }
    
    async fn update_lifecycle(&self, _agent_id: &str, _status: LifecycleStatus) -> Result<(), IdentityError> {
        Ok(())
    }
    
    async fn check_conditional_access(&self, _agent_id: &str, resource: &str) -> Result<AccessDecision, IdentityError> {
        Ok(AccessDecision {
            allowed: true,
            reason: format!("[Demo] Access to {} allowed in demo mode", resource),
            conditions_met: vec!["DemoMode".into()],
            conditions_failed: vec![],
        })
    }
    
    async fn report_trust_score(&self, _agent_id: &str, _score: &TrustScore) -> Result<(), IdentityError> {
        Ok(())
    }
}

/// Factory to get identity provider with graceful fallback.
pub struct IdentityFactory;

impl IdentityFactory {
    /// Get identity provider.
    pub fn get() -> DemoIdentity {
        DemoIdentity::new()
    }
    
    /// Get connection status.
    pub fn status() -> ConnectionStatus {
        ConnectionStatus::new("identity")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_identity_works_without_credentials() {
        let identity = DemoIdentity::new();
        assert!(identity.is_available());
    }

    #[tokio::test]
    async fn test_demo_register_agent() {
        let identity = DemoIdentity::new();
        let registration = AgentRegistration {
            did: "did:verimantle:test".into(),
            display_name: "Test".into(),
            description: None,
            owner: "test@example.com".into(),
            agent_type: AgentType::Custom,
            tags: vec![],
        };
        
        let result = entra.register_agent(&registration).await;
        assert!(result.is_ok());
    }
}
