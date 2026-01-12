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
        .route("/messages/send", post(send_message))
        .route("/agents/register", post(register_agent))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "pillar": "nexus",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn send_message(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    // TODO: Wire to MessageRouter
    (StatusCode::OK, Json(json!({
        "sent": true,
        "id": "msg-789",
        "input": payload
    })))
}

async fn register_agent(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    // TODO: Wire to AgentRegistry
    (StatusCode::CREATED, Json(json!({
        "registered": true,
        "input": payload
    })))
}
