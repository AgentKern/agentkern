//! WASM Policy Engine - Full Implementation
//!
//! Per ARCHITECTURE.md: "Nano-Isolation"
//! - Microsecond startup vs milliseconds for containers
//! - Capability-based security model
//! - Hot-swappable policy modules
//!
//! Policies are compiled to WASM and run in isolated sandboxes.

pub mod registry;

pub use registry::{Capability, RegistryError, RegistryStats, WasmActorMeta, WasmInvokeResult, WasmRegistry};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "wasm")]
use wasmtime::*;

/// Result of WASM policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPolicyResult {
    pub allowed: bool,
    pub risk_score: u8,
    pub message: Option<String>,
}

/// A compiled WASM policy module.
#[cfg(feature = "wasm")]
pub struct WasmPolicy {
    module: Module,
    name: String,
}

/// WASM Policy Engine for nano-isolation.
#[cfg(feature = "wasm")]
pub struct WasmPolicyEngine {
    engine: Engine,
    linker: Linker<PolicyState>,
    policies: HashMap<String, WasmPolicy>,
}

/// State passed to WASM policies.
#[cfg(feature = "wasm")]
pub struct PolicyState {
    pub action: String,
    pub context: serde_json::Value,
    pub result: WasmPolicyResult,
}

#[cfg(feature = "wasm")]
impl WasmPolicyEngine {
    /// Create a new WASM policy engine.
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut config = Config::new();
        config.async_support(true);
        config.consume_fuel(true); // Resource limiting

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);

        // Add host functions for policies to call
        linker.func_wrap(
            "env",
            "log",
            |caller: Caller<'_, PolicyState>, ptr: i32, len: i32| {
                // Log function for policies
                tracing::debug!("WASM policy log: ptr={}, len={}", ptr, len);
            },
        )?;

        linker.func_wrap(
            "env",
            "get_action_len",
            |caller: Caller<'_, PolicyState>| -> i32 { caller.data().action.len() as i32 },
        )?;

        linker.func_wrap(
            "env",
            "set_allowed",
            |mut caller: Caller<'_, PolicyState>, allowed: i32| {
                caller.data_mut().result.allowed = allowed != 0;
            },
        )?;

        linker.func_wrap(
            "env",
            "set_risk_score",
            |mut caller: Caller<'_, PolicyState>, score: i32| {
                caller.data_mut().result.risk_score = score.clamp(0, 100) as u8;
            },
        )?;

        Ok(Self {
            engine,
            linker,
            policies: HashMap::new(),
        })
    }

    /// Load a policy from WASM bytes.
    pub fn load_policy(
        &mut self,
        name: impl Into<String>,
        wasm_bytes: &[u8],
    ) -> Result<(), anyhow::Error> {
        let name = name.into();
        let module = Module::new(&self.engine, wasm_bytes)?;
        self.policies
            .insert(name.clone(), WasmPolicy { module, name });
        Ok(())
    }

    /// Load a policy from a WAT (WebAssembly Text) string.
    pub fn load_policy_wat(
        &mut self,
        name: impl Into<String>,
        wat: &str,
    ) -> Result<(), anyhow::Error> {
        let name = name.into();
        let module = Module::new(&self.engine, wat)?;
        self.policies
            .insert(name.clone(), WasmPolicy { module, name });
        Ok(())
    }

    /// Evaluate a policy.
    pub async fn evaluate(
        &self,
        policy_name: &str,
        action: &str,
        context: &serde_json::Value,
    ) -> Result<WasmPolicyResult, anyhow::Error> {
        let policy = self
            .policies
            .get(policy_name)
            .ok_or_else(|| anyhow::anyhow!("Policy not found: {}", policy_name))?;

        let mut store = Store::new(
            &self.engine,
            PolicyState {
                action: action.to_string(),
                context: context.clone(),
                result: WasmPolicyResult {
                    allowed: true,
                    risk_score: 0,
                    message: None,
                },
            },
        );

        // Set fuel limit for resource control
        store.set_fuel(10_000)?;

        let instance = self
            .linker
            .instantiate_async(&mut store, &policy.module)
            .await?;

        // Call the evaluate function
        if let Some(evaluate) = instance
            .get_typed_func::<(), ()>(&mut store, "evaluate")
            .ok()
        {
            evaluate.call_async(&mut store, ()).await?;
        }

        Ok(store.data().result.clone())
    }

    /// Get list of loaded policies.
    pub fn list_policies(&self) -> Vec<&str> {
        self.policies.keys().map(|s| s.as_str()).collect()
    }

    /// Unload a policy.
    pub fn unload_policy(&mut self, name: &str) -> bool {
        self.policies.remove(name).is_some()
    }
}

#[cfg(feature = "wasm")]
impl Default for WasmPolicyEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM engine")
    }
}

// ============================================================================
// Fallback when WASM feature is disabled
// ============================================================================

#[cfg(not(feature = "wasm"))]
pub struct WasmPolicyEngine;

#[cfg(not(feature = "wasm"))]
impl WasmPolicyEngine {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self)
    }

    pub fn load_policy(
        &mut self,
        _name: impl Into<String>,
        _wasm_bytes: &[u8],
    ) -> Result<(), anyhow::Error> {
        Err(anyhow::anyhow!(
            "WASM feature not enabled. Compile with --features wasm"
        ))
    }

    pub async fn evaluate(
        &self,
        _policy_name: &str,
        _action: &str,
        _context: &serde_json::Value,
    ) -> Result<WasmPolicyResult, anyhow::Error> {
        Err(anyhow::anyhow!("WASM feature not enabled"))
    }
}

#[cfg(not(feature = "wasm"))]
impl Default for WasmPolicyEngine {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_engine_creation() {
        let engine = WasmPolicyEngine::new();
        assert!(engine.is_ok());
    }

    #[cfg(feature = "wasm")]
    #[tokio::test]
    async fn test_wasm_policy_evaluation() {
        let mut engine = WasmPolicyEngine::new().unwrap();

        // Simple WAT policy that always allows
        let wat = r#"
            (module
                (import "env" "set_allowed" (func $set_allowed (param i32)))
                (import "env" "set_risk_score" (func $set_risk_score (param i32)))
                (func (export "evaluate")
                    i32.const 1
                    call $set_allowed
                    i32.const 10
                    call $set_risk_score
                )
            )
        "#;

        engine.load_policy_wat("test-policy", wat).unwrap();

        let result = engine
            .evaluate("test-policy", "test_action", &serde_json::json!({}))
            .await
            .unwrap();

        assert!(result.allowed);
        assert_eq!(result.risk_score, 10);
    }
}
