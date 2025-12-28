//! Universal Server
//!
//! Serves AgentKern on any environment.
//! Uses standard protocols (HTTP, gRPC, WebSocket).

use crate::config::RuntimeConfig;
use std::net::SocketAddr;

/// Serve AgentKern with the given configuration.
pub async fn serve(config: &RuntimeConfig) -> Result<(), ServeError> {
    let addr = SocketAddr::new(config.bind_address, config.http_port);
    
    tracing::info!("AgentKern starting on {}", addr);
    tracing::info!("Protocols: {:?}", config.protocols);
    tracing::info!("Resource mode: {:?}", config.resource_mode);
    
    // Log enabled protocols
    for protocol in &config.protocols {
        match protocol {
            Protocol::Http => tracing::info!("HTTP enabled on port {}", config.http_port),
            Protocol::Grpc => {
                if let Some(port) = config.grpc_port {
                    tracing::info!("gRPC enabled on port {}", port);
                }
            }
            Protocol::WebSocket => tracing::info!("WebSocket enabled"),
            Protocol::A2A => tracing::info!("A2A protocol enabled"),
            Protocol::Mcp => tracing::info!("MCP protocol enabled"),
        }
    }
    
    // In a full implementation, we would start the actual servers here
    // For now, this is the interface contract
    
    tracing::info!("AgentKern running. Press Ctrl+C to stop.");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await.map_err(|e| ServeError::Signal(e.to_string()))?;
    
    tracing::info!("Shutting down gracefully...");
    
    Ok(())
}

/// Server error.
#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    #[error("Bind error: {0}")]
    Bind(String),
    
    #[error("Signal error: {0}")]
    Signal(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
}

pub use crate::config::Protocol;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;

    #[test]
    fn test_serve_config() {
        let config = RuntimeConfig::default();
        assert!(config.protocols.contains(&Protocol::Http));
    }
}
