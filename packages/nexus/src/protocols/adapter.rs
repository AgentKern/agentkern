//! Protocol Adapter Trait and Registry
//!
//! This is the core extensibility point for Nexus.
//! New protocols only need to implement `ProtocolAdapter`.

use crate::error::NexusError;
use crate::types::{NexusMessage, Protocol};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Protocol adapter trait.
///
/// Implement this trait to add support for a new protocol.
///
/// # Example
///
/// ```rust,ignore
/// struct MyProtocolAdapter;
///
/// #[async_trait]
/// impl ProtocolAdapter for MyProtocolAdapter {
///     fn protocol(&self) -> Protocol {
///         Protocol::Custom(42)
///     }
///
///     fn detect(&self, raw: &[u8]) -> bool {
///         // Check if this looks like my protocol
///         raw.starts_with(b"MYPROTO")
///     }
///
///     async fn parse(&self, raw: &[u8]) -> Result<NexusMessage, NexusError> {
///         // Parse into unified format
///     }
///
///     async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
///         // Convert to wire format
///     }
/// }
/// ```
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Get the protocol this adapter handles.
    fn protocol(&self) -> Protocol;

    /// Get protocol name for logging.
    fn name(&self) -> &'static str {
        self.protocol().name()
    }

    /// Detect if raw bytes match this protocol.
    ///
    /// Used for auto-detection when protocol is unknown.
    fn detect(&self, raw: &[u8]) -> bool;

    /// Parse raw bytes into unified NexusMessage.
    async fn parse(&self, raw: &[u8]) -> Result<NexusMessage, NexusError>;

    /// Serialize NexusMessage to protocol wire format.
    async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError>;

    /// Get protocol version supported.
    fn version(&self) -> &'static str {
        "1.0"
    }

    /// Check if protocol supports streaming.
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Check if protocol supports bidirectional communication.
    fn is_bidirectional(&self) -> bool {
        true
    }
}

/// Registry of protocol adapters.
///
/// Maintains a collection of adapters and provides:
/// - Protocol detection (sniffing)
/// - Adapter lookup by protocol
/// - Dynamic registration
pub struct AdapterRegistry {
    adapters: HashMap<Protocol, Arc<dyn ProtocolAdapter>>,
    detection_order: Vec<Protocol>,
}

impl AdapterRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
            detection_order: Vec::new(),
        }
    }

    /// Register an adapter.
    pub fn register(&mut self, adapter: Box<dyn ProtocolAdapter>) {
        let protocol = adapter.protocol();
        tracing::info!(protocol = ?protocol, "Registered protocol adapter");
        self.detection_order.push(protocol);
        self.adapters.insert(protocol, Arc::from(adapter));
    }

    /// Get adapter for a specific protocol.
    pub fn get(&self, protocol: &Protocol) -> Result<Arc<dyn ProtocolAdapter>, NexusError> {
        self.adapters
            .get(protocol)
            .cloned()
            .ok_or(NexusError::AdapterNotRegistered {
                protocol: *protocol,
            })
    }

    /// Auto-detect protocol from raw bytes.
    pub fn detect(&self, raw: &[u8]) -> Result<Protocol, NexusError> {
        for protocol in &self.detection_order {
            if let Some(adapter) = self.adapters.get(protocol) {
                if adapter.detect(raw) {
                    return Ok(*protocol);
                }
            }
        }
        Err(NexusError::UnknownProtocol)
    }

    /// Get count of registered adapters.
    pub fn count(&self) -> usize {
        self.adapters.len()
    }

    /// Check if a protocol is supported.
    pub fn supports(&self, protocol: &Protocol) -> bool {
        self.adapters.contains_key(protocol)
    }

    /// List all registered protocols.
    pub fn protocols(&self) -> Vec<Protocol> {
        self.adapters.keys().cloned().collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockAdapter {
        proto: Protocol,
    }

    #[async_trait]
    impl ProtocolAdapter for MockAdapter {
        fn protocol(&self) -> Protocol {
            self.proto
        }

        fn detect(&self, raw: &[u8]) -> bool {
            match self.proto {
                Protocol::GoogleA2A => raw.starts_with(b"{\"jsonrpc\":\"2.0\""),
                _ => false,
            }
        }

        async fn parse(&self, _raw: &[u8]) -> Result<NexusMessage, NexusError> {
            Ok(NexusMessage::default())
        }

        async fn serialize(&self, _msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_registry() {
        let mut registry = AdapterRegistry::new();

        registry.register(Box::new(MockAdapter {
            proto: Protocol::GoogleA2A,
        }));

        assert_eq!(registry.count(), 1);
        assert!(registry.supports(&Protocol::GoogleA2A));
        assert!(!registry.supports(&Protocol::AnthropicMCP));
    }

    #[test]
    fn test_protocol_detection() {
        let mut registry = AdapterRegistry::new();
        registry.register(Box::new(MockAdapter {
            proto: Protocol::GoogleA2A,
        }));

        let a2a_msg = b"{\"jsonrpc\":\"2.0\",\"method\":\"tasks/send\"}";
        let detected = registry.detect(a2a_msg);

        assert!(matches!(detected, Ok(Protocol::GoogleA2A)));
    }

    #[tokio::test]
    async fn test_adapter_lookup() {
        let mut registry = AdapterRegistry::new();
        registry.register(Box::new(MockAdapter {
            proto: Protocol::GoogleA2A,
        }));

        let adapter = registry.get(&Protocol::GoogleA2A).unwrap();
        assert_eq!(adapter.protocol(), Protocol::GoogleA2A);
    }
}
