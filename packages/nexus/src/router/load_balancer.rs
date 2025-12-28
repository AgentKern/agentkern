//! Load Balancer for Nexus
//!
//! Distributes tasks across agents with various strategies.

use crate::agent_card::AgentCard;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Load balancing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadBalanceStrategy {
    /// Simple round-robin
    #[default]
    RoundRobin,
    /// Least connections/active tasks
    LeastConnections,
    /// Weighted by agent capacity
    Weighted,
    /// Random selection
    Random,
    /// Sticky sessions (same agent for same client)
    Sticky,
}

/// Agent load statistics.
#[derive(Debug, Clone, Default)]
pub struct AgentLoad {
    /// Current active tasks
    pub active_tasks: usize,
    /// Total tasks completed
    pub completed_tasks: usize,
    /// Average response time (ms)
    pub avg_response_ms: u64,
    /// Error rate (0-100)
    pub error_rate: u8,
    /// Weight (for weighted balancing)
    pub weight: u32,
    /// Is healthy
    pub healthy: bool,
}

/// Load balancer for distributing tasks.
pub struct LoadBalancer {
    /// Current strategy
    strategy: LoadBalanceStrategy,
    /// Round-robin counter
    rr_counter: AtomicUsize,
    /// Agent load statistics
    agent_loads: HashMap<String, AgentLoad>,
    /// Sticky session mapping (client -> agent)
    sticky_sessions: HashMap<String, String>,
}

impl LoadBalancer {
    /// Create a new load balancer.
    pub fn new(strategy: LoadBalanceStrategy) -> Self {
        Self {
            strategy,
            rr_counter: AtomicUsize::new(0),
            agent_loads: HashMap::new(),
            sticky_sessions: HashMap::new(),
        }
    }

    /// Register an agent with the load balancer.
    pub fn register_agent(&mut self, agent_id: &str, weight: u32) {
        self.agent_loads.insert(agent_id.to_string(), AgentLoad {
            weight,
            healthy: true,
            ..Default::default()
        });
    }

    /// Unregister an agent.
    pub fn unregister_agent(&mut self, agent_id: &str) {
        self.agent_loads.remove(agent_id);
        self.sticky_sessions.retain(|_, v| v != agent_id);
    }

    /// Select an agent from the list.
    pub fn select<'a>(&self, agents: &'a [AgentCard], client_id: Option<&str>) -> Option<&'a AgentCard> {
        if agents.is_empty() {
            return None;
        }

        // Filter to healthy agents only
        let healthy: Vec<_> = agents.iter()
            .filter(|a| self.is_healthy(&a.id))
            .collect();

        if healthy.is_empty() {
            // Fall back to any agent if none healthy
            return agents.first();
        }

        match self.strategy {
            LoadBalanceStrategy::RoundRobin => self.select_round_robin(&healthy),
            LoadBalanceStrategy::LeastConnections => self.select_least_connections(&healthy),
            LoadBalanceStrategy::Weighted => self.select_weighted(&healthy),
            LoadBalanceStrategy::Random => self.select_random(&healthy),
            LoadBalanceStrategy::Sticky => {
                if let Some(cid) = client_id {
                    self.select_sticky(&healthy, cid)
                } else {
                    self.select_round_robin(&healthy)
                }
            }
        }
    }

    /// Round-robin selection.
    fn select_round_robin<'a>(&self, agents: &[&'a AgentCard]) -> Option<&'a AgentCard> {
        let idx = self.rr_counter.fetch_add(1, Ordering::Relaxed) % agents.len();
        agents.get(idx).copied()
    }

    /// Select agent with least active connections.
    fn select_least_connections<'a>(&self, agents: &[&'a AgentCard]) -> Option<&'a AgentCard> {
        agents.iter()
            .min_by_key(|a| {
                self.agent_loads
                    .get(&a.id)
                    .map(|l| l.active_tasks)
                    .unwrap_or(0)
            })
            .copied()
    }

    /// Weighted selection based on capacity.
    fn select_weighted<'a>(&self, agents: &[&'a AgentCard]) -> Option<&'a AgentCard> {
        let total_weight: u32 = agents.iter()
            .map(|a| self.agent_loads.get(&a.id).map(|l| l.weight).unwrap_or(1))
            .sum();

        if total_weight == 0 {
            return self.select_round_robin(agents);
        }

        let random_point = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32) % total_weight;

        let mut cumulative = 0u32;
        for agent in agents {
            let weight = self.agent_loads.get(&agent.id).map(|l| l.weight).unwrap_or(1);
            cumulative += weight;
            if random_point < cumulative {
                return Some(agent);
            }
        }

        agents.first().copied()
    }

    /// Random selection.
    fn select_random<'a>(&self, agents: &[&'a AgentCard]) -> Option<&'a AgentCard> {
        let idx = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as usize) % agents.len();
        agents.get(idx).copied()
    }

    /// Sticky session selection.
    fn select_sticky<'a>(&self, agents: &[&'a AgentCard], client_id: &str) -> Option<&'a AgentCard> {
        if let Some(agent_id) = self.sticky_sessions.get(client_id) {
            if let Some(agent) = agents.iter().find(|a| &a.id == agent_id) {
                return Some(agent);
            }
        }
        
        // Fall back to round-robin for new clients
        self.select_round_robin(agents)
    }

    /// Check if agent is healthy.
    fn is_healthy(&self, agent_id: &str) -> bool {
        self.agent_loads
            .get(agent_id)
            .map(|l| l.healthy)
            .unwrap_or(true) // Default to healthy if unknown
    }

    /// Record task start.
    pub fn task_started(&mut self, agent_id: &str) {
        if let Some(load) = self.agent_loads.get_mut(agent_id) {
            load.active_tasks += 1;
        }
    }

    /// Record task completion.
    pub fn task_completed(&mut self, agent_id: &str, response_ms: u64, success: bool) {
        if let Some(load) = self.agent_loads.get_mut(agent_id) {
            load.active_tasks = load.active_tasks.saturating_sub(1);
            load.completed_tasks += 1;
            
            // Update rolling average
            let total = load.completed_tasks as u64;
            load.avg_response_ms = (load.avg_response_ms * (total - 1) + response_ms) / total;
            
            // Update error rate
            if !success {
                load.error_rate = ((load.error_rate as u32 * (total as u32 - 1) + 100) / total as u32) as u8;
            }
        }
    }

    /// Mark agent as unhealthy.
    pub fn mark_unhealthy(&mut self, agent_id: &str) {
        if let Some(load) = self.agent_loads.get_mut(agent_id) {
            load.healthy = false;
        }
    }

    /// Mark agent as healthy.
    pub fn mark_healthy(&mut self, agent_id: &str) {
        if let Some(load) = self.agent_loads.get_mut(agent_id) {
            load.healthy = true;
        }
    }

    /// Create sticky session.
    pub fn create_sticky_session(&mut self, client_id: &str, agent_id: &str) {
        self.sticky_sessions.insert(client_id.to_string(), agent_id.to_string());
    }

    /// Get load stats for an agent.
    pub fn get_load(&self, agent_id: &str) -> Option<&AgentLoad> {
        self.agent_loads.get(agent_id)
    }

    /// Get all agent loads.
    pub fn all_loads(&self) -> &HashMap<String, AgentLoad> {
        &self.agent_loads
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(LoadBalanceStrategy::RoundRobin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_card::AgentCard;

    fn create_test_agents() -> Vec<AgentCard> {
        vec![
            AgentCard::new("agent-1", "Agent 1", "http://agent1.local"),
            AgentCard::new("agent-2", "Agent 2", "http://agent2.local"),
            AgentCard::new("agent-3", "Agent 3", "http://agent3.local"),
        ]
    }

    #[test]
    fn test_round_robin() {
        let lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        let agents = create_test_agents();
        
        let first = lb.select(&agents, None).unwrap();
        let second = lb.select(&agents, None).unwrap();
        let third = lb.select(&agents, None).unwrap();
        let fourth = lb.select(&agents, None).unwrap();
        
        // Should rotate through agents
        assert_ne!(first.id, second.id);
        assert_ne!(second.id, third.id);
        assert_eq!(first.id, fourth.id); // Wraps around
    }

    #[test]
    fn test_least_connections() {
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::LeastConnections);
        let agents = create_test_agents();
        
        lb.register_agent("agent-1", 1);
        lb.register_agent("agent-2", 1);
        lb.register_agent("agent-3", 1);
        
        // Simulate load
        lb.task_started("agent-1");
        lb.task_started("agent-1");
        lb.task_started("agent-2");
        
        // Should select agent-3 (least loaded)
        let selected = lb.select(&agents, None).unwrap();
        assert_eq!(selected.id, "agent-3");
    }

    #[test]
    fn test_weighted() {
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::Weighted);
        let agents = create_test_agents();
        
        lb.register_agent("agent-1", 10);
        lb.register_agent("agent-2", 1);
        lb.register_agent("agent-3", 1);
        
        // With high weight, agent-1 should be selected most often
        let mut counts = HashMap::new();
        for _ in 0..100 {
            let selected = lb.select(&agents, None).unwrap();
            *counts.entry(selected.id.clone()).or_insert(0) += 1;
        }
        
        // agent-1 should have significantly more selections
        assert!(counts.get("agent-1").unwrap_or(&0) > counts.get("agent-2").unwrap_or(&0));
    }

    #[test]
    fn test_health_check() {
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        let agents = create_test_agents();
        
        lb.register_agent("agent-1", 1);
        lb.register_agent("agent-2", 1);
        
        lb.mark_unhealthy("agent-1");
        
        // Should never select unhealthy agent
        for _ in 0..10 {
            let selected = lb.select(&agents, None).unwrap();
            assert_ne!(selected.id, "agent-1");
        }
    }

    #[test]
    fn test_task_tracking() {
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        
        lb.register_agent("agent-1", 1);
        
        lb.task_started("agent-1");
        lb.task_started("agent-1");
        
        assert_eq!(lb.get_load("agent-1").unwrap().active_tasks, 2);
        
        lb.task_completed("agent-1", 100, true);
        
        assert_eq!(lb.get_load("agent-1").unwrap().active_tasks, 1);
        assert_eq!(lb.get_load("agent-1").unwrap().completed_tasks, 1);
    }
}
