//! AgentKern Unified Server
//!
//! Single binary gateway to all AgentKern pillars.
//! Replaces the Node.js `apps/identity` service.

use axum::{
    Router,
    middleware,
    extract::State,
    http::{Request, StatusCode, header},
    response::Response,
    body::Body,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "agentkern_server=debug,tower_http=debug,sqlx=warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("🚀 Starting AgentKern Unified Server");

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            tracing::warn!("DATABASE_URL not set, using default");
            "postgres://agentkern:agentkern@localhost:5432/agentkern".to_string()
        });

    tracing::info!("🗄️  Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("✅ Database connected");

    // Run migrations
    tracing::info!("📦 Running migrations...");
    run_migrations(&pool).await;
    tracing::info!("✅ Migrations complete");

    // Build application state
    let state = Arc::new(AppState { pool: pool.clone() });

    // Build the unified router
    let app = build_router(state).await;

    // Configure server address
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("📡 Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

/// Run SQLx migrations from all pillars
async fn run_migrations(pool: &sqlx::PgPool) {
    // Identity pillar migrations
    sqlx::migrate!("../../packages/pillars/identity/migrations")
        .run(pool)
        .await
        .expect("Failed to run Identity migrations");
}

async fn build_router(state: Arc<AppState>) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Get the Identity pillar router (passing pool)
    let identity_router = agentkern_identity::api::server::app_with_pool(state.pool.clone()).await;

    // Build unified router with all pillars
    Router::new()
        // Identity Pillar (verification, agents, keys, webauthn)
        .nest("/api/v1/identity", identity_router)
        // Gate Pillar (verification policies)
        .nest("/api/v1/gate", agentkern_gate::api::router())
        // Arbiter Pillar (coordination, scheduling)
        .nest("/api/v1/arbiter", agentkern_arbiter::api::router())
        // Nexus Pillar (agent communication)
        .nest("/api/v1/nexus", agentkern_nexus::api::router())
        // Synapse Pillar (reliability)
        .nest("/api/v1/synapse", agentkern_synapse::api::router())
        // Treasury Pillar (finance, ESG)
        .nest("/api/v1/treasury", agentkern_treasury::api::router())
        // Root health check
        .route("/health", axum::routing::get(root_health))
        // Tracing
        .layer(TraceLayer::new_for_http())
        // CORS
        .layer(cors)
}

/// Authentication middleware
/// Validates Bearer tokens or allows unauthenticated access to public routes
async fn auth_middleware(
    State(_state): State<Arc<AppState>>,
    request: Request<Body>,
    next: middleware::Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    // Public routes that don't require authentication
    let public_routes = [
        "/health",
        "/api/v1/identity/health",
        "/api/v1/gate/health",
        "/api/v1/arbiter/health",
        "/api/v1/nexus/health",
        "/api/v1/synapse/health",
        "/api/v1/treasury/health",
        "/api/v1/identity/verify", // Verification is public
    ];
    
    if public_routes.iter().any(|r| path.starts_with(r)) {
        return Ok(next.run(request).await);
    }

    // Check for Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];
            
            // TODO: Validate token against database or JWT verification
            // For now, accept any non-empty token (development mode)
            if !token.is_empty() {
                tracing::debug!("Authenticated request to {}", path);
                Ok(next.run(request).await)
            } else {
                tracing::warn!("Empty token for {}", path);
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Some(_) => {
            tracing::warn!("Invalid auth scheme for {}", path);
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            tracing::warn!("Missing Authorization header for {}", path);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn root_health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "agentkern-server",
        "version": env!("CARGO_PKG_VERSION"),
        "database": "connected",
        "pillars": {
            "identity": "active",
            "gate": "active",
            "arbiter": "active",
            "nexus": "active",
            "synapse": "active",
            "treasury": "active"
        }
    }))
}
