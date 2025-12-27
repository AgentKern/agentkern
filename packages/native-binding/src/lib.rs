//! AgentKern Native Binding
//!
//! NAPI-RS bindings exposing Rust core to Node.js Gateway.
//! This replaces the TypeScript simulation with real Rust execution.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Verification request from Gateway.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub agent_id: String,
    pub action: String,
    pub context: String, // JSON string
}

/// Verification result to Gateway.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    pub allowed: bool,
    pub evaluated_policies: Vec<String>,
    pub blocking_policies: Vec<String>,
    pub risk_score: u32,
    pub reasoning: Option<String>,
    pub latency_ms: u32,
}

/// TEE Attestation result.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationResult {
    pub platform: String,
    pub quote: String,
    pub measurement: String,
    pub nonce: String,
    pub timestamp: i64,
}

/// Carbon budget configuration.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonBudgetConfig {
    pub daily_limit_grams: f64,
    pub monthly_limit_grams: Option<f64>,
    pub block_on_exceed: bool,
}

/// Verify an agent action using Rust Gate engine.
#[napi]
pub async fn verify_action(request: VerifyRequest) -> Result<VerifyResult> {
    use std::time::Instant;
    let start = Instant::now();
    
    // Parse context
    let context: std::collections::HashMap<String, serde_json::Value> = 
        serde_json::from_str(&request.context).unwrap_or_default();
    
    // Build verification request for Rust core
    let rust_request = agentkern_gate::types::VerificationRequest {
        agent_id: request.agent_id.clone(),
        action: request.action.clone(),
        resource: context.get("resource").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        parameters: context.iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect(),
        timestamp: chrono::Utc::now(),
        session_id: None,
    };
    
    // Create Gate engine and verify
    let engine = agentkern_gate::engine::GateEngine::new();
    let result = engine.verify(&rust_request).await;
    
    let latency_ms = start.elapsed().as_millis() as u32;
    
    match result {
        Ok(verification) => Ok(VerifyResult {
            allowed: verification.allowed,
            evaluated_policies: verification.evaluated_policies,
            blocking_policies: verification.blocking_policies,
            risk_score: verification.risk_score as u32,
            reasoning: verification.reasoning,
            latency_ms,
        }),
        Err(e) => Ok(VerifyResult {
            allowed: false,
            evaluated_policies: vec![],
            blocking_policies: vec!["error".to_string()],
            risk_score: 100,
            reasoning: Some(format!("Verification error: {}", e)),
            latency_ms,
        }),
    }
}

/// Get TEE attestation proof.
#[napi]
pub async fn get_attestation(nonce: String) -> Result<AttestationResult> {
    use agentkern_gate::tee::{TeeRuntime, TeePlatform};
    
    let runtime = TeeRuntime::detect();
    
    let platform_str = match runtime.platform() {
        TeePlatform::IntelTdx => "intel_tdx",
        TeePlatform::AmdSevSnp => "amd_sev_snp",
        TeePlatform::Simulated => "simulated",
    };
    
    // Get attestation from TEE
    let attestation = runtime.get_attestation(nonce.as_bytes())
        .map_err(|e| napi::Error::from_reason(format!("TEE error: {}", e)))?;
    
    Ok(AttestationResult {
        platform: platform_str.to_string(),
        quote: base64::encode(&attestation.quote),
        measurement: hex::encode(&attestation.measurement),
        nonce,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}

/// Check carbon budget for an agent.
#[napi]
pub fn check_carbon_budget(agent_id: String, estimated_grams: f64) -> Result<bool> {
    // Use treasury carbon ledger
    let ledger = agentkern_treasury::carbon::CarbonLedger::new();
    
    match ledger.get_budget(&agent_id) {
        Ok(budget) => {
            let usage = ledger.get_usage(&agent_id).unwrap_or_default();
            let would_exceed = usage.total_grams + estimated_grams > budget.daily_limit_grams;
            Ok(!would_exceed || !budget.block_on_exceed)
        }
        Err(_) => Ok(true), // No budget = allowed
    }
}

/// Initialize the AgentKern native runtime.
#[napi]
pub fn init_runtime() -> Result<String> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    Ok("AgentKern Native Runtime initialized".to_string())
}
