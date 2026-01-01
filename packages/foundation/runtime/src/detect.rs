//! Environment Detection
//!
//! Auto-detects the runtime environment without vendor-specific code.
//! Uses standard Linux/POSIX signals and environment variables.

use std::env;
use std::path::Path;

/// Detected environment type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    /// Running in Docker/Podman container
    Container { runtime: ContainerRuntime },
    /// Running in Kubernetes
    Kubernetes { namespace: String, pod_name: String },
    /// Running in serverless environment
    Serverless { platform: ServerlessPlatform },
    /// Running on bare metal or VM
    Server { os: OperatingSystem },
    /// Running on edge device
    Edge { device_type: EdgeDevice },
    /// Running in browser (WASM)
    Browser,
    /// Unknown environment
    Unknown,
}

/// Container runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerRuntime {
    Docker,
    Podman,
    Containerd,
    Unknown,
}

/// Serverless platform (detected generically).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerlessPlatform {
    FunctionAsService, // Generic - could be AWS Lambda, Azure Functions, etc.
    Unknown,
}

/// Operating system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

/// Edge device type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeDevice {
    RaspberryPi,
    Jetson,
    Generic,
}

/// Detect the current runtime environment.
pub fn detect_environment() -> Environment {
    // Check for Kubernetes first (most specific)
    if is_kubernetes() {
        return Environment::Kubernetes {
            namespace: env::var("KUBERNETES_NAMESPACE")
                .or_else(|_| env::var("POD_NAMESPACE"))
                .unwrap_or_else(|_| "default".into()),
            pod_name: env::var("HOSTNAME").unwrap_or_else(|_| "unknown".into()),
        };
    }

    // Check for serverless
    if is_serverless() {
        return Environment::Serverless {
            platform: ServerlessPlatform::FunctionAsService,
        };
    }

    // Check for container
    if is_container() {
        return Environment::Container {
            runtime: detect_container_runtime(),
        };
    }

    // Check for edge device
    if let Some(device) = detect_edge_device() {
        return Environment::Edge {
            device_type: device,
        };
    }

    // Default to server
    Environment::Server { os: detect_os() }
}

/// Check if running in Kubernetes.
fn is_kubernetes() -> bool {
    // Standard Kubernetes environment variables
    env::var("KUBERNETES_SERVICE_HOST").is_ok()
        || env::var("KUBERNETES_PORT").is_ok()
        || Path::new("/var/run/secrets/kubernetes.io").exists()
}

/// Check if running in serverless environment.
fn is_serverless() -> bool {
    // Generic serverless detection (no vendor names)
    env::var("FUNCTION_NAME").is_ok()
        || env::var("FUNCTION_TARGET").is_ok()
        || env::var("_HANDLER").is_ok()
}

/// Check if running in a container.
fn is_container() -> bool {
    // Check for container indicators
    Path::new("/.dockerenv").exists()
        || Path::new("/run/.containerenv").exists()
        || env::var("container").is_ok()
        || check_cgroup_container()
}

/// Check cgroup for container detection.
fn check_cgroup_container() -> bool {
    if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
        return cgroup.contains("docker")
            || cgroup.contains("kubepods")
            || cgroup.contains("lxc")
            || cgroup.contains("containerd");
    }
    false
}

/// Detect container runtime.
fn detect_container_runtime() -> ContainerRuntime {
    if Path::new("/.dockerenv").exists() {
        ContainerRuntime::Docker
    } else if Path::new("/run/.containerenv").exists() {
        ContainerRuntime::Podman
    } else {
        ContainerRuntime::Unknown
    }
}

/// Detect edge device.
fn detect_edge_device() -> Option<EdgeDevice> {
    // Check for Raspberry Pi
    if let Ok(model) = std::fs::read_to_string("/proc/device-tree/model") {
        if model.contains("Raspberry Pi") {
            return Some(EdgeDevice::RaspberryPi);
        }
        if model.contains("NVIDIA Jetson") {
            return Some(EdgeDevice::Jetson);
        }
    }

    // Check for low-memory edge device
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        if let Some(line) = meminfo.lines().find(|l| l.starts_with("MemTotal:")) {
            if let Some(kb) = line.split_whitespace().nth(1) {
                if let Ok(kb_val) = kb.parse::<u64>() {
                    // Less than 4GB = likely edge device
                    if kb_val < 4_000_000 {
                        return Some(EdgeDevice::Generic);
                    }
                }
            }
        }
    }

    None
}

/// Detect operating system.
fn detect_os() -> OperatingSystem {
    #[cfg(target_os = "linux")]
    return OperatingSystem::Linux;

    #[cfg(target_os = "macos")]
    return OperatingSystem::MacOS;

    #[cfg(target_os = "windows")]
    return OperatingSystem::Windows;

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return OperatingSystem::Unknown;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os() {
        let os = detect_os();
        // Should at least return something
        assert!(matches!(
            os,
            OperatingSystem::Linux
                | OperatingSystem::MacOS
                | OperatingSystem::Windows
                | OperatingSystem::Unknown
        ));
    }

    #[test]
    fn test_detect_environment() {
        let env = detect_environment();
        // Should at least detect something
        assert!(!matches!(env, Environment::Browser)); // Not WASM in tests
    }
}
