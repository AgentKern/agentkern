//! AgentKern Unified Server
//!
//! HTTP Gateway that exposes all Six Pillars via REST API.
//!
//! Routes:
//! - /api/v1/identity → Identity pillar (packages/pillars/identity)
//! - /api/v1/gate → Gate pillar (packages/pillars/gate)
//! - /api/v1/synapse → Synapse pillar (packages/pillars/synapse)
//! - /api/v1/arbiter → Arbiter pillar (packages/pillars/arbiter)
//! - /api/v1/nexus → Nexus pillar (packages/pillars/nexus)
//! - /api/v1/treasury → Treasury pillar (packages/pillars/treasury)
//!
//! All pillars are Rust libraries. This server is a unified HTTP gateway.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::{Router, middleware, routing::post};
use futures_util::{sink::SinkExt, stream::StreamExt};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod agents;
mod auth;
mod chaos;
mod telemetry;

use auth::{Environment, JwtConfig};

const RUNTIME_EDITION_MODE: &str = "oss-core";
const ACTIVE_PILLARS: [&str; 5] = ["identity", "gate", "arbiter", "nexus", "synapse"];
const QUARANTINED_PILLARS: [&str; 1] = ["treasury"];
/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub pool: Option<sqlx::PgPool>,
    pub redis: Option<redis::Client>,
    pub jwt_config: JwtConfig,
    pub gate: Arc<agentkern_gate::engine::GateEngine>,
    pub arbiter: Arc<agentkern_arbiter::Coordinator>,
    pub tx: broadcast::Sender<agentkern_gate::DashboardEvent>,
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

    // Validate required environment variables before startup
    validate_required_environment_variables();

    // JWT configuration - FAILS in production if misconfigured
    let jwt_config = match auth::JwtConfig::from_env().await {
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

    let pool: Option<sqlx::PgPool> = if let Some(ref url) = database_url {
        tracing::info!("🗄️  Connecting to database...");
        match PgPoolOptions::new().max_connections(10).connect(url).await {
            Ok(pool) => {
                tracing::info!("✅ Database connected");
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

    // Connect to Redis (optional - required for production revocation)
    let redis_url = std::env::var("REDIS_URL").ok();
    let redis = if let Some(ref url) = redis_url {
        tracing::info!("📡 Connecting to Redis...");
        match redis::Client::open(url.as_str()) {
            Ok(client) => {
                tracing::info!("✅ Redis client initialized");
                Some(client)
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️  Redis initialization failed: {}. Revocation will be in-memory.",
                    e
                );
                None
            }
        }
    } else {
        tracing::warn!("⚠️  REDIS_URL not set. Revocation will be in-memory.");
        None
    };

    // Initialize Core Engines for sharing
    let gate = Arc::new(
        agentkern_gate::engine::GateEngine::new()
            .with_jurisdiction(agentkern_gate::types::DataRegion::Global),
    );

    let arbiter = if let Some(ref p) = pool {
        agentkern_arbiter::api::init_coordinator_with_pool(p.clone())
    } else {
        agentkern_arbiter::Coordinator::new().map(Arc::new)
    };
    let arbiter = match arbiter {
        Ok(coordinator) => coordinator,
        Err(e) => {
            tracing::error!("❌ Failed to initialize Arbiter coordinator: {}", e);
            std::process::exit(1);
        }
    };

    let (tx, _) = broadcast::channel(1024);

    // Build application state
    let state = Arc::new(AppState {
        pool,
        redis,
        jwt_config,
        gate: gate.clone(),
        arbiter: arbiter.clone(),
        tx: tx.clone(),
    });

    tracing::info!(
        "🧭 Runtime edition mode: {} (active: {:?}, quarantined: {:?})",
        RUNTIME_EDITION_MODE,
        ACTIVE_PILLARS,
        QUARANTINED_PILLARS
    );

    // Start background activity monitor
    tokio::spawn(monitor_gate_activity(gate.clone(), tx.clone()));

    // Start Resident Agents
    agents::start_agents(state.clone());

    // Build the unified router
    let app = match build_router(state).await {
        Ok(app) => app,
        Err(e) => {
            tracing::error!("❌ Failed to build router: {}", e);
            std::process::exit(1);
        }
    };

    // Configure server address
    let port_raw = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = match port_raw.parse() {
        Ok(value) => value,
        Err(e) => {
            tracing::error!("❌ Invalid PORT '{}': {}", port_raw, e);
            std::process::exit(1);
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("📡 Listening on {}", addr);

    // Start server
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("❌ Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("❌ Server failed to start: {}", e);
        std::process::exit(1);
    }
}

async fn build_router(state: Arc<AppState>) -> anyhow::Result<Router> {
    // CORS configuration
    let allowed_origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
    let is_production = Environment::from_env() == Environment::Production;

    let cors = if allowed_origins == "*" {
        if is_production {
            return Err(anyhow::anyhow!(
                "ALLOWED_ORIGINS must be explicitly configured in production"
            ));
        }
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let origins: Vec<axum::http::HeaderValue> = allowed_origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(|origin| {
                origin
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid CORS origin: {}", origin))
            })
            .collect::<Result<_, _>>()?;

        if origins.is_empty() {
            return Err(anyhow::anyhow!(
                "ALLOWED_ORIGINS is set but contains no valid origins"
            ));
        }

        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Get the Identity pillar router (passing pool if available)
    let identity_router: Router<()> = if let Some(ref pool) = state.pool {
        agentkern_identity::api::server::app_with_pool(pool.clone()).await
    } else {
        agentkern_identity::api::server::app().await
    };

    let auth_routes = Router::new()
        .route("/login", post(auth::login))
        .route("/token", post(auth::login)) // Alias
        .route("/refresh", post(auth::refresh_token))
        .route("/logout", post(auth::logout)) // ← NEW: Revokes token
        .with_state(state.clone());

    // Admin routes (protected)
    let admin_routes = Router::new()
        .route("/hash-secret", post(auth::admin_hash_secret))
        .route("/revoke-token", post(auth::admin_revoke))
        .with_state(state.clone());
    // Build unified router with all pillars
    // Explicitly declare Router<()> to catch type mismatches
    Ok(Router::<()>::new()
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
            resilient_service(
                agentkern_gate::api::router_with_engine(state.gate.clone()),
                100,
                10,
            ),
        )
        // Arbiter Pillar
        .nest_service(
            "/api/v1/arbiter",
            resilient_service(
                agentkern_arbiter::api::router(state.arbiter.clone(), None, state.pool.clone()),
                50,
                60,
            ),
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
        // =====================================================================
        // Treasury Pillar - QUARANTINED (Enterprise Edition Feature)
        // =====================================================================
        // The Treasury pillar is reserved for AgentKern Enterprise Edition.
        // It provides:
        // - Atomic payments and budget management
        // - Carbon footprint tracking and offsetting
        // - Multi-currency support and financial compliance
        //
        // OSS builds: this endpoint is disabled (404).
        // EE builds: see docs/ENTERPRISE_SETUP.md for enablement.
        //
        // To enable locally, uncomment the block below and build with the EE overlay.
        /*
        .nest_service(
            "/api/v1/treasury",
            resilient_service(agentkern_treasury::api::router(state.pool.clone()), 50, 30),
        )
        */
        // Admin Auth Endpoints (Protected)
        .nest("/api/v1/admin", admin_routes)
        // Root health check
        .route("/health", axum::routing::get(root_health))
        // WebSocket Activity Feed (Live Dashboard) - requires state
        .merge(
            Router::new()
                .route(
                    "/api/v1/gate/ws/activity",
                    axum::routing::get(ws_activity_handler),
                )
                .with_state(state.clone()),
        )
        // Authentication middleware for protected routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
        // Tracing
        .layer(TraceLayer::new_for_http())
        // CORS
        .layer(cors))
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
    axum::Json(health_payload())
}

fn health_payload() -> serde_json::Value {
    serde_json::json!({
        "status": "ok",
        "service": "agentkern-server",
        "version": env!("CARGO_PKG_VERSION"),
        "edition_mode": RUNTIME_EDITION_MODE,
        "auth": "jwt",
        "active_pillars": ACTIVE_PILLARS,
        "quarantined_pillars": QUARANTINED_PILLARS,
        "pillars": {
            "identity": "active",
            "gate": "active",
            "arbiter": "active",
            "nexus": "active",
            "synapse": "active",
            "treasury": "quarantined"
        }
    })
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
/// Validate required environment variables at startup
/// Fail-fast principle: exit if critical configs are missing
fn validate_required_environment_variables() {
    let is_production = Environment::from_env() == Environment::Production;

    // Always required
    if std::env::var("JWT_SECRET").is_err() {
        if is_production {
            tracing::error!("❌ CRITICAL: JWT_SECRET not set in PRODUCTION");
            std::process::exit(1);
        } else {
            tracing::warn!("⚠️  JWT_SECRET not set (required in production)");
        }
    }

    // Validate JWT_SECRET length in production
    if is_production
        && let Ok(secret) = std::env::var("JWT_SECRET")
        && secret.len() < 32
    {
        tracing::error!("❌ CRITICAL: JWT_SECRET must be at least 32 bytes in PRODUCTION");
        std::process::exit(1);
    }

    // Warn if DATABASE_URL not set (optional but recommended)
    if std::env::var("DATABASE_URL").is_err() {
        if is_production {
            tracing::warn!("⚠️  DATABASE_URL not set in PRODUCTION (running stateless)");
        } else {
            tracing::info!("ℹ️  DATABASE_URL not set (running in stateless mode)");
        }
    }

    // Validate PORT if specified
    if let Ok(port_str) = std::env::var("PORT")
        && port_str.parse::<u16>().is_err()
    {
        tracing::error!(
            "❌ PORT must be a valid u16 number (0-65535), got: {}",
            port_str
        );
        std::process::exit(1);
    }

    // Log startup validation complete
    if is_production {
        tracing::info!("✅ Production environment variables validated");
    }
}
/// Start background task to pipe GateEngine events to the Dashboard broadcast channel
async fn monitor_gate_activity(
    gate: Arc<agentkern_gate::GateEngine>,
    tx: broadcast::Sender<agentkern_gate::DashboardEvent>,
) {
    let mut rx = gate.subscribe();
    tracing::info!("📡 Background Gate Activity Monitor started");

    while let Ok(event) = rx.recv().await {
        let dashboard_event = agentkern_gate::DashboardEvent::Verification(event);
        if let Err(e) = tx.send(dashboard_event) {
            tracing::debug!("No active dashboard listeners: {}", e);
        }
    }
}

/// WebSocket handler for the live dashboard feed
async fn ws_activity_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut _receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    tracing::info!("🔌 New Dashboard WebSocket client connected");

    while let Ok(event) = rx.recv().await {
        let msg = match serde_json::to_string(&event) {
            Ok(json) => Message::Text(json.into()),
            Err(e) => {
                tracing::error!("Failed to serialize dashboard event: {}", e);
                continue;
            }
        };

        if let Err(e) = sender.send(msg).await {
            tracing::debug!("Dashboard WebSocket client disconnected: {}", e);
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_payload_contract_is_stable() {
        let payload = health_payload();

        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["service"], "agentkern-server");
        assert_eq!(payload["edition_mode"], RUNTIME_EDITION_MODE);
        assert_eq!(payload["auth"], "jwt");
        assert_eq!(payload["pillars"]["identity"], "active");
        assert_eq!(payload["pillars"]["gate"], "active");
        assert_eq!(payload["pillars"]["arbiter"], "active");
        assert_eq!(payload["pillars"]["nexus"], "active");
        assert_eq!(payload["pillars"]["synapse"], "active");
        assert_eq!(payload["pillars"]["treasury"], "quarantined");

        let active_pillars = payload["active_pillars"]
            .as_array()
            .expect("active_pillars must be an array");
        assert!(active_pillars.iter().any(|v| v == "identity"));
        assert!(active_pillars.iter().any(|v| v == "gate"));
        assert!(active_pillars.iter().any(|v| v == "arbiter"));
        assert!(active_pillars.iter().any(|v| v == "nexus"));
        assert!(active_pillars.iter().any(|v| v == "synapse"));

        let quarantined_pillars = payload["quarantined_pillars"]
            .as_array()
            .expect("quarantined_pillars must be an array");
        assert_eq!(quarantined_pillars.len(), 1);
        assert_eq!(quarantined_pillars[0], "treasury");
    }
}
