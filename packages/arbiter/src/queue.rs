//! AgentKern-Arbiter: Priority Queue
//!
//! Manages waiting requests when resources are locked.

use std::collections::HashMap;
use std::cmp::Ordering;
use chrono::{DateTime, Utc};

use crate::types::CoordinationRequest;

/// Entry in the priority queue.
#[derive(Debug, Clone)]
struct QueueEntry {
    request: CoordinationRequest,
    inserted_at: DateTime<Utc>,
}

// Higher priority comes first, then earlier requests
impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.request.priority.cmp(&other.request.priority) {
            Ordering::Equal => other.inserted_at.cmp(&self.inserted_at), // Earlier is better
            ord => ord,
        }
    }
}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.request.agent_id == other.request.agent_id 
            && self.request.resource == other.request.resource
    }
}

impl Eq for QueueEntry {}

/// Priority queue for resource coordination.
pub struct PriorityQueue {
    /// Queue per resource
    queues: HashMap<String, Vec<QueueEntry>>,
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
        }
    }

    /// Add a request to the queue.
    pub fn enqueue(&mut self, request: CoordinationRequest) -> usize {
        let entry = QueueEntry {
            request: request.clone(),
            inserted_at: Utc::now(),
        };

        let queue = self.queues.entry(request.resource.clone()).or_insert_with(Vec::new);
        
        // Check if already in queue
        if queue.iter().any(|e| e.request.agent_id == request.agent_id) {
            // Update existing entry
            queue.retain(|e| e.request.agent_id != request.agent_id);
        }
        
        queue.push(entry);
        queue.sort_by(|a, b| b.cmp(a)); // Sort descending (highest priority first)
        
        // Return position (1-indexed)
        queue.iter().position(|e| e.request.agent_id == request.agent_id)
            .map(|p| p + 1)
            .unwrap_or(0)
    }

    /// Remove a request from the queue.
    pub fn dequeue(&mut self, agent_id: &str, resource: &str) -> Option<CoordinationRequest> {
        if let Some(queue) = self.queues.get_mut(resource) {
            if let Some(pos) = queue.iter().position(|e| e.request.agent_id == agent_id) {
                return Some(queue.remove(pos).request);
            }
        }
        None
    }

    /// Get the next request in queue for a resource.
    pub fn peek(&self, resource: &str) -> Option<&CoordinationRequest> {
        self.queues.get(resource)
            .and_then(|q| q.first())
            .map(|e| &e.request)
    }

    /// Pop the next request from the queue.
    pub fn pop(&mut self, resource: &str) -> Option<CoordinationRequest> {
        if let Some(queue) = self.queues.get_mut(resource) {
            if !queue.is_empty() {
                return Some(queue.remove(0).request);
            }
        }
        None
    }

    /// Get the position of an agent in the queue for a resource.
    pub fn get_position(&self, agent_id: &str, resource: &str) -> Option<usize> {
        self.queues.get(resource)
            .and_then(|q| q.iter().position(|e| e.request.agent_id == agent_id))
            .map(|p| p + 1) // 1-indexed
    }

    /// Get the queue length for a resource.
    pub fn queue_length(&self, resource: &str) -> usize {
        self.queues.get(resource).map(|q| q.len()).unwrap_or(0)
    }

    /// Estimate wait time based on position and average lock duration.
    pub fn estimate_wait_ms(&self, position: usize, avg_lock_duration_ms: u64) -> u64 {
        (position as u64).saturating_sub(1) * avg_lock_duration_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LockType;

    fn make_request(agent: &str, resource: &str, priority: i32) -> CoordinationRequest {
        CoordinationRequest::new(agent, resource).with_priority(priority)
    }

    #[test]
    fn test_enqueue_and_position() {
        let mut queue = PriorityQueue::new();

        let pos1 = queue.enqueue(make_request("agent-1", "res", 5));
        let pos2 = queue.enqueue(make_request("agent-2", "res", 10));
        let pos3 = queue.enqueue(make_request("agent-3", "res", 3));

        // agent-2 should be first (highest priority)
        assert_eq!(queue.get_position("agent-2", "res"), Some(1));
        assert_eq!(queue.get_position("agent-1", "res"), Some(2));
        assert_eq!(queue.get_position("agent-3", "res"), Some(3));
    }

    #[test]
    fn test_pop_order() {
        let mut queue = PriorityQueue::new();

        queue.enqueue(make_request("agent-1", "res", 5));
        queue.enqueue(make_request("agent-2", "res", 10));
        queue.enqueue(make_request("agent-3", "res", 3));

        // Pop in priority order
        assert_eq!(queue.pop("res").unwrap().agent_id, "agent-2");
        assert_eq!(queue.pop("res").unwrap().agent_id, "agent-1");
        assert_eq!(queue.pop("res").unwrap().agent_id, "agent-3");
        assert!(queue.pop("res").is_none());
    }

    #[test]
    fn test_dequeue_specific() {
        let mut queue = PriorityQueue::new();

        queue.enqueue(make_request("agent-1", "res", 5));
        queue.enqueue(make_request("agent-2", "res", 10));

        let removed = queue.dequeue("agent-1", "res");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().agent_id, "agent-1");

        assert_eq!(queue.queue_length("res"), 1);
    }
}
