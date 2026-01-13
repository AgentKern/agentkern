//! AgentKern Unified Server
//!
//! Single binary gateway to all AgentKern pillars.
//! Replaces the Node.js `apps/identity` service.

use axum::{
    Router,
    middleware,
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;

use auth::JwtConfig;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub pool: Option<sqlx::PgPool>,
    pub jwt_config: JwtConfig,
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

    // JWT configuration - FAILS in production if misconfigured
    let jwt_config = match auth::JwtConfig::from_env() {
        Ok(config) => {
            let env_name = if config.is_production() { "PRODUCTION" } else { "development" };
            tracing::info!("🔐 JWT authentication enabled (env: {}, expiry: {}h)", env_name, config.expiration_hours);
            config
        }
        Err(e) => {
            tracing::error!("❌ JWT configuration error: {}", e);
            tracing::error!("Set JWT_SECRET environment variable (minimum 32 bytes)");
            std::process::exit(1);
        }
    };

    // Connect to database (optional - server can run without DB for testing)
    let database_url = std::env::var("DATABASE_URL").ok();
    
    let pool = if let Some(ref url) = database_url {
        tracing::info!("🗄️  Connecting to database...");
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
        {
            Ok(pool) => {
                tracing::info!("✅ Database connected");
                
                // Run migrations
                tracing::info!("📦 Running migrations...");
                run_migrations(&pool).await;
                tracing::info!("✅ Migrations complete");
                
                Some(pool)
            }
            Err(e) => {
                tracing::warn!("⚠️  Database connection failed: {}. Running in stateless mode.", e);
                None
            }
        }
    } else {
        tracing::warn!("⚠️  DATABASE_URL not set. Running in stateless mode.");
        None
    };

    // Build application state
    let state = Arc::new(AppState { 
        pool,
        jwt_config,
    });

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

    // Get the Identity pillar router (passing pool if available)
    let identity_router = if let Some(ref pool) = state.pool {
        agentkern_identity::api::server::app_with_pool(pool.clone()).await
    } else {
        agentkern_identity::api::server::app().await
    };

    // Auth routes (public)
    let auth_routes = Router::new()
        .route("/login", post(auth::login))
        .route("/token", post(auth::login)) // Alias
        .route("/refresh", post(auth::refresh_token))
        .route("/me", get(auth::me))
        .with_state(state.clone());

    // Build unified router with all pillars
    Router::new()
        // Auth routes (public)
        .nest("/api/v1/auth", auth_routes)
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
        .nest("/api/v1/treasury", agentkern_treasury::api::router(state.pool.clone()))
        // Root health check
        .route("/health", axum::routing::get(root_health))
        // Authentication middleware for protected routes
        .layer(middleware::from_fn_with_state(state.clone(), auth::auth_middleware))
        // Tracing
        .layer(TraceLayer::new_for_http())
        // CORS
        .layer(cors)
}

async fn root_health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "agentkern-server",
        "version": env!("CARGO_PKG_VERSION"),
        "auth": "jwt",
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
