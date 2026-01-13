use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::engine::{GateEngine, VerificationRequestBuilder};

/// Gate App State
#[derive(Clone)]
pub struct GateState {
    pub engine: Arc<GateEngine>,
}

pub fn router() -> Router {
    let engine = Arc::new(GateEngine::new());
    // TODO: Load policies from database or config file

    let state = GateState { engine };

    Router::new()
        .route("/health", get(health_check))
        .route("/verify", post(verify_policy))
        .with_state(state)
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "pillar": "gate",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn verify_policy(
    State(state): State<GateState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let agent_id = payload["agent_id"].as_str().unwrap_or("unknown");
    let action = payload["action"].as_str().unwrap_or("unknown");
    let namespace = payload["namespace"].as_str().unwrap_or("default");

    // Build context
    let mut builder = VerificationRequestBuilder::new(agent_id, action).namespace(namespace);

    if let Some(ctx) = payload["context"].as_object() {
        for (k, v) in ctx {
            builder = builder.context(k, v.clone());
        }
    }

    let request = builder.build();
    let result = state.engine.verify(request).await;

    // Log intent (Neuro-Symbolic)
    if let Some(score) = result.neural_risk_score {
        tracing::info!(
            "🧠 Neural analysis for {}: risk_score={} (symbolic={})",
            action,
            score,
            result.symbolic_risk_score
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "allowed": result.allowed,
            "request_id": result.request_id,
            "final_risk_score": result.final_risk_score,
            "reasoning": result.reasoning,
            "blocking_policies": result.blocking_policies,
            "latency_us": result.latency.total_us
        })),
    )
}
