use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

use crate::models::VerificationKey;
use crate::services::{AgentManager, AuditService, VerificationService};

/// Application state with database pool and services
pub struct AppState {
    pub verifier: VerificationService,
    pub pool: Option<PgPool>,
    pub agent_manager: Option<AgentManager>,
    pub audit_service: Option<AuditService>,
}

/// Create router without database (for testing)
pub async fn app() -> Router {
    let verifier = VerificationService::new();
    let state = Arc::new(AppState {
        verifier,
        pool: None,
        agent_manager: None,
        audit_service: None,
    });
    build_router(state)
}

/// Create router with database pool (production)
pub async fn app_with_pool(pool: PgPool) -> Router {
    let verifier = VerificationService::new();
    let agent_manager = AgentManager::new(pool.clone());
    let audit_service = AuditService::new(pool.clone());

    let state = Arc::new(AppState {
        verifier,
        pool: Some(pool),
        agent_manager: Some(agent_manager),
        audit_service: Some(audit_service),
    });
    build_router(state)
}

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health
        .route("/health", get(health_check))
        // Proof Verification
        .route("/verify", post(verify_endpoint))
        // Agent Management
        .route("/agents", get(list_agents))
        .route("/agents", post(create_agent))
        .route("/agents/{id}", get(get_agent))
        .route("/agents/{id}", delete(delete_agent))
        // Key Management
        .route("/keys", post(register_key))
        .route("/keys/{id}", delete(revoke_key))
        // Trust Service (Reputation)
        .route("/reputation/{id}", get(get_reputation))
        .route("/reputation/{id}/success", post(report_success))
        .route("/reputation/{id}/failure", post(report_failure))
        // Compliance Service (Audit)
        .route("/compliance/audit", post(log_audit_event))
        .with_state(state)
}

// ============================================================================
// Health
// ============================================================================

async fn health_check(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db_status = if state.pool.is_some() {
        "connected"
    } else {
        "none"
    };
    Json(json!({
        "status": "ok",
        "pillar": "identity",
        "implementation": "rust",
        "database": db_status,
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// ============================================================================
// Proof Verification
// ============================================================================

#[derive(Deserialize)]
struct VerifyRequest {
    proof_header: String,
    public_key: Option<VerificationKey>,
}

async fn verify_endpoint(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyRequest>,
) -> (StatusCode, Json<Value>) {
    let proof_res = state.verifier.parse_header(&payload.proof_header);
    match proof_res {
        Ok(proof) => {
            if let Some(key) = payload.public_key {
                match state.verifier.verify(&proof, &key).await {
                    Ok(valid) => (StatusCode::OK, Json(json!({ "valid": valid }))),
                    Err(e) => (
                        StatusCode::BAD_REQUEST,
                        Json(json!({ "valid": false, "error": e.to_string() })),
                    ),
                }
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "valid": false, "error": "Missing key for simulation" })),
                )
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "valid": false, "error": e.to_string() })),
        ),
    }
}

// ============================================================================
// Agent Management
// ============================================================================

#[derive(Deserialize)]
struct CreateAgentRequest {
    id: String,
    name: String,
    version: String,
    namespace: Option<String>,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct AgentResponse {
    id: String,
    name: String,
    status: String,
    namespace: String,
}

async fn list_agents(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    if let Some(ref manager) = state.agent_manager {
        match manager.list(None).await {
            Ok(agents) => {
                let response: Vec<_> = agents
                    .iter()
                    .map(|a| {
                        json!({
                            "id": a.id,
                            "name": a.name,
                            "status": format!("{:?}", a.status),
                            "namespace": a.namespace
                        })
                    })
                    .collect();
                (StatusCode::OK, Json(json!({ "agents": response })))
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::OK,
            Json(json!({ "agents": [], "note": "No database connected" })),
        )
    }
}

async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAgentRequest>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref manager) = state.agent_manager {
        match manager
            .register(
                &payload.id,
                &payload.name,
                &payload.version,
                payload.namespace.as_deref(),
            )
            .await
        {
            Ok(agent) => (
                StatusCode::CREATED,
                Json(json!({
                    "id": agent.id,
                    "name": agent.name,
                    "status": format!("{:?}", agent.status),
                    "namespace": agent.namespace
                })),
            ),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::CREATED,
            Json(json!({
                "id": payload.id,
                "name": payload.name,
                "status": "active",
                "note": "No database connected"
            })),
        )
    }
}

async fn get_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref manager) = state.agent_manager {
        match manager.get(&id).await {
            Ok(agent) => (
                StatusCode::OK,
                Json(json!({
                    "id": agent.id,
                    "name": agent.name,
                    "status": format!("{:?}", agent.status),
                    "namespace": agent.namespace,
                    "reputation": agent.reputation
                })),
            ),
            Err(e) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::OK,
            Json(json!({ "id": id, "note": "No database connected" })),
        )
    }
}

async fn delete_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref manager) = state.agent_manager {
        match manager.delete(&id).await {
            Ok(()) => (StatusCode::OK, Json(json!({ "deleted": id }))),
            Err(e) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::OK,
            Json(json!({ "deleted": id, "note": "No database connected" })),
        )
    }
}

// ============================================================================
// Trust Service (Reputation)
// ============================================================================

async fn get_reputation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref manager) = state.agent_manager {
        match manager.get(&id).await {
            Ok(agent) => (
                StatusCode::OK,
                Json(json!({
                    "id": agent.id,
                    "score": agent.reputation.score,
                    "level": agent.reputation.trust_level
                })),
            ),
            Err(e) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::OK,
            Json(json!({ "id": id, "score": 50, "level": "neutral" })),
        )
    }
}

async fn report_success(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref manager) = state.agent_manager {
        match manager.record_success(&id, 0).await {
            Ok(()) => (StatusCode::OK, Json(json!({ "id": id, "change": "+1" }))),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::OK,
            Json(json!({ "id": id, "score": 51, "change": "+1" })),
        )
    }
}

async fn report_failure(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref manager) = state.agent_manager {
        match manager.record_failure(&id).await {
            Ok(()) => (StatusCode::OK, Json(json!({ "id": id, "change": "-10" }))),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::OK,
            Json(json!({ "id": id, "score": 40, "change": "-10" })),
        )
    }
}

// ============================================================================
// Compliance Service (Audit)
// ============================================================================

#[derive(Deserialize)]
struct AuditRequest {
    event_type: String,
    action: String,
    outcome: String,
    actor_id: Option<String>,
    target_id: Option<String>,
}

async fn log_audit_event(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuditRequest>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref audit) = state.audit_service {
        match audit
            .log(
                &payload.event_type,
                payload.actor_id.as_deref(),
                None, // actor_type
                payload.target_id.as_deref(),
                None, // target_type
                &payload.action,
                &payload.outcome,
                None, // details
                None, // ip_address
            )
            .await
        {
            Ok(id) => (
                StatusCode::CREATED,
                Json(json!({ "logged": true, "id": id.to_string() })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::CREATED,
            Json(json!({ "logged": true, "note": "No database connected" })),
        )
    }
}

// ============================================================================
// Key Management
// ============================================================================

#[derive(Deserialize)]
struct RegisterKeyRequest {
    principal_id: String,
    credential_id: String,
    public_key: String,
    algorithm: Option<String>,
}

async fn register_key(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterKeyRequest>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref pool) = state.pool {
        // Insert key into database
        let result = sqlx::query(
            r#"
            INSERT INTO verification_keys (id, principal_id, algorithm, public_key_pem, created_at, last_used_at, active)
            VALUES ($1, $2, $3, $4, NOW(), NOW(), true)
            "#
        )
        .bind(&payload.credential_id)
        .bind(&payload.principal_id)
        .bind(payload.algorithm.as_deref().unwrap_or("Ed25519"))
        .bind(&payload.public_key)
        .execute(pool)
        .await;

        match result {
            Ok(_) => (
                StatusCode::CREATED,
                Json(json!({
                    "credential_id": payload.credential_id,
                    "principal_id": payload.principal_id,
                    "active": true
                })),
            ),
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::CREATED,
            Json(json!({
                "credential_id": payload.credential_id,
                "principal_id": payload.principal_id,
                "active": true,
                "note": "No database connected"
            })),
        )
    }
}

async fn revoke_key(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    if let Some(ref pool) = state.pool {
        let result = sqlx::query("UPDATE verification_keys SET active = false WHERE id = $1")
            .bind(&id)
            .execute(pool)
            .await;

        match result {
            Ok(r) if r.rows_affected() > 0 => (StatusCode::OK, Json(json!({ "revoked": id }))),
            Ok(_) => (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Key not found" })),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
        }
    } else {
        (
            StatusCode::OK,
            Json(json!({ "revoked": id, "note": "No database connected" })),
        )
    }
}
