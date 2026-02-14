use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

use crate::db::{PgTransferEngine, TransferRequest};
use crate::types::Amount;

/// Treasury App State
#[derive(Clone)]
pub struct TreasuryState {
    pub engine: Option<Arc<PgTransferEngine>>,
}

pub fn router(pool: Option<PgPool>) -> Router {
    let engine = pool.map(|p| Arc::new(PgTransferEngine::new(p)));
    let state = TreasuryState { engine };

    Router::new()
        .route("/health", get(health_check))
        .route("/transfer", post(transfer))
        .route("/balance/{id}", get(get_balance))
        .with_state(state)
}

async fn health_check(State(state): State<TreasuryState>) -> Json<Value> {
    let db_status = if state.engine.is_some() {
        "connected"
    } else {
        "disconnected"
    };
    Json(json!({
        "status": "ok",
        "pillar": "treasury",
        "version": env!("CARGO_PKG_VERSION"),
        "database": db_status
    }))
}

async fn transfer(
    State(state): State<TreasuryState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let engine = match &state.engine {
        Some(e) => e,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "Database not connected" })),
            );
        }
    };

    // Parse payload into TransferRequest
    // Note: In a real app we'd use a DTO struct with Json<TransferDto>
    // For now manually parsing to keep flexibility during migration
    let from = payload["from"].as_str().unwrap_or_default();
    let to = payload["to"].as_str().unwrap_or_default();
    let amount_float = payload["amount"].as_f64().unwrap_or(0.0);
    // Assuming 6 decimals for VMC
    let amount = Amount::from_float(amount_float, 6);
    let reference = payload["reference"].as_str().map(String::from);
    let idempotency_key = payload["idempotency_key"].as_str().map(String::from);

    let request = TransferRequest {
        from: from.to_string(),
        to: to.to_string(),
        amount,
        reference,
        idempotency_key,
    };

    let result = engine.transfer(request).await;

    if result.status == crate::db::TransferStatus::Completed {
        (
            StatusCode::OK,
            Json(json!({
                "transaction_id": result.transaction_id,
                "status": "completed",
                "timestamp": result.timestamp
            })),
        )
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "transaction_id": result.transaction_id,
                "status": "failed",
                "error": result.error
            })),
        )
    }
}

async fn get_balance(
    State(state): State<TreasuryState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> (StatusCode, Json<Value>) {
    let engine = match &state.engine {
        Some(e) => e,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "Database not connected" })),
            );
        }
    };

    match engine.get_balance(&id).await {
        Ok(balance_micros) => {
            let amount = Amount::new(balance_micros, 6);
            (
                StatusCode::OK,
                Json(json!({
                    "agent_id": id,
                    "balance": amount.to_float(),
                    "currency": "VMC"
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
