use crate::AppState;
use crate::auth::Environment;
use agentkern_gate::engine::VerificationRequestBuilder;
use agentkern_identity::services::manager::AgentManager;
use agentkern_synapse::passport::export::{ExportFormat, ExportOptions, PassportExporter};
use agentkern_synapse::passport::schema::{AgentIdentity, MemoryPassport, ProvenanceSignature};
use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

pub struct SecurityWatchdog;

#[derive(Clone)]
struct WatchdogConfig {
    agent_id: String,
    public_key: String,
    signing_key: String,
    encryption_key: String,
}

impl WatchdogConfig {
    fn from_env() -> anyhow::Result<Self> {
        let is_production = Environment::from_env() == Environment::Production;
        let public_key =
            Self::read_or_generate("SECURITY_WATCHDOG_PUBLIC_KEY", is_production, || {
                format!("ephemeral:{}", Uuid::new_v4())
            })?;
        let signing_key =
            Self::read_or_generate("SECURITY_WATCHDOG_SIGNING_KEY", is_production, || {
                Uuid::new_v4().to_string()
            })?;
        let encryption_key =
            Self::read_or_generate("SECURITY_WATCHDOG_ENCRYPTION_KEY", is_production, || {
                Uuid::new_v4().to_string()
            })?;

        Ok(Self {
            agent_id: std::env::var("SECURITY_WATCHDOG_AGENT_ID")
                .unwrap_or_else(|_| "agentkern:security-watchdog".to_string()),
            public_key,
            signing_key,
            encryption_key,
        })
    }

    fn read_or_generate<F>(name: &str, strict: bool, fallback: F) -> anyhow::Result<String>
    where
        F: FnOnce() -> String,
    {
        match std::env::var(name) {
            Ok(value) if !value.trim().is_empty() => Ok(value.trim().to_string()),
            _ if strict => Err(anyhow::anyhow!(
                "{} must be set when ENABLE_SECURITY_WATCHDOG=true in production",
                name
            )),
            _ => {
                tracing::warn!("{name} not set; using ephemeral value");
                Ok(fallback())
            }
        }
    }
}

fn build_provenance_signature(agent_id: &str, iteration: u64, signing_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(signing_key.as_bytes());
    hasher.update(agent_id.as_bytes());
    hasher.update(iteration.to_be_bytes());
    hasher.update(chrono::Utc::now().timestamp_millis().to_be_bytes());
    STANDARD_NO_PAD.encode(hasher.finalize())
}

impl SecurityWatchdog {
    pub fn start(state: Arc<AppState>) {
        let config = match WatchdogConfig::from_env() {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("❌ Security Watchdog startup aborted: {}", e);
                return;
            }
        };

        tokio::spawn(async move {
            tracing::info!("🐕 Security Watchdog starting (ID: {})", config.agent_id);

            // 1. Identity Registration (Self-Onboarding)
            if let Some(ref pool) = state.pool {
                let identity = AgentManager::new(pool.clone());
                match identity
                    .register(
                        &config.agent_id,
                        "Security Watchdog",
                        "1.0.0",
                        Some("internal"),
                    )
                    .await
                {
                    Ok(_) => tracing::info!("✅ Watchdog identity registered"),
                    Err(e) => {
                        tracing::debug!("ℹ️ Watchdog identity already exists or skipped: {:?}", e)
                    }
                }
            }

            // 2. Monitoring Loop
            let mut iteration = 0;
            loop {
                iteration += 1;
                tracing::debug!("🐕 Watchdog pulse (iteration {})", iteration);

                // Task A: Check Arbiter for stale locks
                if let Some(lock) = state
                    .arbiter
                    .get_lock_status("global:shared_resource")
                    .await
                {
                    let now = chrono::Utc::now().timestamp();
                    let acquired_at = lock.acquired_at.timestamp();
                    let held_for_secs = now - acquired_at;

                    tracing::info!(
                        "🐕 Watchdog monitoring lock: {} held by {} for {}s",
                        lock.resource,
                        lock.locked_by,
                        held_for_secs
                    );

                    if held_for_secs > 10 {
                        tracing::warn!(
                            "🚨 SUSPICIOUS LONG-HELD LOCK: Resource {} held by {}",
                            lock.resource,
                            lock.locked_by
                        );
                    }
                }

                // Task B: Self-Governance Check via Gate
                let gate_request =
                    VerificationRequestBuilder::new(&config.agent_id, "system_audit")
                        .namespace("internal")
                        .context("iteration", iteration)
                        .build();

                let gate_result = state.gate.verify(gate_request).await;
                if !gate_result.allowed {
                    tracing::error!(
                        "🛑 Watchdog action BLOCKED by Gate: {}",
                        gate_result.reasoning
                    );
                }

                // Task C: Persist Findings to Synapse (Memory Export)
                if iteration % 6 == 0 {
                    // Every 30 seconds
                    Self::persist_security_logs(&config, iteration).await;
                }

                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    async fn persist_security_logs(config: &WatchdogConfig, iteration: u64) {
        tracing::info!("🧠 Watchdog persisting security logs to Synapse...");

        let exporter = PassportExporter::new();
        let identity = AgentIdentity {
            did: format!("did:agentkern:{}", config.agent_id),
            public_key: config.public_key.clone(),
            algorithm: "Ed25519".into(),
            created_at: chrono::Utc::now().timestamp_millis() as u64,
            updated_at: chrono::Utc::now().timestamp_millis() as u64,
        };

        let mut passport = MemoryPassport::new(identity, "US");
        passport.provenance.signatures.push(ProvenanceSignature {
            signer: format!("did:agentkern:{}", config.agent_id),
            signature: build_provenance_signature(&config.agent_id, iteration, &config.signing_key),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            prev_hash: "0".into(),
        });

        // Add a "security log" entry to episodic memory
        use agentkern_synapse::passport::layers::EpisodicEntry;
        passport.memory.episodic.entries.push(EpisodicEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            event_type: "security_audit".into(),
            summary: format!(
                "Security audit complete at iteration {}. Status: Healthy.",
                iteration
            ),
            participants: vec![],
            importance: 0.8,
            context: std::collections::HashMap::new(),
            embedding: None,
        });

        let options = ExportOptions {
            format: ExportFormat::Encrypted,
            compress: true,
            encryption_key: Some(config.encryption_key.clone()),
            ..Default::default()
        };

        match exporter.export(&passport, &options) {
            Ok(bytes) => tracing::info!("✅ Watchdog secure log exported ({} bytes)", bytes.len()),
            Err(e) => tracing::error!("❌ Watchdog log export failed: {:?}", e),
        }
    }
}
