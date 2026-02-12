use crate::AppState;
use std::sync::Arc;

pub mod rogue_agent;
pub mod security_watchdog;

/// Start all registered background agents
pub fn start_agents(state: Arc<AppState>) {
    tracing::info!("🤖 Initializing Resident Agents...");

    // Start Security Watchdog only when explicitly enabled.
    let watchdog_enabled = std::env::var("ENABLE_SECURITY_WATCHDOG")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if watchdog_enabled {
        security_watchdog::SecurityWatchdog::start(state.clone());
    } else {
        tracing::info!("🐕 Security Watchdog disabled (set ENABLE_SECURITY_WATCHDOG=true)");
    }

    // Start Rogue Agent if Chaos Mode is enabled
    if std::env::var("CHAOS_MODE").unwrap_or_default() == "true" {
        rogue_agent::RogueAgent::start(state.clone());
    }
}
