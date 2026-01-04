//! MicroVM Driver
//!
//! Generic trait for microVM technologies
//! Supports: Firecracker, gVisor, Kata Containers

use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// MicroVM driver trait - implement for each VM technology.
#[async_trait]
pub trait MicroVmDriver: Send + Sync {
    /// Driver name.
    fn name(&self) -> &str;
    
    /// VM technology type.
    fn vm_type(&self) -> VmType;
    
    /// Create a new VM instance.
    async fn create(&self, config: &VmConfig) -> Result<VmInstance, VmError>;
    
    /// Start a VM.
    async fn start(&self, instance_id: &str) -> Result<(), VmError>;
    
    /// Stop a VM.
    async fn stop(&self, instance_id: &str) -> Result<(), VmError>;
    
    /// Destroy a VM.
    async fn destroy(&self, instance_id: &str) -> Result<(), VmError>;
    
    /// Get VM state.
    async fn state(&self, instance_id: &str) -> Result<VmState, VmError>;
    
    /// Execute command in VM.
    async fn exec(&self, instance_id: &str, command: &[String]) -> Result<ExecResult, VmError>;
}

/// VM technology type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VmType {
    /// AWS Firecracker
    Firecracker,
    /// Google gVisor
    Gvisor,
    /// Kata Containers
    Kata,
    /// QEMU/KVM
    Qemu,
    /// Custom
    Custom,
}

/// VM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    /// Instance name
    pub name: String,
    /// Memory in MB
    pub memory_mb: u32,
    /// vCPUs
    pub vcpus: u32,
    /// Root filesystem path
    pub rootfs_path: String,
    /// Kernel path
    pub kernel_path: String,
    /// Kernel arguments
    pub kernel_args: Option<String>,
    /// Network config
    pub network: Option<NetworkConfig>,
    /// Max lifetime (seconds, 0 = unlimited)
    pub max_lifetime_secs: u32,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            memory_mb: 128,
            vcpus: 1,
            rootfs_path: String::new(),
            kernel_path: String::new(),
            kernel_args: None,
            network: None,
            max_lifetime_secs: 300, // 5 minutes
        }
    }
}

/// Network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub interface: String,
    pub mac_address: Option<String>,
    pub host_ip: Option<String>,
    pub guest_ip: Option<String>,
}

/// VM instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInstance {
    /// Instance ID
    pub id: String,
    /// State
    pub state: VmState,
    /// IP address
    pub ip_address: Option<String>,
    /// Started at (Unix timestamp)
    pub started_at: Option<u64>,
}

/// VM state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmState {
    /// Creating
    Creating,
    /// Running
    Running,
    /// Paused
    Paused,
    /// Stopped
    Stopped,
    /// Failed
    Failed,
}

/// Execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// VM error.
#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("VM not found: {0}")]
    NotFound(String),
    
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid state: expected {expected:?}, got {actual:?}")]
    InvalidState { expected: VmState, actual: VmState },
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("License error: {0}")]
    LicenseError(#[from] agentkern_connectors_ee::license::LicenseError),
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_config_default() {
        let config = VmConfig::default();
        assert_eq!(config.memory_mb, 128);
        assert_eq!(config.vcpus, 1);
    }

    #[test]
    fn test_vm_state() {
        assert_ne!(VmState::Running, VmState::Stopped);
    }

    #[test]
    fn test_vm_type() {
        assert_eq!(VmType::Firecracker, VmType::Firecracker);
        assert_ne!(VmType::Firecracker, VmType::Gvisor);
    }
}
