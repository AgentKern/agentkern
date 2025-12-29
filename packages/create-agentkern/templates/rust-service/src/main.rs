//! {{PROJECT_NAME}} - AgentKern Service
//!
//! A Zero-Trust Rust service with mTLS and Gate integration.

use agentkern_gate::{GateEngine, MtlsConfig, ObservabilityPlane};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Starting {{PROJECT_NAME}}");

    // Initialize observability
    let observability = ObservabilityPlane::new();
    info!("📊 Observability plane initialized");

    // Configure mTLS (Zero-Trust default)
    let mtls_config = MtlsConfig::new()
        .require_client_certs(true)
        .verify_chain(true);
    info!("🔐 mTLS configured: {:?}", mtls_config);

    // Initialize Gate engine
    let engine = GateEngine::default();
    info!("⚙️ Gate engine initialized");

    // Your agent logic here
    info!("✅ {{PROJECT_NAME}} ready");

    // Keep running
    tokio::signal::ctrl_c().await?;
    info!("👋 Shutting down");

    Ok(())
}
