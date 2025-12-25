//! VeriMantle-Arbiter Server

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, delete},
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use verimantle_arbiter::{
    Coordinator,
    CoordinationRequest,
    CoordinationResult,
    BusinessLock,
    types::LockType,
};

struct AppState {
    coordinator: Coordinator,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Debug, Deserialize)]
struct CoordinateRequest {
    agent_id: String,
    resource: String,
    #[serde(default)]
    operation: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default)]
    expected_duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct LockRequest {
    agent_id: String,
    resource: String,
    #[serde(default)]
    priority: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct QueueQuery {
    agent_id: String,
    resource: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState {
        coordinator: Coordinator::new(),
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/coordinate", post(coordinate))
        .route("/lock", post(acquire_lock).delete(release_lock))
        .route("/lock/:resource", get(lock_status))
        .route("/queue", get(queue_position))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3003".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("⚖️ VeriMantle-Arbiter server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: "0.1.0",
    })
}

async fn coordinate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CoordinateRequest>,
) -> Json<CoordinationResult> {
    let operation = match req.operation.as_deref() {
        Some("read") => LockType::Read,
        Some("exclusive") => LockType::Exclusive,
        _ => LockType::Write,
    };

    let mut request = CoordinationRequest::new(req.agent_id, req.resource)
        .with_operation(operation);
    
    if let Some(p) = req.priority {
        request = request.with_priority(p);
    }
    if let Some(d) = req.expected_duration_ms {
        request = request.with_duration_ms(d);
    }

    Json(state.coordinator.request(request).await)
}

async fn acquire_lock(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LockRequest>,
) -> Result<Json<BusinessLock>, StatusCode> {
    state.coordinator
        .acquire_lock(&req.agent_id, &req.resource, req.priority.unwrap_or(0))
        .await
        .map(Json)
        .map_err(|_| StatusCode::CONFLICT)
}

async fn release_lock(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LockRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.coordinator
        .release_lock(&req.agent_id, &req.resource)
        .await
        .map(|_| Json(serde_json::json!({"released": true})))
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn lock_status(
    State(state): State<Arc<AppState>>,
    Path(resource): Path<String>,
) -> Result<Json<BusinessLock>, StatusCode> {
    state.coordinator
        .get_lock_status(&resource)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn queue_position(
    State(state): State<Arc<AppState>>,
    Query(query): Query<QueueQuery>,
) -> Json<serde_json::Value> {
    let position = state.coordinator
        .get_queue_position(&query.agent_id, &query.resource)
        .await;
    
    Json(serde_json::json!({
        "agent_id": query.agent_id,
        "resource": query.resource,
        "position": position
    }))
}
