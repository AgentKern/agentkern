//! Treasury Server Binary

use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("AgentKern-Treasury starting...");

    let app = Router::new().route("/health", get(|| async { "OK" }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3003));
    tracing::info!("Treasury listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    tracing::info!("✅ Treasury server started successfully");

    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
