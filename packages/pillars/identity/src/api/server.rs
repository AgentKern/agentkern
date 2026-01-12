use axum::{
    Router,
    routing::{get, post},
    Json,
};
use serde_json::json;

pub async fn app() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/verify", post(verify_endpoint))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "pillar": "identity",
        "implementation": "rust"
    }))
}

async fn verify_endpoint() -> Json<serde_json::Value> {
    // Placeholder for actual verification logic usage
    Json(json!({ "valid": false, "error": "Not implemented" }))
}
