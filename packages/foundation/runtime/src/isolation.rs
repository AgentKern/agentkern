//! WASM Isolation Layer
//!
//! Per ARCHITECTURE.md: "Nano-Isolation"
//! AgentKern uses WASM Components (Nano-Light), NOT Docker (Heavy)
//!
//! Priority: WASM > Container > Process

use serde::{Deserialize, Serialize};

/// Isolation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IsolationMode {
    /// WASM Components (Default, Nano-Light)
    /// - Microsecond startup
    /// - Capability-based security
    /// - Truly universal binaries
    #[default]
    Wasm,

    /// Container fallback (for legacy workloads)
    /// Only used when WASM is not available
    Container,

    /// Process isolation (minimal)
    /// For development/testing only
    Process,
}

/// Isolation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    /// Primary isolation mode
    pub mode: IsolationMode,
    /// Fallback mode if primary fails
    pub fallback: Option<IsolationMode>,
    /// WASM-specific config
    pub wasm: WasmConfig,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            mode: IsolationMode::Wasm,
            fallback: Some(IsolationMode::Container),
            wasm: WasmConfig::default(),
        }
    }
}

/// WASM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Runtime (wasmtime, wasmer, wasmedge)
    pub runtime: WasmRuntime,
    /// Fuel limit per execution
    pub fuel_limit: u64,
    /// Enable capability verification
    pub verify_capabilities: bool,
    /// Enable module signing
    pub require_signatures: bool,
    /// Allowed WASI capabilities
    pub capabilities: Vec<WasiCapability>,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            runtime: WasmRuntime::Wasmtime,
            fuel_limit: 10_000,
            verify_capabilities: true,
            require_signatures: false, // Strict in production
            capabilities: vec![WasiCapability::Clock, WasiCapability::Random],
        }
    }
}

/// WASM runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmRuntime {
    Wasmtime,
    Wasmer,
    WasmEdge,
}

/// WASI capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasiCapability {
    /// Clock/time access
    Clock,
    /// Random number generation
    Random,
    /// Environment variables
    Environment,
    /// Filesystem (with path restrictions)
    Filesystem { path: String, readonly: bool },
    /// Network (with allowlist)
    Network { hosts: Vec<String> },
}

/// Check if WASM is available in the current environment.
pub fn wasm_available() -> bool {
    // WASM is available if we can create a wasmtime engine
    // In practice, this is available on most platforms
    #[cfg(target_arch = "wasm32")]
    return true; // Already in WASM

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Check for wasmtime features
        cfg!(feature = "wasm")
    }
}

/// Detect the best isolation mode for the environment.
pub fn detect_best_isolation() -> IsolationMode {
    if wasm_available() {
        IsolationMode::Wasm
    } else if container_runtime_available() {
        IsolationMode::Container
    } else {
        IsolationMode::Process
    }
}

/// Check if a container runtime is available.
fn container_runtime_available() -> bool {
    std::path::Path::new("/var/run/docker.sock").exists()
        || std::path::Path::new("/run/containerd/containerd.sock").exists()
        || std::path::Path::new("/run/podman/podman.sock").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_wasm() {
        assert_eq!(IsolationMode::default(), IsolationMode::Wasm);
    }

    #[test]
    fn test_isolation_config_defaults() {
        let config = IsolationConfig::default();
        assert_eq!(config.mode, IsolationMode::Wasm);
        assert!(config.wasm.verify_capabilities);
    }
}
