//! Mock Connector - For testing and development
//!
//! This connector simulates legacy system interactions for testing.

use super::sdk::{
    A2ATaskPayload, ConnectorConfig, ConnectorError, ConnectorHealth, ConnectorProtocol,
    ConnectorResult, LegacyConnector, LegacyMessage,
};
use std::collections::HashMap;

/// Mock connector for testing.
pub struct MockConnector {
    config: ConnectorConfig,
    should_fail: bool,
    latency_ms: u64,
}

impl MockConnector {
    /// Create a new mock connector.
    pub fn new(name: impl Into<String>) -> Self {
        let mut config = ConnectorConfig::default();
        config.name = name.into();
        config.protocol = ConnectorProtocol::Sql; // Mock as SQL

        Self {
            config,
            should_fail: false,
            latency_ms: 0,
        }
    }

    /// Set whether operations should fail.
    pub fn with_failure(mut self, fail: bool) -> Self {
        self.should_fail = fail;
        self
    }

    /// Set simulated latency.
    pub fn with_latency(mut self, ms: u64) -> Self {
        self.latency_ms = ms;
        self
    }
}

#[async_trait::async_trait]
impl LegacyConnector for MockConnector {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn protocol(&self) -> ConnectorProtocol {
        self.config.protocol
    }

    fn config(&self) -> &ConnectorConfig {
        &self.config
    }

    async fn health_check(&self) -> ConnectorResult<ConnectorHealth> {
        if self.should_fail {
            Ok(ConnectorHealth::unhealthy("Mock failure enabled"))
        } else {
            let mut health = ConnectorHealth::healthy();
            health.latency_ms = Some(self.latency_ms);
            Ok(health)
        }
    }

    fn translate_to_legacy(&self, task: &A2ATaskPayload) -> ConnectorResult<LegacyMessage> {
        if self.should_fail {
            return Err(ConnectorError::ConnectionFailed("Mock failure".into()));
        }

        // Simple mock translation - serialize task as JSON
        let data =
            serde_json::to_vec(task).map_err(|e| ConnectorError::ParseError(e.to_string()))?;

        Ok(LegacyMessage {
            data,
            message_type: format!("MOCK_{}", task.method.to_uppercase()),
            metadata: HashMap::new(),
        })
    }

    fn translate_from_legacy(&self, msg: &LegacyMessage) -> ConnectorResult<A2ATaskPayload> {
        if self.should_fail {
            return Err(ConnectorError::ConnectionFailed("Mock failure".into()));
        }

        // Try to parse as JSON
        serde_json::from_slice(&msg.data).map_err(|e| ConnectorError::ParseError(e.to_string()))
    }

    async fn execute(&self, msg: &LegacyMessage) -> ConnectorResult<LegacyMessage> {
        if self.should_fail {
            return Err(ConnectorError::ConnectionFailed("Mock failure".into()));
        }

        // Simulate latency
        if self.latency_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.latency_ms)).await;
        }

        // Echo back with RESPONSE prefix
        Ok(LegacyMessage {
            data: msg.data.clone(),
            message_type: format!("RESPONSE_{}", msg.message_type),
            metadata: msg.metadata.clone(),
        })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_connector_create() {
        let connector = MockConnector::new("test-mock");
        assert_eq!(connector.name(), "test-mock");
        assert_eq!(connector.protocol(), ConnectorProtocol::Sql);
    }

    #[tokio::test]
    async fn test_mock_connector_health() {
        let connector = MockConnector::new("test");
        let health = connector.health_check().await.unwrap();
        assert!(health.healthy);
    }

    #[tokio::test]
    async fn test_mock_connector_failure_mode() {
        let connector = MockConnector::new("test").with_failure(true);
        let health = connector.health_check().await.unwrap();
        assert!(!health.healthy);
    }

    #[test]
    fn test_mock_translate_to_legacy() {
        let connector = MockConnector::new("test");
        let task = A2ATaskPayload {
            id: "task-1".into(),
            method: "query".into(),
            params: serde_json::json!({}),
            source_agent: None,
            target_agent: None,
        };

        let msg = connector.translate_to_legacy(&task).unwrap();
        assert_eq!(msg.message_type, "MOCK_QUERY");
    }

    #[tokio::test]
    async fn test_mock_execute() {
        let connector = MockConnector::new("test");
        let msg = LegacyMessage {
            data: b"test data".to_vec(),
            message_type: "TEST".into(),
            metadata: HashMap::new(),
        };

        let response = connector.execute(&msg).await.unwrap();
        assert_eq!(response.message_type, "RESPONSE_TEST");
    }
}
