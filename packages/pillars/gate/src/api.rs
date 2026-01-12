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
        .route("/verify", post(verify_policy))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "pillar": "gate",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn verify_policy(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    // TODO: Wire to GateEngine
    (StatusCode::OK, Json(json!({
        "allowed": true,
        "policy": "default",
        "input": payload
    })))
}
