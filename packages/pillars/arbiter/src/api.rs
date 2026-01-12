use axum::{
    Router,
    routing::{get, post},
    Json,
    http::StatusCode,
};
use serde_json::{json, Value};

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/schedule", post(schedule_task))
        .route("/locks", post(acquire_lock))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "pillar": "arbiter",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn schedule_task(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    // TODO: Wire to Coordinator
    (StatusCode::CREATED, Json(json!({
        "scheduled": true,
        "id": "task-123",
        "input": payload
    })))
}

async fn acquire_lock(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    // TODO: Wire to LockManager
    (StatusCode::OK, Json(json!({
        "locked": true,
        "token": "lock-456",
        "input": payload
    })))
}
