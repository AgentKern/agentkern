//! AgentKern-Arbiter: Coordinator
//!
//! High-level coordination API combining locks and queues.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::locks::{LockManager, LockError};
use crate::queue::PriorityQueue;
use crate::types::{
    BusinessLock, CoordinationRequest, CoordinationResult, LockType,
};

/// The Arbiter Coordinator.
pub struct Coordinator {
    lock_manager: LockManager,
    queue: Arc<RwLock<PriorityQueue>>,
    avg_lock_duration_ms: u64,
}

impl Default for Coordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Coordinator {
    pub fn new() -> Self {
        Self {
            lock_manager: LockManager::new(),
            queue: Arc::new(RwLock::new(PriorityQueue::new())),
            avg_lock_duration_ms: 5000, // 5 seconds default
        }
    }

    /// Request coordination for a resource.
    pub async fn request(&self, request: CoordinationRequest) -> CoordinationResult {
        // Try to acquire lock
        match self.lock_manager.acquire(
            &request.agent_id,
            &request.resource,
            request.priority,
            request.operation,
            Some(request.expected_duration_ms),
        ).await {
            Ok(lock) => {
                // Lock acquired, remove from queue if present
                let mut queue = self.queue.write().await;
                queue.dequeue(&request.agent_id, &request.resource);
                CoordinationResult::granted(lock)
            }
            Err(LockError::ResourceLocked { .. }) => {
                // Add to queue
                let mut queue = self.queue.write().await;
                let position = queue.enqueue(request.clone()) as u32;
                let wait_ms = queue.estimate_wait_ms(position as usize, self.avg_lock_duration_ms);
                CoordinationResult::queued(position, wait_ms)
            }
            Err(e) => {
                CoordinationResult::denied(e.to_string())
            }
        }
    }

    /// Acquire a lock directly (bypass queue).
    pub async fn acquire_lock(
        &self,
        agent_id: &str,
        resource: &str,
        priority: i32,
    ) -> Result<BusinessLock, String> {
        self.lock_manager
            .acquire(agent_id, resource, priority, LockType::Write, None)
            .await
            .map_err(|e| e.to_string())
    }

    /// Release a lock and grant to next in queue if any.
    pub async fn release_lock(&self, agent_id: &str, resource: &str) -> Result<(), String> {
        self.lock_manager
            .release(agent_id, resource)
            .await
            .map_err(|e| e.to_string())?;

        // Check queue for next waiter
        let mut queue = self.queue.write().await;
        if let Some(next_request) = queue.pop(resource) {
            drop(queue); // Release lock before recursive call
            
            // Auto-grant to next in queue
            let _ = self.lock_manager.acquire(
                &next_request.agent_id,
                resource,
                next_request.priority,
                next_request.operation,
                Some(next_request.expected_duration_ms),
            ).await;
        }

        Ok(())
    }

    /// Get the status of a lock.
    pub async fn get_lock_status(&self, resource: &str) -> Option<BusinessLock> {
        self.lock_manager.get_status(resource).await
    }

    /// Get queue position for an agent.
    pub async fn get_queue_position(&self, agent_id: &str, resource: &str) -> Option<usize> {
        let queue = self.queue.read().await;
        queue.get_position(agent_id, resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_request_granted() {
        let coord = Coordinator::new();

        let request = CoordinationRequest::new("agent-1", "resource-1");
        let result = coord.request(request).await;

        assert!(result.granted);
        assert!(result.lock.is_some());
    }

    #[tokio::test]
    async fn test_coordinator_request_queued() {
        let coord = Coordinator::new();

        // First request gets lock
        let req1 = CoordinationRequest::new("agent-1", "resource-1");
        let result1 = coord.request(req1).await;
        assert!(result1.granted);

        // Second request gets queued
        let req2 = CoordinationRequest::new("agent-2", "resource-1");
        let result2 = coord.request(req2).await;
        assert!(!result2.granted);
        assert_eq!(result2.queue_position, Some(1));
    }

    #[tokio::test]
    async fn test_coordinator_release_grants_next() {
        let coord = Coordinator::new();

        // First agent gets lock
        let req1 = CoordinationRequest::new("agent-1", "resource-1");
        coord.request(req1).await;

        // Second agent queued
        let req2 = CoordinationRequest::new("agent-2", "resource-1");
        let result2 = coord.request(req2).await;
        assert!(!result2.granted);

        // First agent releases
        coord.release_lock("agent-1", "resource-1").await.unwrap();

        // Second agent should now have the lock
        let status = coord.get_lock_status("resource-1").await;
        assert!(status.is_some());
        assert_eq!(status.unwrap().locked_by, "agent-2");
    }
}
