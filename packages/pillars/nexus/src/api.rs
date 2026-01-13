use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{AgentCard, Nexus, NexusMessage};

/// Nexus App State
#[derive(Clone)]
pub struct NexusState {
    pub nexus: Arc<Nexus>,
}

pub fn router() -> Router {
    let nexus = Arc::new(Nexus::new());
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
    Json(payload): Json<NexusMessage>,
) -> (StatusCode, Json<Value>) {
    // In a real scenario, this might come from an external adapter
    // For now, simpler injection of NexusMessage

    // We treat incoming API calls as "receiving" a message into the mesh
    // which then gets routed to the target agent

    // Convert serde_json::Value to byte payload?
    // Actually receiving NexusMessage directly via Json<> is cleaner if types match

    // Simulate receiving raw bytes (serializing the message)
    // In reality, this endpoint might accept a specific format for "A2A" or "HTTP"
    // and the adapter would parse it.
    // Here we'll just try to route it.

    // Direct routing for now
    // TODO: Use adapters properly

    // For now, let's assume this is a direct injection
    // We actually need to use `nexus.receive` which expects bytes, OR internal routing

    // Quick hack: Serialize to bytes and feed to "receive" which defaults to basic adapter?
    // Or just skip to routing if we trust the API input?

    // Let's rely on logic similar to `nexus.receive` but bypassing adapter detection for now
    // since we already have a structured object.

    // Re-serializing to mimic "wire format"
    let raw = match serde_json::to_vec(&payload) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
        }
    };

    match state.nexus.receive(&raw).await {
        Ok(msg) => (
            StatusCode::OK,
            Json(json!({
                "sent": true,
                "id": msg.id,
                "status": "routed"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
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
