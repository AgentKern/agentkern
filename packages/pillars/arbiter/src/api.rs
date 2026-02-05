use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{CoordinationRequest, Coordinator, PgCoordinator};
use agentkern_pulse::Pulse;

use ::openraft::raft::{AppendEntriesRequest, InstallSnapshotRequest, VoteRequest};
// use ::openraft::error::{RaftError, InstallSnapshotError};
use crate::storage::TypeConfig;

/// Arbiter App State (in-memory)
#[derive(Clone)]
pub struct ArbiterState {
    pub coordinator: Arc<Coordinator>,
    pub raft: Option<Arc<crate::RaftLockManager>>,
}

/// Arbiter App State (Postgres-backed, distributed)
#[derive(Clone)]
pub struct PgArbiterState {
    pub coordinator: Arc<PgCoordinator>,
    pub raft: Option<Arc<crate::RaftLockManager>>,
}

/// Create router without database (in-memory, for development/testing)
pub fn router(
    coordinator: Arc<Coordinator>,
    raft_manager: Option<Arc<crate::RaftLockManager>>,
    _pool: Option<::sqlx::PgPool>,
) -> Router {
    let state = ArbiterState {
        coordinator,
        raft: raft_manager,
    };

    Router::new()
        .route("/health", get(health_check))
        .route("/schedule", post(schedule_task))
        .route("/locks", post(acquire_lock_endpoint))
        // Raft RPCs
        .route("/raft/init", post(raft_init))
        .route("/raft/append", post(raft_append))
        .route("/raft/vote", post(raft_vote))
        .route("/raft/snapshot", post(raft_snapshot))
        .with_state(state)
}

/// Create router with database (Postgres-backed, for production)
pub fn router_with_pool(pool: PgPool) -> Router {
    let coordinator = Arc::new(Coordinator::new()); // Standard coordinator
    router(coordinator, Some(pool))
}

pub fn init_coordinator_with_pool(_pool: PgPool) -> Arc<Coordinator> {
    // Current Coordinator handles PG via internal managers if needed
    // or we use the PgCoordinator. For now, let's keep it simple.
    Arc::new(Coordinator::new())
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

    if let Some(_op) = payload["operation"].as_str() {
        request.operation = crate::types::LockType::Write; // Simplify for now
    }

    if let Some(priority) = payload["priority"].as_i64() {
        request.priority = priority as i32;
    }

    let result = state.coordinator.request(request).await;

    if result.granted {
        (
            StatusCode::CREATED,
            Json(json!({
                "scheduled": true,
                "lock_id": result.lock.map(|l| l.id),
                "status": "granted"
            })),
        )
    } else if let Some(pos) = result.queue_position {
        (
            StatusCode::ACCEPTED,
            Json(json!({
                "scheduled": false,
                "status": "queued",
                "position": pos,
                "wait_ms": result.estimated_wait_ms
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "scheduled": false,
                "status": "denied",
                "reason": result.reason
            })),
        )
    }
}

async fn acquire_lock_endpoint(
    State(state): State<ArbiterState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let agent_id = payload["agent_id"].as_str().unwrap_or("unknown");
    let resource = payload["resource"].as_str().unwrap_or("global");
    let priority = payload["priority"].as_i64().unwrap_or(0) as i32;

    match state
        .coordinator
        .acquire_lock(agent_id, resource, priority)
        .await
    {
        Ok(lock) => (
            StatusCode::OK,
            Json(json!({
                "locked": true,
                "lock_id": lock.id
            })),
        ),
        Err(e) => (
            StatusCode::CONFLICT,
            Json(json!({
                "locked": false,
                "error": e
            })),
        ),
    }
}

// ============================================================================
// Postgres-backed handlers (for PgCoordinator)
// ============================================================================

async fn pg_health_check(State(state): State<PgArbiterState>) -> Json<Value> {
    let report = state.coordinator.get_health().await;

    Json(json!({
        "status": "ok",
        "pillar": "arbiter",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "distributed",
        "report": report
    }))
}

async fn pg_schedule_task(
    State(state): State<PgArbiterState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let agent_id = payload["agent_id"].as_str().unwrap_or("unknown");
    let resource = payload["resource"].as_str().unwrap_or("global");

    let mut request = CoordinationRequest::new(agent_id, resource);

    if let Some(_op) = payload["operation"].as_str() {
        request.operation = crate::types::LockType::Write;
    }

    if let Some(priority) = payload["priority"].as_i64() {
        request.priority = priority as i32;
    }

    let result = state.coordinator.request(request).await;

    if result.granted {
        (
            StatusCode::CREATED,
            Json(json!({
                "scheduled": true,
                "lock_id": result.lock.map(|l| l.id),
                "status": "granted",
                "persistent": true
            })),
        )
    } else if let Some(pos) = result.queue_position {
        (
            StatusCode::ACCEPTED,
            Json(json!({
                "scheduled": false,
                "status": "queued",
                "position": pos,
                "wait_ms": result.estimated_wait_ms,
                "persistent": true
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "scheduled": false,
                "status": "denied",
                "reason": result.reason
            })),
        )
    }
}

async fn pg_acquire_lock_endpoint(
    State(state): State<PgArbiterState>,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let agent_id = payload["agent_id"].as_str().unwrap_or("unknown");
    let resource = payload["resource"].as_str().unwrap_or("global");
    let priority = payload["priority"].as_i64().unwrap_or(0) as i32;

    match state
        .coordinator
        .acquire_lock(agent_id, resource, priority)
        .await
    {
        Ok(lock) => (
            StatusCode::OK,
            Json(json!({
                "locked": true,
                "lock_id": lock.id,
                "persistent": true
            })),
        ),
        Err(e) => (
            StatusCode::CONFLICT,
            Json(json!({
                "locked": false,
                "error": e
            })),
        ),
    }
}

// Raft RPC Handlers

async fn raft_append(
    State(state): State<ArbiterState>,
    Json(rpc): Json<AppendEntriesRequest<TypeConfig>>,
) -> impl axum::response::IntoResponse {
    if let Some(raft_manager) = &state.raft {
        let res = raft_manager.raft.append_entries(rpc).await;
        (StatusCode::OK, Json(json!(res)))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Raft not initialized"})))
    }
}

async fn raft_vote(
    State(state): State<ArbiterState>,
    Json(rpc): Json<VoteRequest<u64>>,
) -> impl axum::response::IntoResponse {
    if let Some(raft_manager) = &state.raft {
        let res = raft_manager.raft.vote(rpc).await;
        (StatusCode::OK, Json(json!(res)))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Raft not initialized"})))
    }
}

async fn raft_snapshot(
    State(state): State<ArbiterState>,
    Json(rpc): Json<InstallSnapshotRequest<TypeConfig>>,
) -> impl axum::response::IntoResponse {
    if let Some(raft_manager) = &state.raft {
        let res = raft_manager.raft.install_snapshot(rpc).await;
        (StatusCode::OK, Json(json!(res)))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Raft not initialized"})))
    }
}

async fn raft_init(
    State(state): State<ArbiterState>,
    Json(nodes_req): Json<std::collections::BTreeMap<u64, Value>>,
) -> impl axum::response::IntoResponse {
    use axum::response::IntoResponse;
    if let Some(raft_manager) = &state.raft {
        // Check if already initialized
        let metrics = raft_manager.raft.metrics().borrow().clone();
        if metrics.last_log_index.is_some() {
            return (StatusCode::CONFLICT, Json(json!({"error": "Raft already initialized"}))).into_response();
        }

        // Validate peer count (Arbiter standard: max 7 voters for low latency)
        if nodes_req.is_empty() {
             return (StatusCode::BAD_REQUEST, Json(json!({"error": "Node list cannot be empty"}))).into_response();
        }
        if nodes_req.len() > 7 {
             return (StatusCode::BAD_REQUEST, Json(json!({"error": "Arbiter cluster size restricted to 7 nodes for performance"}))).into_response();
        }

        let nodes: std::collections::BTreeMap<u64, ()> = nodes_req.into_iter().map(|(k, _)| (k, ())).collect();
        let res = raft_manager.raft.initialize(nodes).await;
        match res {
            Ok(_) => (StatusCode::OK, Json(json!({"ok": true}))).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
        }
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Raft not initialized"}))).into_response()
    }
}
