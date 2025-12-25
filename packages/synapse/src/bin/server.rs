//! VeriMantle-Synapse Server
//!
//! HTTP server for the Synapse state store.

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, put},
    extract::{State, Path},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use verimantle_synapse::{StateStore, StateUpdate, IntentPath};

/// Application state
struct AppState {
    store: StateStore,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Debug, Deserialize)]
struct StartIntentRequest {
    intent: String,
    expected_steps: u32,
}

#[derive(Debug, Deserialize)]
struct RecordStepRequest {
    action: String,
    result: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create store
    let state = Arc::new(AppState {
        store: StateStore::new(),
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/state/:agent_id", get(get_state).put(update_state))
        .route("/intent/:agent_id", get(get_intent).post(start_intent))
        .route("/intent/:agent_id/step", post(record_step))
        .route("/intent/:agent_id/drift", get(check_drift))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("🧠 VeriMantle-Synapse server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: "0.1.0",
    })
}

async fn get_state(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> Result<Json<verimantle_synapse::AgentState>, StatusCode> {
    state.store.get_state(&agent_id).await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn update_state(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
    Json(updates): Json<std::collections::HashMap<String, serde_json::Value>>,
) -> Json<verimantle_synapse::AgentState> {
    let update = StateUpdate {
        agent_id,
        updates,
        deletes: None,
    };
    Json(state.store.update_state(update).await)
}

async fn get_intent(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> Result<Json<IntentPath>, StatusCode> {
    state.store.get_intent(&agent_id).await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn start_intent(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
    Json(req): Json<StartIntentRequest>,
) -> Json<IntentPath> {
    Json(state.store.start_intent(agent_id, req.intent, req.expected_steps).await)
}

async fn record_step(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
    Json(req): Json<RecordStepRequest>,
) -> Result<Json<IntentPath>, StatusCode> {
    state.store.record_step(&agent_id, req.action, req.result).await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn check_drift(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.store.check_drift(&agent_id).await
        .map(|r| Json(serde_json::json!({
            "drifted": r.drifted,
            "score": r.score,
            "reason": r.reason
        })))
        .ok_or(StatusCode::NOT_FOUND)
}
