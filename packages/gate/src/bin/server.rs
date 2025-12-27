//! AgentKern-Gate Server
//!
//! HTTP server for the Gate verification engine.
//! Uses Axum for high-performance HTTP handling.

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
    extract::State,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use agentkern_gate::{
    GateEngine,
    Policy,
    VerificationRequest,
    VerificationResult,
};

/// Application state
struct AppState {
    engine: GateEngine,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Debug, Deserialize)]
struct VerifyRequest {
    agent_id: String,
    action: String,
    #[serde(default)]
    context: std::collections::HashMap<String, serde_json::Value>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create engine
    let state = Arc::new(AppState {
        engine: GateEngine::new(),
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/verify", post(verify))
        .route("/policies", get(list_policies).post(register_policy))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("🚀 AgentKern-Gate server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: "0.1.0",
    })
}

async fn verify(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerificationResult>, StatusCode> {
    use agentkern_gate::engine::VerificationRequestBuilder;
    
    let mut builder = VerificationRequestBuilder::new(req.agent_id, req.action);
    for (key, value) in req.context {
        builder = builder.context(key, value);
    }
    
    let result = state.engine.verify(builder.build()).await;
    Ok(Json(result))
}

async fn list_policies(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Policy>> {
    let policies = state.engine.get_policies().await;
    Json(policies)
}

async fn register_policy(
    State(state): State<Arc<AppState>>,
    Json(policy): Json<Policy>,
) -> Result<Json<Policy>, StatusCode> {
    state.engine.register_policy(policy.clone()).await;
    Ok(Json(policy))
}
