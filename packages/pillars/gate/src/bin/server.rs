//! AgentKern-Gate Server
//!
//! HTTP server for the Gate verification engine.
//! Uses Axum for high-performance HTTP handling.

use axum::error_handling::HandleErrorLayer;
use axum::{
    extract::State,
    http::{StatusCode, header::AUTHORIZATION},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;
use tower::{BoxError, ServiceBuilder, buffer::BufferLayer, limit::RateLimitLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use agentkern_gate::{GateEngine, Policy, VerificationResult};

/// Application state
struct AppState {
    engine: GateEngine,
    rate_limiter: Arc<agentkern_gate::RateLimiter>,
    auth: AuthConfig,
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

#[derive(Debug, Clone)]
struct AuthConfig {
    tokens: HashSet<String>,
}

impl AuthConfig {
    fn from_env() -> anyhow::Result<Self> {
        let is_production = runtime_is_production();
        let mut tokens = HashSet::new();

        if let Ok(raw) = std::env::var("GATE_AUTH_TOKENS") {
            for token in raw.split(',').map(str::trim).filter(|v| !v.is_empty()) {
                tokens.insert(token.to_string());
            }
        }

        if let Ok(single_token) = std::env::var("GATE_API_KEY") {
            let single_token = single_token.trim();
            if !single_token.is_empty() {
                tokens.insert(single_token.to_string());
            }
        }

        if tokens.is_empty() {
            if is_production {
                return Err(anyhow::anyhow!(
                    "Gate authentication is not configured in production. Set GATE_AUTH_TOKENS or GATE_API_KEY."
                ));
            }
            tracing::warn!(
                "⚠️ No Gate auth tokens configured in development; using default token 'agentkern-dev-token'"
            );
            tokens.insert("agentkern-dev-token".to_string());
        }

        tracing::info!("🔐 Gate auth initialized with {} token(s)", tokens.len());
        Ok(Self { tokens })
    }

    fn is_authorized(&self, auth_header: Option<&str>) -> bool {
        let Some(auth_header) = auth_header else {
            return false;
        };
        let Some((scheme, token)) = parse_auth_header(auth_header) else {
            return false;
        };
        if token.is_empty() {
            return false;
        }

        if !scheme.eq_ignore_ascii_case("bearer") && !scheme.eq_ignore_ascii_case("apikey") {
            return false;
        }

        self.tokens.contains(token)
    }
}

fn runtime_is_production() -> bool {
    let env_name = std::env::var("AGENTKERN_ENV")
        .or_else(|_| std::env::var("RUST_ENV"))
        .unwrap_or_else(|_| "development".to_string());
    matches!(env_name.to_lowercase().as_str(), "production" | "prod")
}

fn parse_auth_header(header: &str) -> Option<(&str, &str)> {
    let (scheme, token) = header.split_once(' ')?;
    Some((scheme.trim(), token.trim()))
}

fn anonymize_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let digest = hasher.finalize();
    format!("{:x}", digest)[..16].to_string()
}

fn request_identity_from_header(auth_header: Option<&str>) -> String {
    auth_header
        .and_then(parse_auth_header)
        .map(|(_, token)| format!("auth:{}", anonymize_token(token)))
        .unwrap_or_else(|| "anon".to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let auth = AuthConfig::from_env()?;

    // Initialize Cache & Rate Limiter (Phase 20)
    let redis_url = std::env::var("REDIS_URL").ok();
    if redis_url.is_some() {
        tracing::info!("🔌 Redis configured for distributed rate limiting");
    } else {
        tracing::warn!(
            "⚠️ No REDIS_URL found. Distributed rate limiting disabled (fallback to local)."
        );
    }

    let cache = agentkern_gate::CacheLayer::new(redis_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize cache layer: {}", e))?;

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
        auth,
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
        // Authentication Middleware
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        // Distributed Rate Limiting (Redis)
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

/// Authentication middleware.
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    // Skip auth for health check
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if state.auth.is_authorized(auth_header) {
        tracing::debug!("Authenticated Gate request");
        return Ok(next.run(req).await);
    }

    tracing::warn!("Unauthorized access attempt to {}", req.uri().path());
    Err(StatusCode::UNAUTHORIZED)
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

/// Distributed Rate Limiting Middleware.
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

    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    let key = request_identity_from_header(auth_header);

    let (allowed, remaining, error) = state.rate_limiter.check(&key).await;

    if error {
        tracing::warn!("Rate limiter error for identity {}", key);
        // Fail open is default in RateLimiter logic
    }

    if !allowed {
        tracing::warn!("Rate limit exceeded for identity: {}", key);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let mut response = next.run(req).await;
    if let Ok(header_value) = remaining.to_string().parse() {
        response
            .headers_mut()
            .insert("X-RateLimit-Remaining", header_value);
    }

    Ok(response)
}
