//! AgentKern-Arbiter Server

use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use agentkern_arbiter::Coordinator;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let node_id: u64 = std::env::var("NODE_ID")
        .unwrap_or_else(|_| "1".to_string())
        .parse()
        .expect("NODE_ID must be a u64");

    let port = std::env::var("PORT").unwrap_or_else(|_| "3003".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let storage_path =
        std::env::var("STORAGE_PATH").unwrap_or_else(|_| format!("/tmp/raft-node-{}", node_id));

    tracing::info!("🚀 Starting Arbiter Node {} on {}", node_id, addr);

    let raft_manager = Arc::new(
        agentkern_arbiter::RaftLockManager::new(node_id, addr.clone(), storage_path).await,
    );

    // Register peers from PEERS env var (format: 1=127.0.0.1:3001,2=127.0.0.1:3002)
    if let Ok(peers_str) = std::env::var("PEERS") {
        for peer in peers_str.split(',') {
            if let Some((id_str, addr_str)) = peer.split_once('=') {
                if let Ok(id) = id_str.parse::<u64>() {
                    if id != node_id {
                        raft_manager.network.register_node(id, addr_str.to_string());
                        tracing::info!("Registered peer {} at {}", id, addr_str);
                    }
                }
            }
        }
    }

    let coordinator = {
        #[allow(unused_mut)]
        let mut coordinator = Coordinator::new();

        // Enterprise Feature Wiring
        #[cfg(feature = "ee")]
        {
            tracing::info!("🏢 Initializing Enterprise Edition Features...");

            // 1. Carbon Grid API (Real-time ESG)
            match agentkern_energy_ee::GridFactory::get() {
                api => {
                    coordinator = coordinator.with_grid_api(Arc::new(api));
                    tracing::info!("✅ EE: Real-time Carbon Grid API enabled");
                }
            }

            // 2. Escalation Connectors (Slack, Teams, PagerDuty)
            if let Ok(token) = std::env::var("SLACK_BOT_TOKEN") {
                let config = agentkern_escalation_ee::SlackConfig {
                    bot_token: token,
                    app_token: std::env::var("SLACK_APP_TOKEN").ok(),
                    signing_secret: std::env::var("SLACK_SIGNING_SECRET").unwrap_or_default(),
                    default_channel: std::env::var("SLACK_DEFAULT_CHANNEL")
                        .unwrap_or("#alerts".into()),
                };
                if let Ok(slack) = agentkern_escalation_ee::SlackIntegration::new(config) {
                    coordinator.add_escalation_connector(Arc::new(slack)).await;
                    tracing::info!("✅ EE: Slack Escalation enabled");
                }
            }
        }
        coordinator
    };

    let coordinator = Arc::new(coordinator);

    let app = agentkern_arbiter::api::router(coordinator, Some(raft_manager), None)
        .layer(TraceLayer::new_for_http());

    tracing::info!("⚖️ AgentKern-Arbiter server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
