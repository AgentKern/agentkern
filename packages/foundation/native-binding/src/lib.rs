#![warn(unused)] // Production: warn on unused code
//! AgentKern Native Binding
//!
//! NAPI-RS bindings exposing Rust core to Node.js Gateway.
//! This replaces the TypeScript simulation with real Rust execution.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    let context_data: HashMap<String, serde_json::Value> =
        serde_json::from_str(&request.context).unwrap_or_default();

    // Build verification request for Rust core
    let rust_request = agentkern_gate::types::VerificationRequest {
        request_id: uuid::Uuid::new_v4(),
        agent_id: request.agent_id.clone(),
        action: request.action.clone(),
        context: agentkern_gate::types::VerificationContext { data: context_data },
        timestamp: chrono::Utc::now(),
    };

    // Create Gate engine and verify
    let engine = agentkern_gate::engine::GateEngine::new();
    let verification = engine.verify(rust_request).await;

    let latency_ms = start.elapsed().as_millis() as u32;

    Ok(VerifyResult {
        allowed: verification.allowed,
        evaluated_policies: verification.evaluated_policies,
        blocking_policies: verification.blocking_policies,
        risk_score: verification.final_risk_score as u32,
        reasoning: Some(verification.reasoning),
        latency_ms,
    })
}

/// Get TEE attestation proof.
#[napi]
pub async fn get_attestation(nonce: String) -> Result<AttestationResult> {
    use agentkern_gate::tee::{TeePlatform, TeeRuntime};

    let runtime = TeeRuntime::detect()
        .map_err(|e| napi::Error::from_reason(format!("TEE detection failed: {}", e)))?;

    let platform_str = match runtime.platform() {
        TeePlatform::IntelTdx => "intel_tdx",
        TeePlatform::AmdSevSnp => "amd_sev_snp",
        TeePlatform::IntelSgx => "intel_sgx",
        TeePlatform::ArmCca => "arm_cca",
        TeePlatform::Simulated => "simulated",
    };

    // Get attestation from TEE
    let attestation = runtime
        .get_attestation(nonce.as_bytes())
        .map_err(|e| napi::Error::from_reason(format!("TEE error: {}", e)))?;

    Ok(AttestationResult {
        platform: platform_str.to_string(),
        quote: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &attestation.quote,
        ),
        measurement: hex::encode(&attestation.measurement),
        nonce,
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
}

/// Check carbon budget for an agent.
#[napi]
pub fn check_carbon_budget(agent_id: String, estimated_grams: f64) -> Result<bool> {
    use rust_decimal::prelude::ToPrimitive;

    // Use treasury carbon ledger
    let ledger = agentkern_treasury::carbon::CarbonLedger::new();

    match ledger.get_budget(&agent_id) {
        Some(budget) => {
            let usage = ledger.get_daily_usage(&agent_id);
            let current_usage = usage.total_co2_grams.to_f64().unwrap_or(0.0);
            let limit = budget.daily_limit_grams.to_f64().unwrap_or(f64::MAX);
            let would_exceed = current_usage + estimated_grams > limit;
            Ok(!would_exceed || !budget.block_on_exceed)
        }
        None => Ok(true), // No budget = allowed
    }
}

/// Initialize the AgentKern native runtime.
#[napi]
pub fn init_runtime() -> Result<String> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    Ok("AgentKern Native Runtime initialized".to_string())
}
