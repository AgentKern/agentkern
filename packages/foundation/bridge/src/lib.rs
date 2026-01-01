#![deny(clippy::all)]
#![allow(unused_imports)]

use napi_derive::napi;
use std::sync::OnceLock;
use std::sync::Arc;

// Gate Pillar
use agentkern_gate::tee::Enclave;
use agentkern_gate::prompt_guard::PromptGuard;
use agentkern_gate::context_guard::ContextGuard;
use agentkern_gate::engine::{GateEngine, VerificationRequestBuilder};

// Treasury Pillar
use agentkern_treasury::{BalanceLedger, TransferEngine, TransferRequest, BudgetManager, CarbonLedger, Currency, Amount};

// Synapse Pillar  
use agentkern_synapse::{StateStore, StateUpdate};

// Arbiter Pillar
use agentkern_arbiter::{KillSwitch, KillReason, TerminationType, AuditLedger};
use agentkern_arbiter::chaos::{ChaosMonkey, ChaosConfig, ChaosStats};

// Static instances for zero-latency hot path (avoid re-initialization)
static PROMPT_GUARD: OnceLock<PromptGuard> = OnceLock::new();
static CONTEXT_GUARD: OnceLock<ContextGuard> = OnceLock::new();
static GATE_ENGINE: OnceLock<GateEngine> = OnceLock::new();
static BALANCE_LEDGER: OnceLock<Arc<BalanceLedger>> = OnceLock::new();
static TRANSFER_ENGINE: OnceLock<TransferEngine> = OnceLock::new();
static BUDGET_MANAGER: OnceLock<BudgetManager> = OnceLock::new();
static CARBON_LEDGER: OnceLock<CarbonLedger> = OnceLock::new();
static STATE_STORE: OnceLock<StateStore> = OnceLock::new();
static KILL_SWITCH: OnceLock<KillSwitch> = OnceLock::new();
static AUDIT_LEDGER: OnceLock<AuditLedger> = OnceLock::new();
static CHAOS_MONKEY: OnceLock<ChaosMonkey> = OnceLock::new();

fn get_prompt_guard() -> &'static PromptGuard {
    PROMPT_GUARD.get_or_init(PromptGuard::new)
}

fn get_context_guard() -> &'static ContextGuard {
    CONTEXT_GUARD.get_or_init(ContextGuard::default)
}

fn get_gate_engine() -> &'static GateEngine {
    GATE_ENGINE.get_or_init(GateEngine::new)
}

fn get_balance_ledger() -> &'static Arc<BalanceLedger> {
    BALANCE_LEDGER.get_or_init(|| Arc::new(BalanceLedger::new(Currency::VMC)))
}

fn get_transfer_engine() -> &'static TransferEngine {
    TRANSFER_ENGINE.get_or_init(|| TransferEngine::new(get_balance_ledger().clone()))
}

fn get_budget_manager() -> &'static BudgetManager {
    BUDGET_MANAGER.get_or_init(BudgetManager::new)
}

fn get_carbon_ledger() -> &'static CarbonLedger {
    CARBON_LEDGER.get_or_init(CarbonLedger::new)
}

fn get_state_store() -> &'static StateStore {
    STATE_STORE.get_or_init(StateStore::new)
}

fn get_kill_switch() -> &'static KillSwitch {
    KILL_SWITCH.get_or_init(KillSwitch::new)
}

fn get_audit_ledger() -> &'static AuditLedger {
    AUDIT_LEDGER.get_or_init(AuditLedger::new)
}

fn get_chaos_monkey() -> &'static ChaosMonkey {
    CHAOS_MONKEY.get_or_init(|| ChaosMonkey::new(ChaosConfig::default()))
}

// ============================================================================
// Gate Pillar Exports
// ============================================================================

#[napi]
pub fn attest(nonce: String) -> String {
    let enclave = if std::path::Path::new("/dev/tdx-guest").exists() || std::path::Path::new("/dev/sev-guest").exists() {
         Enclave::new("agentkern-gateway").unwrap_or_else(|_| Enclave::simulated("fallback-sim"))
    } else {
         Enclave::simulated("sim-gateway")
    };

    match enclave.attest(nonce.as_bytes()) {
        Ok(attestation) => attestation.to_json(),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

/// Prompt Injection Guard (Hot Path: 0ms)
#[napi]
pub fn guard_prompt(prompt: String) -> String {
    let guard = get_prompt_guard();
    let analysis = guard.analyze(&prompt);
    serde_json::to_string(&analysis).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
}

/// RAG Context Guard (Hot Path: 0ms)
#[napi]
pub fn guard_context(chunks: Vec<String>) -> String {
    let guard = get_context_guard();
    let result = guard.scan(&chunks);
    serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
}

/// Gate Engine Verification (Hot Path: 0ms)
#[napi]
pub async fn verify(agent_id: String, action: String, context_json: Option<String>) -> String {
    let engine = get_gate_engine();

    let mut builder = VerificationRequestBuilder::new(agent_id, action);

    if let Some(ctx_str) = context_json {
        if let Ok(ctx_map) = serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(&ctx_str) {
            for (k, v) in ctx_map {
                builder = builder.context(k, v);
            }
        }
    }

    let request = builder.build();
    let result = engine.verify(request).await;
    
    serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
}

// ============================================================================
// Treasury Pillar Exports
// ============================================================================

/// Get agent balance
#[napi]
pub fn treasury_get_balance(agent_id: String) -> String {
    let ledger = get_balance_ledger();
    let balance = ledger.get_balance(&agent_id);
    serde_json::to_string(&balance).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
}

/// Deposit to agent balance
#[napi]
pub fn treasury_deposit(agent_id: String, amount: f64) -> String {
    let ledger = get_balance_ledger();
    let amt = Amount::from_float(amount, 6); // VMC has 6 decimals
    match ledger.deposit(&agent_id, amt) {
        Ok(balance) => serde_json::to_string(&balance).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

/// Transfer between agents
#[napi]
pub async fn treasury_transfer(from_agent: String, to_agent: String, amount: f64, reference: Option<String>) -> String {
    let engine = get_transfer_engine();
    let amt = Amount::from_float(amount, 6);
    let mut request = TransferRequest::new(&from_agent, &to_agent, amt);
    
    if let Some(ref_str) = reference {
        request = request.with_reference(ref_str);
    }
    
    let result = engine.transfer(request).await;
    serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
}

/// Get agent budget remaining
#[napi]
pub fn treasury_get_budget(agent_id: String) -> String {
    let manager = get_budget_manager();
    match manager.get_remaining(&agent_id) {
        Some(remaining) => format!("{{\"agent_id\": \"{}\", \"remaining\": {}}}", agent_id, remaining.to_float()),
        None => format!("{{\"agent_id\": \"{}\", \"remaining\": null, \"message\": \"No budget set\"}}", agent_id),
    }
}

/// Get carbon footprint
#[napi]
pub fn treasury_get_carbon(agent_id: String) -> String {
    let ledger = get_carbon_ledger();
    let usage = ledger.get_daily_usage(&agent_id);
    serde_json::to_string(&usage).unwrap_or_else(|_| format!("{{\"agent_id\": \"{}\", \"total_grams_co2\": 0}}", agent_id))
}

// ============================================================================
// Synapse Pillar Exports
// ============================================================================

/// Get agent state
#[napi]
pub async fn synapse_get_state(agent_id: String) -> String {
    let store = get_state_store();
    match store.get_state(&agent_id).await {
        Some(state) => serde_json::to_string(&state).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        None => format!("{{\"agent_id\": \"{}\", \"state\": {{}}, \"version\": 0}}", agent_id),
    }
}

/// Update agent state
#[napi]
pub async fn synapse_update_state(agent_id: String, state_json: String) -> String {
    let store = get_state_store();
    match serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(&state_json) {
        Ok(updates) => {
            let update = StateUpdate {
                agent_id: agent_id.clone(),
                updates,
                deletes: None,
            };
            let result = store.update_state(update).await;
            serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
        }
        Err(e) => format!("{{\"error\": \"invalid_json: {}\"}}", e),
    }
}

// ============================================================================
// Arbiter Pillar Exports
// ============================================================================

/// Activate kill switch (terminate agent)
#[napi]
pub async fn arbiter_kill_switch_activate(reason: String, agent_id: Option<String>) -> String {
    let ks = get_kill_switch();
    
    if let Some(aid) = agent_id {
        let record = ks.terminate_agent(
            &aid,
            KillReason::Custom(reason),
            TerminationType::Graceful,
            None,
        ).await;
        serde_json::to_string(&record).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
    } else {
        let record = ks.emergency_shutdown(Some(reason)).await;
        serde_json::to_string(&record).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
    }
}

/// Get kill switch status
#[napi]
pub async fn arbiter_kill_switch_status() -> String {
    let ks = get_kill_switch();
    let is_emergency = ks.is_emergency().await;
    let terminated_count = ks.terminated_count().await;
    format!("{{\"active\": {}, \"terminated_count\": {}}}", is_emergency, terminated_count)
}

/// Deactivate kill switch
#[napi]
pub async fn arbiter_kill_switch_deactivate() -> String {
    let ks = get_kill_switch();
    ks.lift_emergency().await;
    "{\"active\": false}".to_string()
}

/// Query audit statistics
#[napi]
pub async fn arbiter_query_audit(_limit: u32) -> String {
    let ledger = get_audit_ledger();
    let stats = ledger.get_statistics().await;
    serde_json::to_string(&stats).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
}

/// Get chaos statistics
#[napi]
pub fn arbiter_chaos_stats() -> String {
    let monkey = get_chaos_monkey();
    let stats = monkey.stats();
    format!(
        "{{\"total_ops\": {}, \"latency_injections\": {}, \"error_injections\": {}}}",
        stats.total_ops, stats.latency_injections, stats.error_injections
    )
}
