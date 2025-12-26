//! Task Router
//!
//! Routes tasks to the best matching agent based on skills and availability.
//!
//! OPEN SOURCE: Skill-based matching, round-robin load balancing
//! ENTERPRISE (ee/nexus-enterprise):
//! - Weighted routing based on performance history
//! - Cost-aware routing (via Treasury integration)
//! - Geographic/latency-aware routing
//! - Priority queues with SLA guarantees
//! - ML-based routing optimization

use std::sync::Arc;
use crate::agent_card::AgentCard;
use crate::registry::AgentRegistry;
use crate::types::Task;
use crate::error::NexusError;

/// Task router for matching tasks to agents.
pub struct TaskRouter {
    registry: Arc<AgentRegistry>,
    round_robin_counter: std::sync::atomic::AtomicUsize,
}

impl TaskRouter {
    /// Create a new task router.
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self {
            registry,
            round_robin_counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Find the best agent for a task.
    pub async fn find_best_agent(&self, task: &Task) -> Result<AgentCard, NexusError> {
        let candidates = self.find_candidates(task).await?;
        
        if candidates.is_empty() {
            return Err(NexusError::NoMatchingAgent { task_type: task.task_type.clone() });
        }
        
        // Score candidates and pick best
        let mut scored: Vec<(AgentCard, u8)> = candidates
            .into_iter()
            .map(|card| {
                let score = self.score_agent(&card, task);
                (card, score)
            })
            .collect();
        
        // Sort by score descending
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        
        // If top candidates have same score, use round-robin
        let top_score = scored[0].1;
        let top_candidates: Vec<_> = scored
            .into_iter()
            .filter(|(_, s)| *s == top_score)
            .map(|(c, _)| c)
            .collect();
        
        let idx = self.round_robin_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let selected = &top_candidates[idx % top_candidates.len()];
        
        Ok(selected.clone())
    }

    /// Find all candidate agents for a task.
    async fn find_candidates(&self, task: &Task) -> Result<Vec<AgentCard>, NexusError> {
        if task.required_skills.is_empty() {
            // Return all agents if no skills required
            return Ok(self.registry.list().await);
        }
        
        // Find agents with at least one required skill
        let mut candidates = Vec::new();
        for skill in &task.required_skills {
            let agents = self.registry.find_by_skill(skill).await;
            for agent in agents {
                if !candidates.iter().any(|a: &AgentCard| a.id == agent.id) {
                    candidates.push(agent);
                }
            }
        }
        
        Ok(candidates)
    }

    /// Score an agent for a task (0-100).
    fn score_agent(&self, card: &AgentCard, task: &Task) -> u8 {
        // Base score is skill match
        let skill_score = card.skill_match_score(&task.required_skills);
        
        // Could add more factors here in Enterprise:
        // - Historical success rate
        // - Current load
        // - Latency
        // - Cost
        
        skill_score
    }

    /// Route a task and return the selected agent.
    pub async fn route(&self, task: &Task) -> Result<AgentCard, NexusError> {
        self.find_best_agent(task).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Skill;

    fn skill(id: &str) -> Skill {
        Skill {
            id: id.into(),
            name: id.into(),
            description: "".into(),
            tags: vec![],
            input_schema: None,
            output_schema: None,
        }
    }

    #[tokio::test]
    async fn test_route_by_skill() {
        let registry = Arc::new(AgentRegistry::new());
        
        // Agent with NLP skill
        let mut nlp_agent = AgentCard::new("nlp-agent", "NLP Agent", "http://localhost:8001");
        nlp_agent.skills.push(skill("nlp"));
        registry.register(nlp_agent).await.unwrap();
        
        // Agent with vision skill
        let mut vision_agent = AgentCard::new("vision-agent", "Vision Agent", "http://localhost:8002");
        vision_agent.skills.push(skill("vision"));
        registry.register(vision_agent).await.unwrap();
        
        let router = TaskRouter::new(registry);
        
        // Task requiring NLP
        let task = Task::new("summarize", serde_json::Value::Null)
            .require_skills(vec!["nlp".into()]);
        
        let selected = router.route(&task).await.unwrap();
        assert_eq!(selected.id, "nlp-agent");
    }

    #[tokio::test]
    async fn test_no_matching_agent() {
        let registry = Arc::new(AgentRegistry::new());
        let router = TaskRouter::new(registry);
        
        let task = Task::new("impossible", serde_json::Value::Null)
            .require_skills(vec!["quantum_teleportation".into()]);
        
        let result = router.route(&task).await;
        assert!(matches!(result, Err(NexusError::NoMatchingAgent { .. })));
    }

    #[tokio::test]
    async fn test_round_robin_same_score() {
        let registry = Arc::new(AgentRegistry::new());
        
        // Two identical agents
        let mut agent1 = AgentCard::new("agent-1", "Agent 1", "http://localhost:8001");
        agent1.skills.push(skill("web"));
        registry.register(agent1).await.unwrap();
        
        let mut agent2 = AgentCard::new("agent-2", "Agent 2", "http://localhost:8002");
        agent2.skills.push(skill("web"));
        registry.register(agent2).await.unwrap();
        
        let router = TaskRouter::new(registry);
        let task = Task::new("browse", serde_json::Value::Null)
            .require_skills(vec!["web".into()]);
        
        // Route multiple times
        let first = router.route(&task).await.unwrap();
        let second = router.route(&task).await.unwrap();
        
        // Should alternate between agents
        assert_ne!(first.id, second.id);
    }
}
