//! AgentKern-Gate Server
//!
//! HTTP server for the Gate verification engine.
//! Uses Axum for high-performance HTTP handling.

use axum::error_handling::HandleErrorLayer;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use agentkern_gate::{GateEngine, Policy, VerificationResult};

/// Application state
struct AppState {
    engine: GateEngine,
    rate_limiter: Arc<agentkern_gate::RateLimiter>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

#[derive(Debug, Deserialize)]
struct VerifyRequest {
    agent_id: String,
    action: String,
    #[serde(default)]
    context: std::collections::HashMap<String, serde_json::Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize Cache & Rate Limiter (Phase 20)
    let redis_url = std::env::var("REDIS_URL").ok();
    if redis_url.is_some() {
        tracing::info!("🔌 Connecting to Redis at {:?}", redis_url);
    } else {
        tracing::warn!(
            "⚠️ No REDIS_URL found. Distributed rate limiting disabled (fallback to local)."
        );
    }

    let cache = agentkern_gate::CacheLayer::new(redis_url)
        .await
        .expect("Failed to initialize cache layer");

    // Default distributed limit: 1000 requests per minute per key
    let dist_limit = std::env::var("DIST_RATE_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    let rate_limiter = Arc::new(agentkern_gate::RateLimiter::new(
        cache,
        dist_limit,
        std::time::Duration::from_secs(60),
    ));

    // Create engine
    let state = Arc::new(AppState {
        engine: GateEngine::new(),
        rate_limiter: rate_limiter.clone(),
    });

    let rate_limit = std::env::var("RATE_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/verify", post(verify))
        .route("/policies", get(list_policies).post(register_policy))
        .layer(TraceLayer::new_for_http())
        // P0: Rate Limiting Enforcement (100 RPM default)
        // Note: RateLimit requires Buffer to be cloneable for Axum,
        // and HandleErrorLayer to map errors to Infallible
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(
                    rate_limit,
                    std::time::Duration::from_secs(60),
                )),
        )
        // P2: Authentication Middleware (simple implementation)
        // P2: Authentication Middleware (simple implementation)
        .layer(axum::middleware::from_fn(auth_middleware))
        // P2: Distributed Rate Limiting (Redis)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            dist_rate_limit_middleware,
        ))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());

    let addr = format!("0.0.0.0:{}", port);

    tracing::info!(
        "🚀 AgentKern-Gate server running on http://{} (Rate Limit: {}/min)",
        addr,
        rate_limit
    );

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}

/// P2: Authentication Middleware
async fn auth_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    // Skip auth for health check
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") || auth.starts_with("ApiKey ") => {
            // In production: Validate JWT or check API key against DB/TEE
            // Here we accept any non-empty token for simulation
            let token = &auth[7..];
            if token.is_empty() {
                return Err(StatusCode::UNAUTHORIZED);
            }

            tracing::debug!("Authenticated request with token: [REDACTED]");
            Ok(next.run(req).await)
        }
        _ => {
            tracing::warn!("Unauthorized access attempt to {}", req.uri().path());
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: "0.1.0",
    })
}

async fn verify(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerificationResult>, StatusCode> {
    use agentkern_gate::engine::VerificationRequestBuilder;

    let mut builder = VerificationRequestBuilder::new(req.agent_id, req.action);
    for (key, value) in req.context {
        builder = builder.context(key, value);
    }

    let result = state.engine.verify(builder.build()).await;
    Ok(Json(result))
}

async fn list_policies(State(state): State<Arc<AppState>>) -> Json<Vec<Policy>> {
    let policies = state.engine.get_policies().await;
    Json(policies)
}

async fn register_policy(
    State(state): State<Arc<AppState>>,
    Json(policy): Json<Policy>,
) -> Result<Json<Policy>, StatusCode> {
    state.engine.register_policy(policy.clone()).await;
    Ok(Json(policy))
}

/// P2: Distributed Rate Limiting Middleware
/// Uses Redis to enforce limits across all instances.
async fn dist_rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    // Skip for health check
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    // Identify client: Try "X-Forwarded-For", then "Authorization" token hash, fallback to "unknown"
    let key = if let Some(auth) = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
    {
        format!("auth:{}", auth)
    } else {
        "anon".to_string()
    };

    let (allowed, remaining, error) = state.rate_limiter.check(&key).await;

    if error {
        tracing::warn!("Rate limiter error for key {}", key);
        // Fail open is default in RateLimiter logic
    }

    if !allowed {
        tracing::warn!("Rate limit exceeded for client: {}", key);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        remaining.to_string().parse().unwrap(),
    );

    Ok(response)
}
