#![deny(clippy::all)]

use napi_derive::napi;
use std::sync::OnceLock;

// Gate Pillar
use agentkern_gate::tee::Enclave;
use agentkern_gate::prompt_guard::PromptGuard;
use agentkern_gate::context_guard::ContextGuard;
use agentkern_gate::engine::{GateEngine, VerificationRequestBuilder};

// Treasury Pillar
use agentkern_treasury::{BalanceLedger, TransferEngine, TransferRequest, BudgetManager, CarbonLedger};

// Synapse Pillar
use agentkern_synapse::{StateStore, MemoryPassport, ContextGuard as SynapseContextGuard};

// Arbiter Pillar
use agentkern_arbiter::{KillSwitch, KillReason, LockManager, AuditLedger, ChaosMonkey};

// Static instances for zero-latency hot path (avoid re-initialization)
static PROMPT_GUARD: OnceLock<PromptGuard> = OnceLock::new();
static CONTEXT_GUARD: OnceLock<ContextGuard> = OnceLock::new();
static GATE_ENGINE: OnceLock<GateEngine> = OnceLock::new();
static BALANCE_LEDGER: OnceLock<BalanceLedger> = OnceLock::new();
static TRANSFER_ENGINE: OnceLock<TransferEngine> = OnceLock::new();
static BUDGET_MANAGER: OnceLock<BudgetManager> = OnceLock::new();
static CARBON_LEDGER: OnceLock<CarbonLedger> = OnceLock::new();
static STATE_STORE: OnceLock<StateStore> = OnceLock::new();
static KILL_SWITCH: OnceLock<KillSwitch> = OnceLock::new();
static LOCK_MANAGER: OnceLock<LockManager> = OnceLock::new();
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

fn get_balance_ledger() -> &'static BalanceLedger {
    BALANCE_LEDGER.get_or_init(BalanceLedger::new)
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

fn get_lock_manager() -> &'static LockManager {
    LOCK_MANAGER.get_or_init(LockManager::new)
}

fn get_audit_ledger() -> &'static AuditLedger {
    AUDIT_LEDGER.get_or_init(AuditLedger::new)
}

fn get_chaos_monkey() -> &'static ChaosMonkey {
    CHAOS_MONKEY.get_or_init(ChaosMonkey::new)
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
    match ledger.get(&agent_id) {
        Some(balance) => serde_json::to_string(&balance).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        None => format!("{{\"agent_id\": \"{}\", \"balance\": 0.0, \"currency\": \"USD\"}}", agent_id),
    }
}

/// Deposit to agent balance
#[napi]
pub fn treasury_deposit(agent_id: String, amount: f64) -> String {
    let ledger = get_balance_ledger();
    match ledger.deposit(&agent_id, amount) {
        Ok(balance) => serde_json::to_string(&balance).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

/// Transfer between agents
#[napi]
pub async fn treasury_transfer(from_agent: String, to_agent: String, amount: f64, reference: Option<String>) -> String {
    let engine = get_transfer_engine();
    let request = TransferRequest {
        from: from_agent,
        to: to_agent,
        amount,
        reference: reference.unwrap_or_default(),
    };
    
    match engine.transfer(request).await {
        Ok(result) => serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

/// Get agent budget
#[napi]
pub fn treasury_get_budget(agent_id: String) -> String {
    let manager = get_budget_manager();
    match manager.get(&agent_id) {
        Some(budget) => serde_json::to_string(&budget).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        None => format!("{{\"agent_id\": \"{}\", \"limit\": 100.0, \"spent\": 0.0, \"period\": \"daily\"}}", agent_id),
    }
}

/// Get carbon footprint
#[napi]
pub fn treasury_get_carbon(agent_id: String) -> String {
    let ledger = get_carbon_ledger();
    match ledger.get(&agent_id) {
        Some(footprint) => serde_json::to_string(&footprint).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        None => format!("{{\"agent_id\": \"{}\", \"total_grams_co2\": 0, \"compute_hours\": 0.0}}", agent_id),
    }
}

// ============================================================================
// Synapse Pillar Exports
// ============================================================================

/// Get agent state
#[napi]
pub fn synapse_get_state(agent_id: String) -> String {
    let store = get_state_store();
    match store.get(&agent_id) {
        Some(state) => serde_json::to_string(&state).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        None => format!("{{\"agent_id\": \"{}\", \"state\": {{}}, \"version\": 0}}", agent_id),
    }
}

/// Update agent state
#[napi]
pub fn synapse_update_state(agent_id: String, state_json: String) -> String {
    let store = get_state_store();
    match serde_json::from_str::<serde_json::Value>(&state_json) {
        Ok(state) => {
            match store.update(&agent_id, state) {
                Ok(updated) => serde_json::to_string(&updated).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            }
        }
        Err(e) => format!("{{\"error\": \"invalid_json: {}\"}}", e),
    }
}

/// Delete agent state
#[napi]
pub fn synapse_delete_state(agent_id: String) -> String {
    let store = get_state_store();
    store.delete(&agent_id);
    format!("{{\"deleted\": true, \"agent_id\": \"{}\"}}", agent_id)
}

// ============================================================================
// Arbiter Pillar Exports
// ============================================================================

/// Activate kill switch
#[napi]
pub fn arbiter_kill_switch_activate(reason: String, agent_id: Option<String>) -> String {
    let ks = get_kill_switch();
    let kill_reason = KillReason::Manual(reason);
    
    match ks.activate(kill_reason, agent_id.as_deref()) {
        Ok(record) => serde_json::to_string(&record).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

/// Get kill switch status
#[napi]
pub fn arbiter_kill_switch_status() -> String {
    let ks = get_kill_switch();
    let status = ks.status();
    serde_json::to_string(&status).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
}

/// Deactivate kill switch
#[napi]
pub fn arbiter_kill_switch_deactivate() -> String {
    let ks = get_kill_switch();
    ks.deactivate();
    "{\"active\": false}".to_string()
}

/// Acquire lock
#[napi]
pub async fn arbiter_acquire_lock(resource_id: String, agent_id: String, ttl_seconds: u64) -> String {
    let lm = get_lock_manager();
    match lm.acquire(&resource_id, &agent_id, std::time::Duration::from_secs(ttl_seconds)).await {
        Ok(lock) => serde_json::to_string(&lock).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

/// Release lock
#[napi]
pub fn arbiter_release_lock(resource_id: String) -> String {
    let lm = get_lock_manager();
    lm.release(&resource_id);
    format!("{{\"released\": true, \"resource_id\": \"{}\"}}", resource_id)
}

/// Query audit log
#[napi]
pub fn arbiter_query_audit(agent_id: Option<String>, limit: u32) -> String {
    let ledger = get_audit_ledger();
    let entries = ledger.query(agent_id.as_deref(), limit as usize);
    serde_json::to_string(&entries).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
}

/// Inject chaos
#[napi]
pub fn arbiter_inject_chaos(chaos_type: String, target: String, duration_seconds: u64) -> String {
    let monkey = get_chaos_monkey();
    match monkey.inject(&chaos_type, &target, std::time::Duration::from_secs(duration_seconds)) {
        Ok(result) => serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

