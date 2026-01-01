//! Auto-Configuration
//!
//! Configures AgentKern based on detected environment.
//! No vendor-specific settings - just universal parameters.

use crate::detect::Environment;
use std::env;
use std::net::{IpAddr, Ipv4Addr};

/// Runtime configuration.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Bind address
    pub bind_address: IpAddr,
    /// HTTP port
    pub http_port: u16,
    /// gRPC port (if enabled)
    pub grpc_port: Option<u16>,
    /// WebSocket enabled
    pub websocket_enabled: bool,
    /// Max concurrent connections
    pub max_connections: usize,
    /// Memory limit (bytes, 0 = unlimited)
    pub memory_limit: usize,
    /// Database URL (auto-detected or from env)
    pub database_url: Option<String>,
    /// Cache URL (auto-detected or from env)
    pub cache_url: Option<String>,
    /// Protocols to enable
    pub protocols: Vec<Protocol>,
    /// Resource mode
    pub resource_mode: ResourceMode,
}

/// Protocol types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Grpc,
    WebSocket,
    A2A, // Agent-to-Agent
    Mcp, // Model Context Protocol
}

/// Resource allocation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceMode {
    /// Minimal resources (edge devices)
    Minimal,
    /// Standard resources (containers)
    Standard,
    /// Full resources (dedicated servers)
    Full,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            bind_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            http_port: 3000,
            grpc_port: Some(50051),
            websocket_enabled: true,
            max_connections: 1000,
            memory_limit: 0,
            database_url: None,
            cache_url: None,
            protocols: vec![Protocol::Http, Protocol::WebSocket, Protocol::A2A],
            resource_mode: ResourceMode::Standard,
        }
    }
}

/// Auto-configure based on environment.
pub fn auto_configure(env: &Environment) -> RuntimeConfig {
    let mut config = RuntimeConfig::default();

    // Apply environment-specific settings
    match env {
        Environment::Kubernetes { .. } => {
            // Kubernetes: use standard settings, let K8s handle scaling
            config.max_connections = 5000;
            config.resource_mode = ResourceMode::Full;
        }

        Environment::Container { .. } => {
            // Container: respect memory limits
            config.memory_limit = detect_memory_limit();
            config.resource_mode = ResourceMode::Standard;
        }

        Environment::Serverless { .. } => {
            // Serverless: minimal footprint, fast startup
            config.grpc_port = None; // Save resources
            config.max_connections = 100;
            config.resource_mode = ResourceMode::Minimal;
        }

        Environment::Edge { .. } => {
            // Edge: very constrained
            config.grpc_port = None;
            config.websocket_enabled = false;
            config.max_connections = 50;
            config.memory_limit = 256 * 1024 * 1024; // 256MB max
            config.resource_mode = ResourceMode::Minimal;
        }

        Environment::Server { .. } => {
            // Bare metal: use all resources
            config.max_connections = 10000;
            config.resource_mode = ResourceMode::Full;
            config.protocols.push(Protocol::Grpc);
            config.protocols.push(Protocol::Mcp);
        }

        _ => {}
    }

    // Override with environment variables (user preference)
    apply_env_overrides(&mut config);

    // Auto-detect database and cache
    config.database_url = detect_database();
    config.cache_url = detect_cache();

    config
}

/// Apply environment variable overrides.
fn apply_env_overrides(config: &mut RuntimeConfig) {
    if let Ok(port) = env::var("PORT") {
        if let Ok(p) = port.parse() {
            config.http_port = p;
        }
    }

    if let Ok(port) = env::var("GRPC_PORT") {
        if let Ok(p) = port.parse() {
            config.grpc_port = Some(p);
        }
    }

    if let Ok(addr) = env::var("BIND_ADDRESS") {
        if let Ok(a) = addr.parse() {
            config.bind_address = a;
        }
    }

    if let Ok(max) = env::var("MAX_CONNECTIONS") {
        if let Ok(m) = max.parse() {
            config.max_connections = m;
        }
    }
}

/// Detect memory limit from cgroup or system.
fn detect_memory_limit() -> usize {
    // Try cgroup v2
    if let Ok(limit) = std::fs::read_to_string("/sys/fs/cgroup/memory.max") {
        if let Ok(bytes) = limit.trim().parse::<usize>() {
            return bytes;
        }
    }

    // Try cgroup v1
    if let Ok(limit) = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.limit_in_bytes") {
        if let Ok(bytes) = limit.trim().parse::<usize>() {
            if bytes < usize::MAX / 2 {
                // Not "unlimited"
                return bytes;
            }
        }
    }

    // No limit
    0
}

/// Auto-detect database from common environment variables.
fn detect_database() -> Option<String> {
    // Try common database URL patterns (no vendor-specific names)
    env::var("DATABASE_URL")
        .or_else(|_| env::var("DB_URL"))
        .or_else(|_| env::var("POSTGRES_URL"))
        .or_else(|_| env::var("MYSQL_URL"))
        .or_else(|_| env::var("SQLITE_URL"))
        .ok()
}

/// Auto-detect cache from common environment variables.
fn detect_cache() -> Option<String> {
    env::var("CACHE_URL")
        .or_else(|_| env::var("REDIS_URL"))
        .or_else(|_| env::var("MEMCACHED_URL"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect::{Environment, OperatingSystem};

    #[test]
    fn test_default_config() {
        let config = RuntimeConfig::default();
        assert_eq!(config.http_port, 3000);
        assert!(config.protocols.contains(&Protocol::Http));
    }

    #[test]
    fn test_auto_configure_server() {
        let env = Environment::Server {
            os: OperatingSystem::Linux,
        };
        let config = auto_configure(&env);
        assert_eq!(config.resource_mode, ResourceMode::Full);
        assert_eq!(config.max_connections, 10000);
    }

    #[test]
    fn test_auto_configure_edge() {
        let env = Environment::Edge {
            device_type: crate::detect::EdgeDevice::RaspberryPi,
        };
        let config = auto_configure(&env);
        assert_eq!(config.resource_mode, ResourceMode::Minimal);
        assert!(config.grpc_port.is_none());
    }
}
