//! WASM Actor Registry - Hot-Swappable Capability Routing
//!
//! Per Roadmap: "Gate must be refactored into a lean orchestrator
//! with all domain-specific logic running in the WASM actor plane."
//!
//! This registry provides:
//! - Capability-based routing to WASM modules
//! - Hot-swap without restart
//! - Version management
//! - Health monitoring

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "wasm")]
use wasmtime::{Engine, Module, Store, Instance, Linker};

/// Capability declaration for a WASM module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name (e.g., "prompt_guard", "carbon_check")
    pub name: String,
    /// Input schema (JSON Schema)
    pub input_schema: Option<serde_json::Value>,
    /// Output schema
    pub output_schema: Option<serde_json::Value>,
}

/// Metadata for a registered WASM actor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmActorMeta {
    /// Actor name
    pub name: String,
    /// Version
    pub version: String,
    /// Declared capabilities
    pub capabilities: Vec<Capability>,
    /// Module size in bytes
    pub size_bytes: usize,
    /// Load timestamp
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    /// Invocation count
    pub invocations: u64,
    /// Average latency (microseconds)
    pub avg_latency_us: u64,
}

/// WASM Actor with compiled module.
#[cfg(feature = "wasm")]
pub struct WasmActor {
    pub meta: WasmActorMeta,
    pub module: Module,
    pub engine: Arc<Engine>,
}

/// Result of WASM invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmInvokeResult {
    pub success: bool,
    pub output: Vec<u8>,
    pub latency_us: u64,
}

/// Error from WASM registry.
#[derive(Debug, Clone)]
pub enum RegistryError {
    NotFound(String),
    CompilationFailed(String),
    InvocationFailed(String),
    CapabilityNotFound(String),
    InvalidModule(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(n) => write!(f, "Actor not found: {}", n),
            Self::CompilationFailed(e) => write!(f, "Compilation failed: {}", e),
            Self::InvocationFailed(e) => write!(f, "Invocation failed: {}", e),
            Self::CapabilityNotFound(c) => write!(f, "Capability not found: {}", c),
            Self::InvalidModule(e) => write!(f, "Invalid module: {}", e),
        }
    }
}

impl std::error::Error for RegistryError {}

/// WASM Actor Registry with hot-swap and capability routing.
#[cfg(feature = "wasm")]
pub struct WasmRegistry {
    engine: Arc<Engine>,
    actors: RwLock<HashMap<String, Arc<WasmActor>>>,
    capability_index: RwLock<HashMap<String, Vec<String>>>,
}

#[cfg(feature = "wasm")]
impl WasmRegistry {
    /// Create a new registry.
    pub fn new() -> Result<Self, RegistryError> {
        use wasmtime::Config;
        
        let mut config = Config::new();
        config.async_support(true);
        config.consume_fuel(true);
        
        let engine = Engine::new(&config)
            .map_err(|e| RegistryError::CompilationFailed(e.to_string()))?;
        
        Ok(Self {
            engine: Arc::new(engine),
            actors: RwLock::new(HashMap::new()),
            capability_index: RwLock::new(HashMap::new()),
        })
    }

    /// Register a WASM actor from bytes.
    ///
    /// Supports hot-swap: if actor exists, it's replaced.
    pub fn register(
        &self,
        name: impl Into<String>,
        version: impl Into<String>,
        wasm_bytes: &[u8],
        capabilities: Vec<Capability>,
    ) -> Result<WasmActorMeta, RegistryError> {
        let name = name.into();
        let version = version.into();
        
        let module = Module::new(&*self.engine, wasm_bytes)
            .map_err(|e| RegistryError::CompilationFailed(e.to_string()))?;
        
        let meta = WasmActorMeta {
            name: name.clone(),
            version,
            capabilities: capabilities.clone(),
            size_bytes: wasm_bytes.len(),
            loaded_at: chrono::Utc::now(),
            invocations: 0,
            avg_latency_us: 0,
        };
        
        let actor = WasmActor {
            meta: meta.clone(),
            module,
            engine: Arc::clone(&self.engine),
        };
        
        // Update actor registry
        {
            let mut actors = self.actors.write();
            if actors.contains_key(&name) {
                tracing::info!(actor = %name, "Hot-swapping WASM actor");
            } else {
                tracing::info!(actor = %name, "Registering new WASM actor");
            }
            actors.insert(name.clone(), Arc::new(actor));
        }
        
        // Update capability index
        {
            let mut index = self.capability_index.write();
            for cap in &capabilities {
                index
                    .entry(cap.name.clone())
                    .or_insert_with(Vec::new)
                    .push(name.clone());
            }
        }
        
        Ok(meta)
    }

    /// Unregister an actor.
    pub fn unregister(&self, name: &str) -> bool {
        let removed = self.actors.write().remove(name).is_some();
        
        if removed {
            // Clean capability index
            let mut index = self.capability_index.write();
            for actors in index.values_mut() {
                actors.retain(|n| n != name);
            }
            tracing::info!(actor = %name, "Unregistered WASM actor");
        }
        
        removed
    }

    /// Invoke an actor by name.
    pub async fn invoke(
        &self,
        name: &str,
        input: &[u8],
    ) -> Result<WasmInvokeResult, RegistryError> {
        let actor = self.actors.read()
            .get(name)
            .cloned()
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;
        
        let start = std::time::Instant::now();
        
        // Create store with input data
        let mut store = Store::new(&*actor.engine, input.to_vec());
        store.set_fuel(100_000).ok();
        
        // Create linker and instantiate
        let linker = Linker::<Vec<u8>>::new(&*actor.engine);
        let instance = linker.instantiate_async(&mut store, &actor.module)
            .await
            .map_err(|e| RegistryError::InvocationFailed(e.to_string()))?;
        
        // Call evaluate function
        if let Some(evaluate) = instance.get_typed_func::<(), ()>(&mut store, "evaluate").ok() {
            evaluate.call_async(&mut store, ())
                .await
                .map_err(|e| RegistryError::InvocationFailed(e.to_string()))?;
        }
        
        let latency = start.elapsed().as_micros() as u64;
        
        Ok(WasmInvokeResult {
            success: true,
            output: store.data().clone(),
            latency_us: latency,
        })
    }

    /// Route by capability - find actors that provide a capability.
    pub fn route_by_capability(&self, capability: &str) -> Vec<String> {
        self.capability_index.read()
            .get(capability)
            .cloned()
            .unwrap_or_default()
    }

    /// Invoke by capability (uses first available actor).
    pub async fn invoke_capability(
        &self,
        capability: &str,
        input: &[u8],
    ) -> Result<WasmInvokeResult, RegistryError> {
        let actors = self.route_by_capability(capability);
        let actor_name = actors.first()
            .ok_or_else(|| RegistryError::CapabilityNotFound(capability.to_string()))?;
        
        self.invoke(actor_name, input).await
    }

    /// List all registered actors.
    pub fn list_actors(&self) -> Vec<WasmActorMeta> {
        self.actors.read()
            .values()
            .map(|a| a.meta.clone())
            .collect()
    }

    /// Get actor metadata.
    pub fn get_actor(&self, name: &str) -> Option<WasmActorMeta> {
        self.actors.read().get(name).map(|a| a.meta.clone())
    }

    /// Get registry statistics.
    pub fn stats(&self) -> RegistryStats {
        let actors = self.actors.read();
        RegistryStats {
            actor_count: actors.len(),
            total_size_bytes: actors.values().map(|a| a.meta.size_bytes).sum(),
            total_invocations: actors.values().map(|a| a.meta.invocations).sum(),
        }
    }
}

/// Registry statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub actor_count: usize,
    pub total_size_bytes: usize,
    pub total_invocations: u64,
}

// ============================================================================
// Fallback when WASM is disabled
// ============================================================================

#[cfg(not(feature = "wasm"))]
pub struct WasmRegistry;

#[cfg(not(feature = "wasm"))]
impl WasmRegistry {
    pub fn new() -> Result<Self, RegistryError> {
        Ok(Self)
    }

    pub fn register(
        &self,
        _name: impl Into<String>,
        _version: impl Into<String>,
        _wasm_bytes: &[u8],
        _capabilities: Vec<Capability>,
    ) -> Result<WasmActorMeta, RegistryError> {
        Err(RegistryError::InvalidModule("WASM feature not enabled".to_string()))
    }

    pub fn route_by_capability(&self, _capability: &str) -> Vec<String> {
        Vec::new()
    }

    pub fn list_actors(&self) -> Vec<WasmActorMeta> {
        Vec::new()
    }

    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            actor_count: 0,
            total_size_bytes: 0,
            total_invocations: 0,
        }
    }
}

#[cfg(not(feature = "wasm"))]
impl Default for WasmRegistry {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_struct() {
        let cap = Capability {
            name: "prompt_guard".to_string(),
            input_schema: None,
            output_schema: None,
        };
        assert_eq!(cap.name, "prompt_guard");
    }

    #[cfg(feature = "wasm")]
    #[tokio::test]
    async fn test_registry_creation() {
        let registry = WasmRegistry::new();
        assert!(registry.is_ok());
    }

    #[cfg(feature = "wasm")]
    #[tokio::test]
    async fn test_register_and_route() {
        let registry = WasmRegistry::new().unwrap();
        
        // Simple WAT module
        let wat = r#"(module (func (export "evaluate")))"#;
        let wasm_bytes = wat::parse_str(wat).unwrap();
        
        let caps = vec![Capability {
            name: "test_cap".to_string(),
            input_schema: None,
            output_schema: None,
        }];
        
        registry.register("test-actor", "1.0.0", &wasm_bytes, caps).unwrap();
        
        let actors = registry.route_by_capability("test_cap");
        assert_eq!(actors, vec!["test-actor"]);
    }
}
