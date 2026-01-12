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
        .route("/memory/store", post(store_memory))
        .route("/memory/query", post(query_memory))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "pillar": "synapse",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn store_memory(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    // TODO: Wire to MemoryStore (Vector DB)
    (StatusCode::CREATED, Json(json!({
        "stored": true,
        "id": "mem-101",
        "input": payload
    })))
}

async fn query_memory(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    // TODO: Wire to RAG Engine
    (StatusCode::OK, Json(json!({
        "results": [],
        "query": payload
    })))
}
