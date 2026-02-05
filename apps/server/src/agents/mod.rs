use crate::AppState;
use std::sync::Arc;

pub mod rogue_agent;
pub mod security_watchdog;

/// Start all registered background agents
pub fn start_agents(state: Arc<AppState>) {
    tracing::info!("🤖 Initializing Resident Agents...");

    // Start Security Watchdog
    security_watchdog::SecurityWatchdog::start(state.clone());

    // Start Rogue Agent if Chaos Mode is enabled
    if std::env::var("CHAOS_MODE").unwrap_or_default() == "true" {
        rogue_agent::RogueAgent::start(state.clone());
    }
}
