//! Router Module
//!
//! Task routing and load balancing for Nexus.

mod load_balancer;

pub use load_balancer::{AgentLoad, LoadBalanceStrategy, LoadBalancer};

use crate::agent_card::AgentCard;
use crate::error::NexusError;
use crate::registry::AgentRegistry;
use crate::types::Task;
use std::sync::Arc;

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
            return Err(NexusError::NoMatchingAgent {
                task_type: task.task_type.clone(),
            });
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

        let idx = self
            .round_robin_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
        card.skill_match_score(&task.required_skills)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_balancer_creation() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        assert!(lb.all_loads().is_empty());
    }
}
