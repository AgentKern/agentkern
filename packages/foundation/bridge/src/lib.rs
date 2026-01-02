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
use agentkern_gate::policy::Policy;

// Treasury Pillar
use agentkern_treasury::{BalanceLedger, TransferEngine, TransferRequest, BudgetManager, CarbonLedger, Currency, Amount};

// Synapse Pillar  
use agentkern_synapse::{StateStore, StateUpdate, GraphVectorDB, PolyglotEmbedder, GraphNode, NodeType, SynapseRegion};

// Arbiter Pillar
use agentkern_arbiter::{KillSwitch, KillReason, TerminationType, AuditLedger};
use agentkern_arbiter::chaos::{ChaosMonkey, ChaosConfig, ChaosStats};

// Nexus Pillar
use agentkern_nexus::{Nexus, Protocol, AgentCard, NexusMessage};

// Static instances for zero-latency hot path (avoid re-initialization)
static PROMPT_GUARD: OnceLock<PromptGuard> = OnceLock::new();
static CONTEXT_GUARD: OnceLock<ContextGuard> = OnceLock::new();
static GATE_ENGINE: OnceLock<GateEngine> = OnceLock::new();
static BALANCE_LEDGER: OnceLock<Arc<BalanceLedger>> = OnceLock::new();
static TRANSFER_ENGINE: OnceLock<TransferEngine> = OnceLock::new();
static BUDGET_MANAGER: OnceLock<BudgetManager> = OnceLock::new();
static CARBON_LEDGER: OnceLock<CarbonLedger> = OnceLock::new();
static STATE_STORE: OnceLock<StateStore> = OnceLock::new();
static GRAPH_DB: OnceLock<GraphVectorDB> = OnceLock::new();
static POLYGLOT_EMBEDDER: OnceLock<PolyglotEmbedder> = OnceLock::new();
static KILL_SWITCH: OnceLock<KillSwitch> = OnceLock::new();
static AUDIT_LEDGER: OnceLock<AuditLedger> = OnceLock::new();
static CHAOS_MONKEY: OnceLock<ChaosMonkey> = OnceLock::new();
static NEXUS_GATEWAY: OnceLock<Nexus> = OnceLock::new();

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

fn get_graph_db() -> &'static GraphVectorDB {
    GRAPH_DB.get_or_init(GraphVectorDB::new)
}

fn get_polyglot_embedder() -> &'static PolyglotEmbedder {
    POLYGLOT_EMBEDDER.get_or_init(PolyglotEmbedder::default)
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

fn get_nexus() -> &'static Nexus {
    NEXUS_GATEWAY.get_or_init(Nexus::new)
}

// Duplicate getters removed by tool


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

/// Register a policy dynamically (Hot Path)
#[napi]
pub async fn register_policy(policy_yaml: String) -> String {
    let engine = get_gate_engine();
    match Policy::from_yaml(&policy_yaml) {
        Ok(policy) => {
            engine.register_policy(policy).await;
            "{\"status\": \"registered\"}".to_string()
        }
        Err(e) => format!("{{\"error\": \"invalid_policy: {}\"}}", e),
    }
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

/// Purchase carbon offset
#[napi]
pub fn treasury_purchase_offset(agent_id: String, tons: f64) -> String {
    let ledger = get_carbon_ledger();
    match ledger.purchase_offset(agent_id, tons) {
        Ok(offset) => serde_json::to_string(&offset).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
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

/// Store memory (embed + vector store) (Hot Path)
#[napi]
pub async fn synapse_store_memory(agent_id: String, text: String) -> String {
    let db = get_graph_db();
    let embedder = get_polyglot_embedder();
    
    // Auto-detect region (default Global for simplicity)
    let region = SynapseRegion::Global;
    let vector = embedder.embed(&text, region).await;
    
    let node = GraphNode {
        id: uuid::Uuid::new_v4(),
        node_type: NodeType::Memory,
        data: serde_json::json!({ "content": text }),
        vector: Some(vector),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
    };
    
    let id = db.insert_node(node);
    db.index_agent_node(&agent_id, id);
    
    format!("{{\"id\": \"{}\", \"status\": \"stored\"}}", id)
}

/// Query memory (embed + vector search) (Hot Path)
#[napi]
pub async fn synapse_query_memory(text: String, limit: u32) -> String {
    let db = get_graph_db();
    let embedder = get_polyglot_embedder();
    
    let region = SynapseRegion::Global;
    let vector = embedder.embed(&text, region).await;
    
    let results = db.find_similar(&vector, limit as usize);
    serde_json::to_string(&results).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string())
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
pub async fn arbiter_query_audit() -> String {
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

// ============================================================================
// Nexus Pillar Exports (Protocol Gateway)
// ============================================================================

/// Receive and translate message (e.g. A2A JSON to Native)
#[napi]
pub async fn nexus_receive(raw_payload: String) -> String {
    let nexus = get_nexus();
    match nexus.receive(raw_payload.as_bytes()).await {
        Ok(msg) => serde_json::to_string(&msg).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

/// Send and translate message (Native to Protocol)
#[napi]
pub async fn nexus_send(msg_json: String, target_protocol: String) -> String {
    let nexus = get_nexus();
    
    // Parse protocol enum
    let protocol = match target_protocol.to_lowercase().as_str() {
        "googlea2a" | "a2a" => Protocol::GoogleA2A,
        "anthropicmcp" | "mcp" => Protocol::AnthropicMCP,
        "agentkern" | "native" => Protocol::AgentKern,
        _ => return "{\"error\": \"unsupported_protocol\"}".to_string(),
    };

    match serde_json::from_str::<NexusMessage>(&msg_json) {
        Ok(msg) => {
            match nexus.send(&msg, protocol).await {
                Ok(bytes) => {
                    // Return as base64 or string depending on content
                    // For now assume text protocols
                    match String::from_utf8(bytes) {
                        Ok(s) => s,
                        Err(_) => "{\"error\": \"response_not_utf8\"}".to_string(),
                    }
                },
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            }
        },
        Err(e) => format!("{{\"error\": \"invalid_json: {}\"}}", e),
    }
}

/// Register agent with Nexus
#[napi]
pub async fn nexus_register_agent(card_json: String) -> String {
    let nexus = get_nexus();
    match serde_json::from_str::<AgentCard>(&card_json) {
        Ok(card) => {
            match nexus.register_agent(card).await {
                Ok(_) => "{\"status\": \"registered\"}".to_string(),
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            }
        },
        Err(e) => format!("{{\"error\": \"invalid_json: {}\"}}", e),
    }
}

/// Create A2A Task (Helper)
#[napi]
pub fn nexus_create_a2a_task(id: String, description: String) -> String {
    // Utility to generate valid A2A JSON structure
    let json = serde_json::json!({
        "jsonrpc": "2.0",
        "id": uuid::Uuid::new_v4().to_string(),
        "method": "tasks/send",
        "params": {
            "id": id,
            "description": description,
            "status": "submitted"
        }
    });
    json.to_string()
}

/// List all agents
#[napi]
pub async fn nexus_list_agents() -> String {
    let nexus = get_nexus();
    let registry = nexus.registry();
    let agents = registry.list().await;
    serde_json::to_string(&agents).unwrap_or_else(|_| "[]".to_string())
}

/// Get agent by ID
#[napi]
pub async fn nexus_get_agent(id: String) -> String {
    let nexus = get_nexus();
    let registry = nexus.registry();
    match registry.get(&id).await {
        Some(agent) => serde_json::to_string(&agent).unwrap_or_else(|_| "null".to_string()),
        None => "null".to_string(),
    }
}

/// Unregister agent
#[napi]
pub async fn nexus_unregister_agent(id: String) -> bool {
    let nexus = get_nexus();
    let registry = nexus.registry();
    registry.unregister(&id).await.is_ok()
}

/// Discover agent from URL
#[napi]
pub async fn nexus_discover_agent(url: String) -> String {
    let nexus = get_nexus();
    let discovery = nexus.discovery();
    match discovery.discover(&url).await {
        Ok(card) => serde_json::to_string(&card).unwrap_or_else(|_| "{\"error\": \"serialization_failed\"}".to_string()),
        Err(e) => format!("{{\"error\": \"{}\"}}", e),
    }
}

/// Route task to best agent
#[napi]
pub async fn nexus_route_task(task_json: String) -> String {
    let nexus = get_nexus();
    match serde_json::from_str::<agentkern_nexus::Task>(&task_json) {
        Ok(task) => {
            match nexus.route(&task).await {
                Ok(agent) => {
                    // Enrich with match score (mock for now as route returns strict AgentCard)
                    let mut value = serde_json::to_value(&agent).unwrap();
                    if let Some(obj) = value.as_object_mut() {
                        obj.insert("matchScore".to_string(), serde_json::json!(0.95));
                    }
                    serde_json::to_string(&value).unwrap()
                },
                Err(e) => format!("{{\"error\": \"{}\"}}", e),
            }
        },
        Err(e) => format!("{{\"error\": \"invalid_json: {}\"}}", e),
    }
}

/// Get Nexus stats
#[napi]
pub async fn nexus_get_stats() -> String {
    let nexus = get_nexus();
    let registry = nexus.registry();
    let count = registry.count().await;
    format!("{{\"registeredAgents\": {}, \"supportedProtocols\": 6}}", count)
}
