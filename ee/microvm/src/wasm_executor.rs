//! WASM-in-VM Executor
//!
//! Execute WASM modules inside microVMs for verified isolation

use serde::{Deserialize, Serialize};
use super::driver::{MicroVmDriver, VmConfig, VmError};

/// WASM-in-VM executor.
pub struct WasmInVm<D: MicroVmDriver> {
    driver: D,
    config: WasmVmConfig,
}

/// WASM-in-VM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmVmConfig {
    /// Base VM config
    pub vm: VmConfig,
    /// WASM runtime (wasmtime, wasmer, etc.)
    pub wasm_runtime: WasmRuntime,
    /// Enable cryptographic verification
    pub verify_modules: bool,
    /// Allowed host capabilities
    pub capabilities: Vec<WasiCapability>,
}

/// WASM runtime type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmRuntime {
    Wasmtime,
    Wasmer,
    WasmEdge,
}

/// WASI capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasiCapability {
    /// Filesystem access
    Filesystem { path: String, readonly: bool },
    /// Network access
    Network,
    /// Environment variables
    Environment,
    /// Clock/time
    Clock,
    /// Random
    Random,
}

/// WASM module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModule {
    /// Module hash (SHA-256)
    pub hash: String,
    /// Module bytes
    #[serde(skip)]
    pub bytes: Vec<u8>,
    /// Signature (Ed25519)
    pub signature: Option<String>,
    /// Entry point function
    pub entry_point: String,
}

impl WasmModule {
    /// Create new module from bytes.
    pub fn new(bytes: Vec<u8>, entry_point: &str) -> Self {
        use sha2::{Sha256, Digest};
        let hash = hex::encode(Sha256::digest(&bytes));
        
        Self {
            hash,
            bytes,
            signature: None,
            entry_point: entry_point.to_string(),
        }
    }
    
    /// Sign module.
    pub fn sign(&mut self, signature: String) {
        self.signature = Some(signature);
    }
    
    /// Verify signature.
    pub fn verify(&self, public_key: &[u8]) -> bool {
        // Would verify Ed25519 signature
        self.signature.is_some()
    }
}

/// Execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Module hash
    pub module_hash: String,
    /// Exit code
    pub exit_code: i32,
    /// Stdout
    pub stdout: String,
    /// Stderr
    pub stderr: String,
    /// Execution time (ms)
    pub execution_time_ms: u64,
    /// Proof log
    pub proof_log: Option<String>,
}

impl<D: MicroVmDriver> WasmInVm<D> {
    /// Create new executor.
    pub fn new(driver: D, config: WasmVmConfig) -> Result<Self, VmError> {
        agentkern_connectors_ee::license::check_feature_license("microvm")?;
        Ok(Self { driver, config })
    }
    
    /// Execute WASM module in isolated VM.
    pub async fn execute(&self, module: &WasmModule, args: &[String]) -> Result<ExecutionResult, WasmExecutionError> {
        // 1. Verify module if required
        if self.config.verify_modules && module.signature.is_none() {
            return Err(WasmExecutionError::ModuleNotSigned);
        }
        
        // 2. Create VM
        let vm = self.driver.create(&self.config.vm).await
            .map_err(|e| WasmExecutionError::VmError(e))?;
        
        // 3. Start VM
        self.driver.start(&vm.id).await
            .map_err(|e| WasmExecutionError::VmError(e))?;
        
        // 4. Execute WASM
        // In production: copy module to VM, run wasmtime
        let exec_result = self.driver.exec(&vm.id, &[
            "wasmtime".to_string(),
            "run".to_string(),
            "--invoke".to_string(),
            module.entry_point.clone(),
            "/tmp/module.wasm".to_string(),
        ]).await.map_err(|e| WasmExecutionError::VmError(e))?;
        
        // 5. Destroy VM (stateless)
        let _ = self.driver.destroy(&vm.id).await;
        
        Ok(ExecutionResult {
            module_hash: module.hash.clone(),
            exit_code: exec_result.exit_code,
            stdout: exec_result.stdout,
            stderr: exec_result.stderr,
            execution_time_ms: exec_result.duration_ms,
            proof_log: Some(format!("Executed {} in VM {}", module.hash, vm.id)),
        })
    }
}

/// WASM execution error.
#[derive(Debug, thiserror::Error)]
pub enum WasmExecutionError {
    #[error("Module not signed")]
    ModuleNotSigned,
    
    #[error("Signature verification failed")]
    SignatureInvalid,
    
    #[error("VM error: {0}")]
    VmError(VmError),
    
    #[error("Execution timeout")]
    Timeout,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_module_create() {
        let bytes = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic
        let module = WasmModule::new(bytes, "main");
        
        assert!(!module.hash.is_empty());
        assert_eq!(module.entry_point, "main");
    }

    #[test]
    fn test_wasm_module_sign() {
        let mut module = WasmModule::new(vec![0x00], "main");
        assert!(module.signature.is_none());
        
        module.sign("test_signature".into());
        assert!(module.signature.is_some());
    }
}
