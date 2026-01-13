//! AgentKern Unified Server
//!
//! Single binary gateway to all AgentKern pillars.
//! Replaces the Node.js `apps/identity` service.

use axum::{
    middleware,
    routing::{post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod chaos;
mod telemetry;

use auth::JwtConfig;
/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub pool: Option<sqlx::PgPool>,
    pub jwt_config: JwtConfig,
}

#[tokio::main]
async fn main() {
    // Initialize Telemetry (OpenTelemetry + Tracing)
    if let Err(e) = telemetry::init_telemetry() {
        eprintln!("Failed to initialize telemetry: {}", e);
    }

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("🚀 Starting AgentKern Unified Server");

    // JWT configuration - FAILS in production if misconfigured
    let jwt_config = match auth::JwtConfig::from_env() {
        Ok(config) => {
            let env_name = if config.is_production() {
                "PRODUCTION"
            } else {
                "development"
            };
            tracing::info!(
                "🔐 JWT authentication enabled (env: {}, expiry: {}h)",
                env_name,
                config.expiration_hours
            );
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
        match PgPoolOptions::new().max_connections(10).connect(url).await {
            Ok(pool) => {
                tracing::info!("✅ Database connected");

                // Run migrations
                tracing::info!("📦 Running migrations...");
                run_migrations(&pool).await;
                tracing::info!("✅ Migrations complete");

                Some(pool)
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️  Database connection failed: {}. Running in stateless mode.",
                    e
                );
                None
            }
        }
    } else {
        tracing::warn!("⚠️  DATABASE_URL not set. Running in stateless mode.");
        None
    };

    // Build application state
    let state = Arc::new(AppState { pool, jwt_config });

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
    let mut migrator = sqlx::migrate!("../../packages/pillars/identity/migrations");
    migrator.set_ignore_missing(true)
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
    let identity_router: Router<()> = if let Some(ref pool) = state.pool {
        agentkern_identity::api::server::app_with_pool(pool.clone()).await
    } else {
        agentkern_identity::api::server::app().await
    };

    // Auth routes (public)
    let auth_routes = Router::new()
        .route("/login", post(auth::login))
        .route("/token", post(auth::login)) // Alias
        .route("/refresh", post(auth::refresh_token))
        .with_state(state.clone());
    // Build unified router with all pillars
    // Explicitly declare Router<()> to catch type mismatches
    Router::<()>::new()
        // Auth routes (public)
        .nest_service("/api/v1/auth", auth_routes)
        // Identity Pillar
        .nest_service(
            "/api/v1/identity",
            resilient_service(identity_router, 100, 30),
        )
        // Gate Pillar
        .nest_service(
            "/api/v1/gate",
            resilient_service(agentkern_gate::api::router(), 100, 10),
        )
        // Arbiter Pillar
        .nest_service(
            "/api/v1/arbiter",
            resilient_service(agentkern_arbiter::api::router(), 50, 60),
        )
        // Nexus Pillar
        .nest_service(
            "/api/v1/nexus",
            resilient_service(agentkern_nexus::api::router(), 200, 30),
        )
        // Synapse Pillar
        .nest_service(
            "/api/v1/synapse",
            resilient_service(agentkern_synapse::api::router(), 100, 5),
        )
        // Treasury Pillar
        .nest_service(
            "/api/v1/treasury",
            resilient_service(agentkern_treasury::api::router(state.pool.clone()), 50, 30),
        )
        // Root health check
        .route("/health", axum::routing::get(root_health))
        // Authentication middleware for protected routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
        // Tracing
        .layer(TraceLayer::new_for_http())
        // CORS
        .layer(cors)
}

fn resilient_service(
    router: Router<()>,
    concurrency: usize,
    timeout_secs: u64,
) -> impl tower::Service<
    axum::http::Request<axum::body::Body>,
    Response = axum::response::Response,
    Error = std::convert::Infallible,
    Future = impl Send,
> + Clone
       + Send
       + Sync {
    tower::ServiceBuilder::new()
        .layer(axum::error_handling::HandleErrorLayer::new(
            handle_middleware_error,
        ))
        .layer(tower::limit::ConcurrencyLimitLayer::new(concurrency))
        .layer(tower::timeout::TimeoutLayer::new(
            std::time::Duration::from_secs(timeout_secs),
        ))
        .service(router)
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

/// Handle errors from middleware (Timeout, ConcurrencyLimit)
async fn handle_middleware_error(
    err: axum::BoxError,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            axum::http::StatusCode::GATEWAY_TIMEOUT,
            axum::Json(serde_json::json!({
                "error": "Request timed out",
                "status": "timeout"
            })),
        )
    } else {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({
                "error": format!("Service unavailable: {}", err),
                "status": "overloaded"
            })),
        )
    }
}
