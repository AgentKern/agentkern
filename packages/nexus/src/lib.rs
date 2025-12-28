#![allow(unused)]
#![allow(dead_code)]
//! AgentKern Nexus: Universal Agent Protocol Gateway
//!
//! Per MANDATE.md Section 1: "We do not copy; we innovate."
//! Per Roadmap: "A2A Protocol Gateway - make agents from different vendors talk"
//!
//! Nexus is the protocol translation layer that allows AgentKern to communicate
//! with agents using ANY protocol - current or future.
//!
//! # Supported Protocols
//!
//! | Protocol | Feature Flag | Status |
//! |----------|--------------|--------|
//! | Google A2A | `a2a` | ✅ Stable |
//! | Anthropic MCP | `mcp` | ✅ Stable |
//! | W3C ANP | `anp` | 🟡 Beta |
//! | ECMA NLIP | `nlip` | 🟡 Beta |
//! | NEAR AITP | `aitp` | 🟡 Beta |
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Nexus Gateway                          │
//! ├────────┬────────┬────────┬────────┬────────┬───────────────┤
//! │  A2A   │  MCP   │  ANP   │  NLIP  │  AITP  │  [Future...]  │
//! │ Adapter│ Adapter│ Adapter│ Adapter│ Adapter│   Pluggable   │
//! ├────────┴────────┴────────┴────────┴────────┴───────────────┤
//! │              Unified Message Bus (NexusMessage)             │
//! ├─────────────────────────────────────────────────────────────┤
//! │         Protocol Registry + Dynamic Adapter Loading         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_nexus::{Nexus, Protocol, AgentCard};
//!
//! let nexus = Nexus::new();
//! nexus.register_adapter(A2AAdapter::new())?;
//! nexus.register_adapter(MCPAdapter::new())?;
//!
//! // Incoming A2A message auto-translates to AgentKern native
//! let msg = nexus.receive(incoming_bytes).await?;
//! ```

pub mod types;
pub mod agent_card;
pub mod protocols;
pub mod router;
pub mod discovery;
pub mod registry;
pub mod error;
pub mod marketplace;

// Re-exports
pub use types::*;
pub use agent_card::AgentCard;
pub use protocols::{Protocol, ProtocolAdapter, AdapterRegistry};
pub use router::TaskRouter;
pub use discovery::AgentDiscovery;
pub use registry::AgentRegistry;
pub use error::NexusError;
pub use marketplace::{Marketplace, TaskAuction, Bid, Settlement};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Nexus Gateway - Universal Protocol Translation
pub struct Nexus {
    /// Registered protocol adapters
    adapters: Arc<RwLock<AdapterRegistry>>,
    /// Agent registry
    agents: Arc<AgentRegistry>,
    /// Task router
    router: Arc<TaskRouter>,
    /// Discovery service
    discovery: Arc<AgentDiscovery>,
}

impl Nexus {
    /// Create a new Nexus gateway with default adapters.
    pub fn new() -> Self {
        let adapters = Arc::new(RwLock::new(AdapterRegistry::new()));
        let agents = Arc::new(AgentRegistry::new());
        let router = Arc::new(TaskRouter::new(agents.clone()));
        let discovery = Arc::new(AgentDiscovery::new(agents.clone()));
        
        Self {
            adapters,
            agents,
            router,
            discovery,
        }
    }

    /// Register a protocol adapter.
    pub async fn register_adapter<A: ProtocolAdapter + 'static>(&self, adapter: A) {
        let mut adapters = self.adapters.write().await;
        adapters.register(Box::new(adapter));
    }

    /// Register an agent in the registry.
    pub async fn register_agent(&self, card: AgentCard) -> Result<(), NexusError> {
        self.agents.register(card).await
    }

    /// Receive and translate an incoming message.
    pub async fn receive(&self, raw: &[u8]) -> Result<NexusMessage, NexusError> {
        let adapters = self.adapters.read().await;
        
        // Auto-detect protocol
        let protocol = adapters.detect(raw)?;
        
        // Parse using appropriate adapter
        let adapter = adapters.get(&protocol)?;
        adapter.parse(raw).await
    }

    /// Send a message, translating to target protocol.
    pub async fn send(
        &self,
        msg: &NexusMessage,
        target_protocol: Protocol,
    ) -> Result<Vec<u8>, NexusError> {
        let adapters = self.adapters.read().await;
        let adapter = adapters.get(&target_protocol)?;
        adapter.serialize(msg).await
    }

    /// Route a task to the best matching agent.
    pub async fn route(&self, task: &Task) -> Result<AgentCard, NexusError> {
        self.router.find_best_agent(task).await
    }

    /// Get agent discovery service.
    pub fn discovery(&self) -> Arc<AgentDiscovery> {
        self.discovery.clone()
    }

    /// Get agent registry.
    pub fn registry(&self) -> Arc<AgentRegistry> {
        self.agents.clone()
    }
}

impl Default for Nexus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nexus_creation() {
        let nexus = Nexus::new();
        assert!(nexus.adapters.read().await.count() == 0);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let nexus = Nexus::new();
        
        let card = AgentCard {
            id: "test-agent".into(),
            name: "Test Agent".into(),
            description: "A test agent".into(),
            url: "http://localhost:8080".into(),
            version: "1.0.0".into(),
            capabilities: vec![],
            skills: vec![],
            ..Default::default()
        };
        
        nexus.register_agent(card).await.unwrap();
        
        let found = nexus.agents.get("test-agent").await;
        assert!(found.is_some());
    }
}
