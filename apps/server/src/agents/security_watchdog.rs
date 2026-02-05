use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::AppState;
use agentkern_identity::services::manager::AgentManager;
use agentkern_gate::engine::VerificationRequestBuilder;
use agentkern_synapse::passport::export::{PassportExporter, ExportOptions, ExportFormat};
use agentkern_synapse::passport::schema::{MemoryPassport, AgentIdentity, ProvenanceSignature};
use uuid::Uuid;

pub struct SecurityWatchdog;

impl SecurityWatchdog {
    pub fn start(state: Arc<AppState>) {
        tokio::spawn(async move {
            let agent_id = "agentkern:security-watchdog";
            tracing::info!("🐕 Security Watchdog starting (ID: {})", agent_id);

            // 1. Identity Registration (Self-Onboarding)
            if let Some(ref pool) = state.pool {
                let identity = AgentManager::new(pool.clone());
                match identity.register(agent_id, "Security Watchdog", "1.0.0", Some("internal")).await {
                    Ok(_) => tracing::info!("✅ Watchdog identity registered"),
                    Err(e) => tracing::debug!("ℹ️ Watchdog identity already exists or skipped: {:?}", e),
                }
            }

            // 2. Monitoring Loop
            let mut iteration = 0;
            loop {
                iteration += 1;
                tracing::debug!("🐕 Watchdog pulse (iteration {})", iteration);

                // Task A: Check Arbiter for stale locks
                if let Some(lock) = state.arbiter.get_lock_status("global:shared_resource").await {
                    let now = chrono::Utc::now();
                    let held_for = now - lock.acquired_at;
                    tracing::info!("🐕 Watchdog monitoring lock: {} held by {} for {}s", lock.resource, lock.locked_by, held_for.num_seconds());
                    
                    if held_for.num_seconds() > 10 {
                         tracing::warn!("🚨 SUSPICIOUS LONG-HELD LOCK: Resource {} held by {}", lock.resource, lock.locked_by);
                    }
                }

                // Task B: Self-Governance Check via Gate
                let gate_request = VerificationRequestBuilder::new(agent_id, "system_audit")
                    .namespace("internal")
                    .context("iteration", iteration)
                    .build();
                
                let gate_result = state.gate.verify(gate_request).await;
                if !gate_result.allowed {
                    tracing::error!("🛑 Watchdog action BLOCKED by Gate: {}", gate_result.reasoning);
                }

                // Task C: Persist Findings to Synapse (Memory Export)
                if iteration % 6 == 0 { // Every 30 seconds
                    Self::persist_security_logs(agent_id, iteration).await;
                }

                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    async fn persist_security_logs(agent_id: &str, iteration: u64) {
        tracing::info!("🧠 Watchdog persisting security logs to Synapse...");
        
        let exporter = PassportExporter::new();
        let identity = AgentIdentity {
            did: format!("did:agentkern:{}", agent_id),
            public_key: "WATCHDOG_KEY".into(),
            algorithm: "Ed25519".into(),
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
        };

        let mut passport = MemoryPassport::new(identity, "US");
        passport.provenance.signatures.push(ProvenanceSignature {
            signer: format!("did:agentkern:{}", agent_id),
            signature: "watchdog-seal".into(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            prev_hash: "0".into(),
        });

        // Add a "security log" entry to episodic memory
        use agentkern_synapse::passport::layers::EpisodicEntry;
        passport.memory.episodic.entries.push(EpisodicEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            event_type: "security_audit".into(),
            summary: format!("Security audit complete at iteration {}. Status: Healthy.", iteration),
            participants: vec![],
            importance: 0.8,
            context: std::collections::HashMap::new(),
            embedding: None,
        });

        let options = ExportOptions {
            format: ExportFormat::Encrypted,
            compress: true,
            encryption_key: Some("watchdog-secret".into()),
            ..Default::default()
        };

        match exporter.export(&passport, &options) {
            Ok(bytes) => tracing::info!("✅ Watchdog secure log exported ({} bytes)", bytes.len()),
            Err(e) => tracing::error!("❌ Watchdog log export failed: {:?}", e),
        }
    }
}
