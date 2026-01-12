//! AgentKern Unified Server
//!
//! Single binary gateway to all AgentKern pillars.
//! Replaces the Node.js `apps/identity` service.

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "agentkern_server=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    tracing::info!("🚀 Starting AgentKern Unified Server");

    // Build the unified router
    let app = build_router().await;

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

async fn build_router() -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Get the Identity pillar router
    let identity_router = agentkern_identity::api::server::app().await;

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
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

async fn root_health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "agentkern-server",
        "version": env!("CARGO_PKG_VERSION"),
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
