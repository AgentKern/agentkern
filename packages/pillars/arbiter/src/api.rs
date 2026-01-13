use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::State,
    http::StatusCode,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{Coordinator, CoordinationRequest};
use agentkern_pulse::Pulse;

/// Arbiter App State
#[derive(Clone)]
pub struct ArbiterState {
    pub coordinator: Arc<Coordinator>,
}

pub fn router() -> Router {
    let coordinator = Arc::new(Coordinator::new());
    let state = ArbiterState { coordinator };

    Router::new()
        .route("/health", get(health_check))
        .route("/schedule", post(schedule_task))
        .route("/locks", post(acquire_lock_endpoint)) // Avoid conflict with method name
        .with_state(state)
}

async fn health_check(State(state): State<ArbiterState>) -> Json<Value> {
    // Check pulse
    let report = state.coordinator.get_health().await;
    
    Json(json!({
        "status": "ok",
        "pillar": "arbiter",
        "version": env!("CARGO_PKG_VERSION"),
        "report": report
    }))
}

async fn schedule_task(
    State(state): State<ArbiterState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    // Construct CoordinationRequest from payload
    let agent_id = payload["agent_id"].as_str().unwrap_or("unknown");
    let resource = payload["resource"].as_str().unwrap_or("global");
    
    let mut request = CoordinationRequest::new(agent_id, resource);
    
    if let Some(op) = payload["operation"].as_str() {
        request.operation = crate::types::LockType::Write; // Simplify for now
    }
    
    if let Some(priority) = payload["priority"].as_i64() {
        request.priority = priority as i32;
    }

    let result = state.coordinator.request(request).await;

    if result.granted {
        (StatusCode::CREATED, Json(json!({
            "scheduled": true,
            "lock_id": result.lock.map(|l| l.id),
            "status": "granted"
        })))
    } else if let Some(pos) = result.queue_position {
        (StatusCode::ACCEPTED, Json(json!({
            "scheduled": false,
            "status": "queued",
            "position": pos,
            "wait_ms": result.estimated_wait_ms
        })))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
            "scheduled": false,
            "status": "denied",
            "reason": result.reason
        })))
    }
}

async fn acquire_lock_endpoint(
    State(state): State<ArbiterState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let agent_id = payload["agent_id"].as_str().unwrap_or("unknown");
    let resource = payload["resource"].as_str().unwrap_or("global");
    let priority = payload["priority"].as_i64().unwrap_or(0) as i32;

    match state.coordinator.acquire_lock(agent_id, resource, priority).await {
        Ok(lock) => (StatusCode::OK, Json(json!({
            "locked": true,
            "lock_id": lock.id
        }))),
        Err(e) => (StatusCode::CONFLICT, Json(json!({
            "locked": false,
            "error": e
        })))
    }
}
