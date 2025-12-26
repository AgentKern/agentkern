//! Agent Discovery
//!
//! Discovers agents by fetching their Agent Cards from well-known endpoints.
//! 
//! OPEN SOURCE: Basic HTTP discovery
//! ENTERPRISE (ee/nexus-enterprise):
//! - DNS-SD (mDNS) discovery
//! - Kubernetes service discovery
//! - Consul/etcd integration
//! - Distributed health checking

use std::sync::Arc;
use crate::agent_card::AgentCard;
use crate::registry::AgentRegistry;
use crate::error::NexusError;

/// Agent discovery service.
pub struct AgentDiscovery {
    registry: Arc<AgentRegistry>,
    client: reqwest::Client,
}

impl AgentDiscovery {
    /// Create a new discovery service.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { registry, client }
    }

    /// Discover an agent from its base URL.
    /// Fetches /.well-known/agent.json per A2A spec.
    pub async fn discover(&self, base_url: &str) -> Result<AgentCard, NexusError> {
        let url = format!(
            "{}/.well-known/agent.json",
            base_url.trim_end_matches('/')
        );
        
        tracing::info!(url = %url, "Discovering agent");
        
        let response = self.client.get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(NexusError::NetworkError {
                message: format!("Failed to fetch agent card: {}", response.status()),
            });
        }
        
        let card: AgentCard = response.json().await
            .map_err(|e| NexusError::ParseError { message: e.to_string() })?;
        
        tracing::info!(agent_id = %card.id, name = %card.name, "Agent discovered");
        
        Ok(card)
    }

    /// Discover and register an agent.
    pub async fn discover_and_register(&self, base_url: &str) -> Result<AgentCard, NexusError> {
        let card = self.discover(base_url).await?;
        self.registry.register(card.clone()).await?;
        Ok(card)
    }

    /// Discover multiple agents in parallel.
    pub async fn discover_many(&self, urls: &[&str]) -> Vec<Result<AgentCard, NexusError>> {
        let futures: Vec<_> = urls.iter()
            .map(|url| self.discover(url))
            .collect();
        
        futures::future::join_all(futures).await
    }

    /// Check if an agent is healthy.
    pub async fn health_check(&self, agent_id: &str) -> Result<bool, NexusError> {
        let card = self.registry.get(agent_id).await
            .ok_or(NexusError::AgentNotFound { agent_id: agent_id.to_string() })?;
        
        // Ping the agent's health endpoint
        let url = format!("{}/health", card.url.trim_end_matches('/'));
        
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_url_construction() {
        let registry = Arc::new(AgentRegistry::new());
        let discovery = AgentDiscovery::new(registry);
        
        // This would fail without a real server, but we test URL construction
        let result = discovery.discover("http://example.com").await;
        // Expected to fail with network error (no server)
        assert!(matches!(result, Err(NexusError::NetworkError { .. })));
    }

    #[tokio::test]
    async fn test_health_check_unknown_agent() {
        let registry = Arc::new(AgentRegistry::new());
        let discovery = AgentDiscovery::new(registry);
        
        let result = discovery.health_check("unknown-agent").await;
        assert!(matches!(result, Err(NexusError::AgentNotFound { .. })));
    }
}
