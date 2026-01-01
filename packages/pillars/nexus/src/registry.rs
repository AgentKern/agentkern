//! Agent Registry
//!
//! Maintains a registry of known agents and their capabilities.
//! This is the OPEN SOURCE version with basic functionality.
//!
//! Enterprise features (in ee/nexus-enterprise):
//! - Distributed registry with Raft consensus
//! - Persistent storage (PostgreSQL, Redis)
//! - Multi-tenant isolation
//! - Advanced search and filtering

use crate::agent_card::AgentCard;
use crate::error::NexusError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent registry - in-memory implementation (Open Source).
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentCard>>>,
}

impl AgentRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent.
    pub async fn register(&self, card: AgentCard) -> Result<(), NexusError> {
        let id = card.id.clone();
        let mut agents = self.agents.write().await;

        if agents.contains_key(&id) {
            return Err(NexusError::AgentAlreadyExists { agent_id: id });
        }

        tracing::info!(agent_id = %id, name = %card.name, "Agent registered");
        agents.insert(id, card);
        Ok(())
    }

    /// Update an existing agent.
    pub async fn update(&self, card: AgentCard) -> Result<(), NexusError> {
        let id = card.id.clone();
        let mut agents = self.agents.write().await;

        if !agents.contains_key(&id) {
            return Err(NexusError::AgentNotFound { agent_id: id });
        }

        agents.insert(id, card);
        Ok(())
    }

    /// Unregister an agent.
    pub async fn unregister(&self, agent_id: &str) -> Result<AgentCard, NexusError> {
        let mut agents = self.agents.write().await;

        agents.remove(agent_id).ok_or(NexusError::AgentNotFound {
            agent_id: agent_id.to_string(),
        })
    }

    /// Get an agent by ID.
    pub async fn get(&self, agent_id: &str) -> Option<AgentCard> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// List all agents.
    pub async fn list(&self) -> Vec<AgentCard> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Find agents with a specific skill.
    pub async fn find_by_skill(&self, skill_id: &str) -> Vec<AgentCard> {
        let agents = self.agents.read().await;
        agents
            .values()
            .filter(|a| a.has_skill(skill_id) || a.has_skill_tag(skill_id))
            .cloned()
            .collect()
    }

    /// Count registered agents.
    pub async fn count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Skill;

    fn test_card(id: &str) -> AgentCard {
        AgentCard::new(id, format!("Agent {}", id), "http://localhost")
    }

    #[tokio::test]
    async fn test_register_agent() {
        let registry = AgentRegistry::new();

        registry.register(test_card("agent-1")).await.unwrap();

        assert_eq!(registry.count().await, 1);
        assert!(registry.get("agent-1").await.is_some());
    }

    #[tokio::test]
    async fn test_duplicate_registration() {
        let registry = AgentRegistry::new();

        registry.register(test_card("agent-1")).await.unwrap();
        let result = registry.register(test_card("agent-1")).await;

        assert!(matches!(result, Err(NexusError::AgentAlreadyExists { .. })));
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = AgentRegistry::new();

        registry.register(test_card("agent-1")).await.unwrap();
        registry.unregister("agent-1").await.unwrap();

        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_find_by_skill() {
        let registry = AgentRegistry::new();

        let mut card = test_card("agent-1");
        card.skills.push(Skill {
            id: "nlp".into(),
            name: "NLP".into(),
            description: "".into(),
            tags: vec!["text".into()],
            input_schema: None,
            output_schema: None,
        });

        registry.register(card).await.unwrap();
        registry.register(test_card("agent-2")).await.unwrap();

        let nlp_agents = registry.find_by_skill("nlp").await;
        assert_eq!(nlp_agents.len(), 1);
        assert_eq!(nlp_agents[0].id, "agent-1");
    }
}
