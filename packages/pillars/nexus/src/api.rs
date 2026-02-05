use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{AgentCard, Nexus};

/// Nexus App State
#[derive(Clone)]
pub struct NexusState {
    pub nexus: Arc<Nexus>,
}

pub fn router() -> Router {
    let nexus = Arc::new(Nexus::new());

    // Register defaults
    let nx_clone = nexus.clone();
    tokio::spawn(async move {
        use crate::protocols::{AgentKernAdapter, McpAdapter};
        nx_clone.register_adapter(AgentKernAdapter::new()).await;
        nx_clone.register_adapter(McpAdapter::new()).await;
        tracing::debug!("Registered AgentKern and MCP adapters");
    });

    let state = NexusState { nexus };

    Router::new()
        .route("/health", get(health_check))
        .route("/messages/send", post(send_message))
        .route("/agents/register", post(register_agent))
        .with_state(state)
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "pillar": "nexus",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn send_message(
    State(state): State<NexusState>,
    // Receive raw bytes to let adapters handle parsing
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    // Pass raw bytes to Nexus which auto-detects protocol via adapters
    match state.nexus.receive(&body).await {
        Ok(msg) => (
            StatusCode::OK,
            Json(json!({
                "sent": true,
                "id": msg.id,
                "status": "routed"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST, // Or InternalServerError depending on error
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn register_agent(
    State(state): State<NexusState>,
    Json(card): Json<AgentCard>,
) -> (StatusCode, Json<Value>) {
    match state.nexus.register_agent(card.clone()).await {
        Ok(_) => (
            StatusCode::CREATED,
            Json(json!({
                "registered": true,
                "agent_id": card.id
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
