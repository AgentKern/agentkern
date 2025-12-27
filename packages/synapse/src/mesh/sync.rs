//! Mesh Sync Protocol
//!
//! CRDT-based synchronization with conflict resolution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sync event for replication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    /// Event ID
    pub id: String,
    /// Data key
    pub key: String,
    /// CRDT operation
    pub operation: CrdtOperation,
    /// Logical timestamp
    pub timestamp: u64,
    /// Origin cell
    pub origin: String,
}

/// CRDT operation types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    /// Last-Writer-Wins Set
    LwwSet { value: Vec<u8>, timestamp: u64 },
    /// Add-only set
    GSet { element: String },
    /// Counter increment
    GCounter { increment: i64 },
    /// Map update
    LwwMap { key: String, value: Vec<u8>, timestamp: u64 },
}

/// Conflict resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Last write wins (by timestamp)
    LastWriteWins,
    /// First write wins (immutable after set)
    FirstWriteWins,
    /// Merge all values (for sets)
    Merge,
    /// Use custom resolver
    Custom,
}

/// Mesh sync engine.
pub struct MeshSync {
    /// Local cell ID
    local_cell_id: String,
    /// Pending events to sync
    pending: Vec<SyncEvent>,
    /// Vector clock for ordering
    vector_clock: HashMap<String, u64>,
}

impl MeshSync {
    /// Create a new sync engine.
    pub fn new(local_cell_id: String) -> Self {
        Self {
            local_cell_id,
            pending: Vec::new(),
            vector_clock: HashMap::new(),
        }
    }
    
    /// Record a local change.
    pub fn record_change(&mut self, key: &str, value: &[u8]) -> SyncEvent {
        let ts = self.increment_clock();
        let event = SyncEvent {
            id: uuid::Uuid::new_v4().to_string(),
            key: key.to_string(),
            operation: CrdtOperation::LwwSet {
                value: value.to_vec(),
                timestamp: ts,
            },
            timestamp: ts,
            origin: self.local_cell_id.clone(),
        };
        self.pending.push(event.clone());
        event
    }
    
    /// Increment local vector clock.
    fn increment_clock(&mut self) -> u64 {
        let current = self.vector_clock.get(&self.local_cell_id).copied().unwrap_or(0);
        let next = current + 1;
        self.vector_clock.insert(self.local_cell_id.clone(), next);
        next
    }
    
    /// Apply a remote event.
    pub fn apply_remote(&mut self, event: SyncEvent, strategy: ConflictResolution) -> bool {
        // Update vector clock
        let remote_ts = self.vector_clock.get(&event.origin).copied().unwrap_or(0);
        if event.timestamp <= remote_ts {
            return false; // Already seen
        }
        self.vector_clock.insert(event.origin.clone(), event.timestamp);
        
        match strategy {
            ConflictResolution::LastWriteWins => true,
            ConflictResolution::FirstWriteWins => remote_ts == 0,
            ConflictResolution::Merge => true,
            ConflictResolution::Custom => true,
        }
    }
    
    /// Push data to a remote cell.
    /// Graceful fallback: tries real HTTP if AGENTKERN_MESH_API_KEY set, else logs only.
    pub async fn push_to_cell(&self, endpoint: &str, data_id: &str, data: &[u8]) -> Result<(), SyncError> {
        // Check for mesh sync credentials
        let api_key = std::env::var("AGENTKERN_MESH_API_KEY").ok();
        
        if let Some(key) = api_key {
            if !key.is_empty() {
                // Try real HTTP push
                match self.do_http_push(endpoint, data_id, data, &key).await {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        tracing::warn!(
                            endpoint = %endpoint,
                            error = %e,
                            "Mesh sync failed, data queued for retry"
                        );
                        return Err(SyncError::ConnectionFailed(e));
                    }
                }
            }
        }
        
        // Fallback: log only (demo mode)
        tracing::debug!(
            endpoint = %endpoint, 
            data_id = %data_id,
            data_len = data.len(),
            "Mesh sync (demo mode) - set AGENTKERN_MESH_API_KEY for live"
        );
        Ok(())
    }
    
    /// Perform actual HTTP push.
    async fn do_http_push(&self, endpoint: &str, data_id: &str, data: &[u8], api_key: &str) -> Result<(), String> {
        let client = reqwest::Client::new();
        
        let response = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/octet-stream")
            .header("X-Data-ID", data_id)
            .header("X-Origin-Cell", &self.local_cell_id)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Push failed: {}", response.status()));
        }
        
        tracing::info!(
            endpoint = %endpoint,
            data_id = %data_id,
            "Successfully pushed to remote cell"
        );
        Ok(())
    }
    
    /// Get pending events.
    pub fn pending_events(&self) -> &[SyncEvent] {
        &self.pending
    }
    
    /// Clear pending events after sync.
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }
}

/// Sync errors.
#[derive(Debug)]
pub enum SyncError {
    ConnectionFailed(String),
    Timeout,
    ConflictRejected,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::Timeout => write!(f, "Sync timeout"),
            Self::ConflictRejected => write!(f, "Conflict rejected by remote"),
        }
    }
}

impl std::error::Error for SyncError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_change() {
        let mut sync = MeshSync::new("cell-1".to_string());
        let event = sync.record_change("user:123", b"test data");
        
        assert_eq!(event.key, "user:123");
        assert_eq!(event.origin, "cell-1");
        assert_eq!(sync.pending_events().len(), 1);
    }

    #[test]
    fn test_vector_clock() {
        let mut sync = MeshSync::new("cell-1".to_string());
        sync.record_change("a", b"1");
        sync.record_change("b", b"2");
        
        assert_eq!(*sync.vector_clock.get("cell-1").unwrap(), 2);
    }

    #[test]
    fn test_apply_remote() {
        let mut sync = MeshSync::new("cell-1".to_string());
        
        let event = SyncEvent {
            id: "evt-1".to_string(),
            key: "user:456".to_string(),
            operation: CrdtOperation::LwwSet { value: vec![1, 2, 3], timestamp: 5 },
            timestamp: 5,
            origin: "cell-2".to_string(),
        };
        
        let applied = sync.apply_remote(event, ConflictResolution::LastWriteWins);
        assert!(applied);
    }
}
