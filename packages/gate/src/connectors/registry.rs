//! Connector Registry - Manages registered legacy connectors
//!
//! Thread-safe registry for connector instances with hot-reload capability.

use super::sdk::{
    ConnectorError, ConnectorHealth, ConnectorProtocol, ConnectorResult, LegacyConnector,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// A registered connector with metadata.
#[derive(Clone)]
pub struct RegisteredConnector {
    /// The connector instance
    pub connector: Arc<dyn LegacyConnector>,
    /// Registration timestamp (Unix ms)
    pub registered_at: u64,
    /// Is connector enabled?
    pub enabled: bool,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Connector registry for managing multiple connectors.
pub struct ConnectorRegistry {
    connectors: RwLock<HashMap<String, RegisteredConnector>>,
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectorRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            connectors: RwLock::new(HashMap::new()),
        }
    }

    /// Register a connector.
    pub fn register(
        &self,
        id: impl Into<String>,
        connector: Arc<dyn LegacyConnector>,
        tags: Vec<String>,
    ) -> ConnectorResult<()> {
        let id = id.into();
        let mut connectors = self.connectors.write();

        if connectors.contains_key(&id) {
            return Err(ConnectorError::Internal(format!(
                "Connector '{}' already registered",
                id
            )));
        }

        connectors.insert(
            id,
            RegisteredConnector {
                connector,
                registered_at: chrono::Utc::now().timestamp_millis() as u64,
                enabled: true,
                tags,
            },
        );

        Ok(())
    }

    /// Unregister a connector.
    pub fn unregister(&self, id: &str) -> ConnectorResult<()> {
        let mut connectors = self.connectors.write();
        connectors
            .remove(id)
            .ok_or_else(|| ConnectorError::Internal(format!("Connector '{}' not found", id)))?;
        Ok(())
    }

    /// Get a connector by ID.
    pub fn get(&self, id: &str) -> Option<RegisteredConnector> {
        let connectors = self.connectors.read();
        connectors.get(id).cloned()
    }

    /// Get all connectors of a specific protocol.
    pub fn by_protocol(&self, protocol: ConnectorProtocol) -> Vec<RegisteredConnector> {
        let connectors = self.connectors.read();
        connectors
            .values()
            .filter(|c| c.connector.protocol() == protocol && c.enabled)
            .cloned()
            .collect()
    }

    /// Get all connector IDs.
    pub fn list_ids(&self) -> Vec<String> {
        let connectors = self.connectors.read();
        connectors.keys().cloned().collect()
    }

    /// Get health status of all connectors.
    pub async fn health_all(&self) -> HashMap<String, ConnectorHealth> {
        // Collect connectors first to release lock
        let connectors: Vec<(String, Arc<dyn LegacyConnector>)> = {
            let lock = self.connectors.read();
            lock.iter()
                .map(|(k, v)| (k.clone(), v.connector.clone()))
                .collect()
        };

        // Iterate securely
        let mut results = HashMap::new();
        for (id, connector) in connectors {
            let health = connector
                .health_check()
                .await
                .unwrap_or_else(|e| ConnectorHealth::unhealthy(e.to_string()));
            results.insert(id, health);
        }
        results
    }

    /// Enable a connector.
    pub fn enable(&self, id: &str) -> ConnectorResult<()> {
        let mut connectors = self.connectors.write();
        let reg = connectors
            .get_mut(id)
            .ok_or_else(|| ConnectorError::Internal(format!("Connector '{}' not found", id)))?;
        reg.enabled = true;
        Ok(())
    }

    /// Disable a connector.
    pub fn disable(&self, id: &str) -> ConnectorResult<()> {
        let mut connectors = self.connectors.write();
        let reg = connectors
            .get_mut(id)
            .ok_or_else(|| ConnectorError::Internal(format!("Connector '{}' not found", id)))?;
        reg.enabled = false;
        Ok(())
    }

    /// Get count of registered connectors.
    pub fn count(&self) -> usize {
        self.connectors.read().len()
    }
}

/// Registry statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_connectors: usize,
    pub enabled_connectors: usize,
    pub protocols: HashMap<String, usize>,
}

impl ConnectorRegistry {
    /// Get registry statistics.
    pub fn stats(&self) -> RegistryStats {
        let connectors = self.connectors.read();

        let mut protocols: HashMap<String, usize> = HashMap::new();
        let mut enabled = 0;

        for reg in connectors.values() {
            if reg.enabled {
                enabled += 1;
            }
            let proto = reg.connector.protocol().name().to_string();
            *protocols.entry(proto).or_insert(0) += 1;
        }

        RegistryStats {
            total_connectors: connectors.len(),
            enabled_connectors: enabled,
            protocols,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::mock::MockConnector;

    #[test]
    fn test_registry_register() {
        let registry = ConnectorRegistry::new();
        let connector = Arc::new(MockConnector::new("test-1"));

        registry
            .register("test-1", connector.clone(), vec!["test".into()])
            .unwrap();

        assert_eq!(registry.count(), 1);
        assert!(registry.get("test-1").is_some());
    }

    #[test]
    fn test_registry_duplicate_error() {
        let registry = ConnectorRegistry::new();
        let connector = Arc::new(MockConnector::new("test-1"));

        registry
            .register("test-1", connector.clone(), vec![])
            .unwrap();
        let result = registry.register("test-1", connector, vec![]);

        assert!(result.is_err());
    }

    #[test]
    fn test_registry_unregister() {
        let registry = ConnectorRegistry::new();
        let connector = Arc::new(MockConnector::new("test-1"));

        registry.register("test-1", connector, vec![]).unwrap();
        registry.unregister("test-1").unwrap();

        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_enable_disable() {
        let registry = ConnectorRegistry::new();
        let connector = Arc::new(MockConnector::new("test-1"));

        registry.register("test-1", connector, vec![]).unwrap();

        registry.disable("test-1").unwrap();
        assert!(!registry.get("test-1").unwrap().enabled);

        registry.enable("test-1").unwrap();
        assert!(registry.get("test-1").unwrap().enabled);
    }

    #[test]
    fn test_registry_stats() {
        let registry = ConnectorRegistry::new();
        registry
            .register("mock-1", Arc::new(MockConnector::new("mock-1")), vec![])
            .unwrap();
        registry
            .register("mock-2", Arc::new(MockConnector::new("mock-2")), vec![])
            .unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_connectors, 2);
        assert_eq!(stats.enabled_connectors, 2);
    }
}
