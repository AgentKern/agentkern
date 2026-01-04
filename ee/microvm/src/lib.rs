//! MicroVM Integration
//!
//! Generic microVM support for WASM-in-VM isolation
//! Supports: Firecracker, gVisor, Kata Containers

pub mod driver;
pub mod wasm_executor;

pub use driver::{MicroVmDriver, VmConfig, VmInstance, VmState};
pub use wasm_executor::{WasmInVm, WasmModule, ExecutionResult};
