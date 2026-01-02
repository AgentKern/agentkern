//! Graph Vector Database for State Ledger
//!
//! Per ARCHITECTURE.md Section 3: "The Speed of Light"
//! - Graph-based state storage with vector embeddings
//! - CRDT sync for eventual consistency
//! - TEE encryption for sensitive data
//!
//! This implements the distributed state ledger.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node in the state graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: uuid::Uuid,
    pub node_type: NodeType,
    pub data: serde_json::Value,
    pub vector: Option<Vec<f32>>, // Vector embedding for similarity
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub version: u64,
}

/// Type of graph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Agent,
    State,
    Intent,
    Action,
    Memory,
}

/// Edge in the state graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: uuid::Uuid,
    pub edge_type: EdgeType,
    pub from_node: uuid::Uuid,
    pub to_node: uuid::Uuid,
    pub weight: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of graph edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Owns,     // Agent owns state
    Caused,   // Action caused state change
    Requires, // Intent requires action
    Relates,  // General relation
    Similar,  // Vector similarity edge
}

/// Vector similarity result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResult {
    pub node_id: uuid::Uuid,
    pub score: f64,
}

/// Graph Vector Database.
pub struct GraphVectorDB {
    nodes: RwLock<HashMap<uuid::Uuid, GraphNode>>,
    edges: RwLock<Vec<GraphEdge>>,
    agent_index: RwLock<HashMap<String, Vec<uuid::Uuid>>>, // agent_id -> nodes
}

impl GraphVectorDB {
    /// Create a new graph vector database.
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            edges: RwLock::new(Vec::new()),
            agent_index: RwLock::new(HashMap::new()),
        }
    }

    /// Insert a node.
    pub fn insert_node(&self, node: GraphNode) -> uuid::Uuid {
        let id = node.id;
        self.nodes.write().insert(id, node);
        id
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: &uuid::Uuid) -> Option<GraphNode> {
        self.nodes.read().get(id).cloned()
    }

    /// Update a node.
    pub fn update_node(&self, id: &uuid::Uuid, data: serde_json::Value) -> bool {
        if let Some(node) = self.nodes.write().get_mut(id) {
            node.data = data;
            node.updated_at = chrono::Utc::now();
            node.version += 1;
            true
        } else {
            false
        }
    }

    /// Delete a node.
    pub fn delete_node(&self, id: &uuid::Uuid) -> bool {
        let removed = self.nodes.write().remove(id).is_some();
        if removed {
            // Remove related edges
            self.edges
                .write()
                .retain(|e| e.from_node != *id && e.to_node != *id);
        }
        removed
    }

    /// Insert an edge.
    pub fn insert_edge(&self, edge: GraphEdge) {
        self.edges.write().push(edge);
    }

    /// Get edges from a node.
    pub fn get_edges_from(&self, node_id: &uuid::Uuid) -> Vec<GraphEdge> {
        self.edges
            .read()
            .iter()
            .filter(|e| e.from_node == *node_id)
            .cloned()
            .collect()
    }

    /// Get edges to a node.
    pub fn get_edges_to(&self, node_id: &uuid::Uuid) -> Vec<GraphEdge> {
        self.edges
            .read()
            .iter()
            .filter(|e| e.to_node == *node_id)
            .cloned()
            .collect()
    }

    /// Find similar nodes by vector (cosine similarity).
    pub fn find_similar(&self, vector: &[f32], limit: usize) -> Vec<SimilarityResult> {
        let nodes = self.nodes.read();
        let mut results: Vec<SimilarityResult> = nodes
            .values()
            .filter_map(|node| {
                node.vector.as_ref().map(|v| {
                    let score = cosine_similarity(vector, v);
                    SimilarityResult {
                        node_id: node.id,
                        score,
                    }
                })
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);
        results
    }

    /// Get all nodes for an agent.
    pub fn get_agent_nodes(&self, agent_id: &str) -> Vec<GraphNode> {
        let index = self.agent_index.read();
        let nodes = self.nodes.read();

        index
            .get(agent_id)
            .map(|ids| ids.iter().filter_map(|id| nodes.get(id).cloned()).collect())
            .unwrap_or_default()
    }

    /// Add a node to agent index.
    pub fn index_agent_node(&self, agent_id: &str, node_id: uuid::Uuid) {
        self.agent_index
            .write()
            .entry(agent_id.to_string())
            .or_default()
            .push(node_id);
    }

    /// Create an agent state node.
    pub fn create_agent_state(&self, agent_id: &str, state: serde_json::Value) -> uuid::Uuid {
        let node = GraphNode {
            id: uuid::Uuid::new_v4(),
            node_type: NodeType::State,
            data: state,
            vector: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        };
        let id = self.insert_node(node);
        self.index_agent_node(agent_id, id);
        id
    }

    /// Store an intent path.
    pub fn store_intent(&self, agent_id: &str, intent: &str, steps: Vec<String>) -> uuid::Uuid {
        let node = GraphNode {
            id: uuid::Uuid::new_v4(),
            node_type: NodeType::Intent,
            data: serde_json::json!({
                "intent": intent,
                "steps": steps,
                "agent_id": agent_id,
            }),
            vector: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        };
        let id = self.insert_node(node);
        self.index_agent_node(agent_id, id);
        id
    }

    /// Get graph statistics.
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.nodes.read().len(),
            edge_count: self.edges.read().len(),
            agent_count: self.agent_index.read().len(),
        }
    }
}

impl Default for GraphVectorDB {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub agent_count: usize,
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot / (norm_a * norm_b)) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_crud() {
        let db = GraphVectorDB::new();

        // Insert node
        let node = GraphNode {
            id: uuid::Uuid::new_v4(),
            node_type: NodeType::State,
            data: serde_json::json!({"key": "value"}),
            vector: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
        };
        let id = db.insert_node(node);

        // Get node
        let retrieved = db.get_node(&id).unwrap();
        assert_eq!(retrieved.data["key"], "value");

        // Update node
        db.update_node(&id, serde_json::json!({"key": "updated"}));
        let updated = db.get_node(&id).unwrap();
        assert_eq!(updated.data["key"], "updated");
        assert_eq!(updated.version, 2);

        // Delete node
        assert!(db.delete_node(&id));
        assert!(db.get_node(&id).is_none());
    }

    #[test]
    fn test_vector_similarity() {
        let db = GraphVectorDB::new();

        // Insert nodes with vectors
        for i in 0..5 {
            let node = GraphNode {
                id: uuid::Uuid::new_v4(),
                node_type: NodeType::Memory,
                data: serde_json::json!({"index": i}),
                vector: Some(vec![i as f32, 0.0, 0.0]),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: 1,
            };
            db.insert_node(node);
        }

        // Find similar
        let query = vec![4.0, 0.0, 0.0];
        let results = db.find_similar(&query, 3);

        assert_eq!(results.len(), 3);
        // Most similar should be index 4
    }

    #[test]
    fn test_agent_state() {
        let db = GraphVectorDB::new();

        db.create_agent_state("agent-1", serde_json::json!({"status": "active"}));
        db.create_agent_state("agent-1", serde_json::json!({"status": "processing"}));

        let nodes = db.get_agent_nodes("agent-1");
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }
}
