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
        .route("/transfer", post(transfer))
        .route("/balance", get(get_balance))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "pillar": "treasury",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn transfer(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    // TODO: Wire to TransferEngine
    (StatusCode::OK, Json(json!({
        "transfer_id": "tx-999",
        "status": "pending",
        "input": payload
    })))
}

async fn get_balance() -> (StatusCode, Json<Value>) {
    // TODO: Wire to BalanceLedger
    (StatusCode::OK, Json(json!({
        "agent_id": "agent-a",
        "balance": 100.0,
        "currency": "USD"
    })))
}
