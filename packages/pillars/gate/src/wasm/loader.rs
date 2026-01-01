//! WASM Policy Loader
//!
//! Auto-loads built WASM policy modules from the policies directory.
//! Supports hot-reload via file system watcher.

use super::registry::{Capability, RegistryError, WasmRegistry};
use std::path::{Path, PathBuf};

/// Pre-built WASM policy paths (relative to crate root).
pub const PROMPT_GUARD_WASM: &str =
    "wasm-policies/prompt-guard/target/wasm32-unknown-unknown/release/prompt_guard_wasm.wasm";

/// Load all built WASM policies into the registry.
pub fn load_policies(registry: &WasmRegistry, base_path: &Path) -> Result<usize, RegistryError> {
    let mut loaded = 0;

    // Prompt Guard
    let prompt_guard_path = base_path.join(PROMPT_GUARD_WASM);
    if prompt_guard_path.exists() {
        load_prompt_guard(registry, &prompt_guard_path)?;
        loaded += 1;
    }

    // Future: Add more policies here
    // - carbon_check
    // - compliance_hipaa
    // - explain_engine

    Ok(loaded)
}

/// Load the prompt_guard WASM module.
fn load_prompt_guard(registry: &WasmRegistry, path: &PathBuf) -> Result<(), RegistryError> {
    let wasm_bytes = std::fs::read(path)
        .map_err(|e| RegistryError::InvalidModule(format!("Failed to read: {}", e)))?;

    let capabilities = vec![
        Capability {
            name: "prompt_guard".to_string(),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string" },
                    "context": { "type": "string" }
                },
                "required": ["prompt"]
            })),
            output_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "safe": { "type": "boolean" },
                    "threat_level": { "type": "string" },
                    "score": { "type": "integer" }
                }
            })),
        },
        Capability {
            name: "injection_detection".to_string(),
            input_schema: None,
            output_schema: None,
        },
    ];

    registry.register("prompt-guard", "1.0.0", &wasm_bytes, capabilities)?;

    tracing::info!("Loaded prompt_guard WASM policy");
    Ok(())
}

/// Invoke prompt_guard capability with a prompt string.
#[cfg(feature = "wasm")]
pub async fn check_prompt(
    registry: &WasmRegistry,
    prompt: &str,
    context: Option<&str>,
) -> Result<PromptCheckResult, RegistryError> {
    let input = serde_json::json!({
        "prompt": prompt,
        "context": context
    });

    let input_bytes = serde_json::to_vec(&input).unwrap_or_default();

    let result = registry
        .invoke_capability("prompt_guard", &input_bytes)
        .await?;

    // Parse output
    let output: PromptCheckResult =
        serde_json::from_slice(&result.output).unwrap_or_else(|_| PromptCheckResult {
            safe: true,
            threat_level: "Unknown".to_string(),
            score: 0,
            latency_us: result.latency_us,
        });

    Ok(PromptCheckResult {
        latency_us: result.latency_us,
        ..output
    })
}

/// Result from prompt check.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptCheckResult {
    pub safe: bool,
    pub threat_level: String,
    pub score: u8,
    pub latency_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_policy_paths() {
        assert!(PROMPT_GUARD_WASM.ends_with(".wasm"));
    }
}
