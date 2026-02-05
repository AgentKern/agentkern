use crate::AppState;
use agentkern_gate::engine::VerificationRequestBuilder;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct RogueAgent;

impl RogueAgent {
    pub fn start(state: Arc<AppState>) {
        tokio::spawn(async move {
            let agent_id = "agentkern:rogue-agent";
            tracing::error!("👺 ROGUE AGENT STARTING (ID: {})", agent_id);

            sleep(Duration::from_secs(10)).await;

            // SCENARIO 1: Stale Lock Injection
            tracing::warn!("👹 Rogue Agent: Injecting stale lock on 'global:shared_resource'...");
            // We use a VERY short duration (1s) but then don't release it.
            match state
                .arbiter
                .acquire_lock(agent_id, "global:shared_resource", 50)
                .await
            {
                Ok(lock) => {
                    tracing::warn!(
                        "👺 Rogue Agent acquired lock {}. It will NOT be released.",
                        lock.id
                    );
                }
                Err(e) => tracing::error!("👺 Rogue Agent failed to acquire lock: {}", e),
            }

            // Keep the rogue agent alive to prevent auto-release (if we implemented it)
            loop {
                // SCENARIO 2: Boundary Violation periodically
                let gate_request =
                    VerificationRequestBuilder::new(agent_id, "unauthorized_system_access").build();

                let _ = state.gate.verify(gate_request).await;

                sleep(Duration::from_secs(15)).await;
            }
        });
    }
}
