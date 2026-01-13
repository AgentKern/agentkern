use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::State,
    http::StatusCode,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::{GraphVectorDB, GraphNode, NodeType};

/// Synapse App State
#[derive(Clone)]
pub struct SynapseState {
    pub db: Arc<GraphVectorDB>,
}

pub fn router() -> Router {
    let db = Arc::new(GraphVectorDB::new());
    let state = SynapseState { db };

    Router::new()
        .route("/health", get(health_check))
        .route("/memory/store", post(store_memory))
        .route("/memory/query", post(query_memory))
        .with_state(state)
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "pillar": "synapse",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn store_memory(
    State(state): State<SynapseState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    // Extract info
    // In real app, we'd use a DTO
    let content = payload["content"].clone();
    let vector: Option<Vec<f32>> = serde_json::from_value(payload["vector"].clone()).ok();
    
    let node = GraphNode {
        id: Uuid::new_v4(),
        node_type: NodeType::Memory,
        data: content,
        vector,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
    };
    
    let id = state.db.insert_node(node);
    
    (StatusCode::CREATED, Json(json!({
        "stored": true,
        "id": id,
        "status": "persisted"
    })))
}

async fn query_memory(
    State(state): State<SynapseState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let vector: Option<Vec<f32>> = serde_json::from_value(payload["vector"].clone()).ok();
    let limit = payload["limit"].as_u64().unwrap_or(5) as usize;
    
    if let Some(vec) = vector {
        let results = state.db.find_similar(&vec, limit);
        (StatusCode::OK, Json(json!({
            "results": results,
            "count": results.len()
        })))
    } else {
        // Fallback: Return recent memories or error?
        // For now, error as vector query requires vector
        (StatusCode::BAD_REQUEST, Json(json!({
            "error": "Vector required for similarity search"
        })))
    }
}
