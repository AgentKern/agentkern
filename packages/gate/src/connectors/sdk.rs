//! Connector SDK - Core traits and types for legacy system integration
//!
//! Per MANDATE.md Section 6: WASM Sandboxing for connectors
//! Per Zero-Trust: All connectors validate credentials through policy

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Result type for connector operations.
pub type ConnectorResult<T> = Result<T, ConnectorError>;

/// Connector errors.
#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Timeout: operation took longer than {0}ms")]
    Timeout(u64),
    
    #[error("Not supported: {0}")]
    NotSupported(String),
    
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Protocol types for legacy connectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorProtocol {
    /// SAP Remote Function Call
    SapRfc,
    /// SAP BAPI
    SapBapi,
    /// SAP OData v4 (S/4HANA)
    SapOdata,
    /// SAP IDOC
    SapIdoc,
    /// SWIFT MT (FIN messages)
    SwiftMt,
    /// SWIFT MX (ISO 20022 XML)
    SwiftMx,
    /// SWIFT GPI
    SwiftGpi,
    /// IBM CICS
    IbmCics,
    /// IBM IMS
    IbmIms,
    /// IBM MQ
    IbmMq,
    /// Generic JDBC/SQL
    Sql,
    /// Oracle OCI
    OracleOci,
    /// Salesforce API
    Salesforce,
    /// Custom protocol
    Custom(u32),
}

impl ConnectorProtocol {
    /// Get human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::SapRfc => "SAP RFC",
            Self::SapBapi => "SAP BAPI",
            Self::SapOdata => "SAP OData",
            Self::SapIdoc => "SAP IDOC",
            Self::SwiftMt => "SWIFT MT",
            Self::SwiftMx => "SWIFT MX",
            Self::SwiftGpi => "SWIFT GPI",
            Self::IbmCics => "IBM CICS",
            Self::IbmIms => "IBM IMS",
            Self::IbmMq => "IBM MQ",
            Self::Sql => "SQL/JDBC",
            Self::OracleOci => "Oracle OCI",
            Self::Salesforce => "Salesforce",
            Self::Custom(_) => "Custom",
        }
    }
    
    /// Check if protocol requires enterprise license.
    pub fn requires_enterprise(&self) -> bool {
        match self {
            Self::Sql => false, // Free tier
            _ => true, // All others require enterprise
        }
    }
}

/// Connector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    /// Unique connector ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Protocol type
    pub protocol: ConnectorProtocol,
    /// Connection endpoint (URL, hostname, etc.)
    pub endpoint: String,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum retries on failure
    pub max_retries: u32,
    /// Additional protocol-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Default Connector".to_string(),
            protocol: ConnectorProtocol::Sql,
            endpoint: "localhost".to_string(),
            timeout_ms: 30_000,
            max_retries: 3,
            settings: HashMap::new(),
        }
    }
}

/// Connector health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorHealth {
    /// Is connector connected and ready?
    pub healthy: bool,
    /// Last successful operation timestamp (Unix ms)
    pub last_success_ms: Option<u64>,
    /// Last failure timestamp (Unix ms)
    pub last_failure_ms: Option<u64>,
    /// Current latency in ms
    pub latency_ms: Option<u64>,
    /// Error count in last hour
    pub error_count: u32,
    /// Additional status info
    pub message: Option<String>,
}

impl ConnectorHealth {
    /// Create healthy status.
    pub fn healthy() -> Self {
        Self {
            healthy: true,
            last_success_ms: Some(chrono::Utc::now().timestamp_millis() as u64),
            last_failure_ms: None,
            latency_ms: None,
            error_count: 0,
            message: None,
        }
    }
    
    /// Create unhealthy status.
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            last_success_ms: None,
            last_failure_ms: Some(chrono::Utc::now().timestamp_millis() as u64),
            latency_ms: None,
            error_count: 1,
            message: Some(message.into()),
        }
    }
}

/// A2A Task representation for connector translation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ATaskPayload {
    /// Task ID
    pub id: String,
    /// Task type/method
    pub method: String,
    /// Task parameters
    pub params: serde_json::Value,
    /// Source agent
    pub source_agent: Option<String>,
    /// Target agent
    pub target_agent: Option<String>,
}

/// Legacy message representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyMessage {
    /// Raw message bytes
    pub data: Vec<u8>,
    /// Protocol-specific message type
    pub message_type: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Core trait for all legacy system connectors.
///
/// Connectors translate between A2A protocol and legacy protocols,
/// enabling AI agents to interact with enterprise systems.
///
/// # WASM Safety
/// 
/// All connectors are designed to run in WASM isolation.
/// They must not use any system calls directly.
#[async_trait::async_trait]
pub trait LegacyConnector: Send + Sync {
    /// Get connector name.
    fn name(&self) -> &str;
    
    /// Get connector protocol.
    fn protocol(&self) -> ConnectorProtocol;
    
    /// Get current configuration.
    fn config(&self) -> &ConnectorConfig;
    
    /// Check connector health.
    async fn health_check(&self) -> ConnectorResult<ConnectorHealth>;
    
    /// Translate A2A task to legacy message.
    fn translate_to_legacy(&self, task: &A2ATaskPayload) -> ConnectorResult<LegacyMessage>;
    
    /// Translate legacy message to A2A task.
    fn translate_from_legacy(&self, msg: &LegacyMessage) -> ConnectorResult<A2ATaskPayload>;
    
    /// Execute a query/operation on the legacy system.
    async fn execute(&self, msg: &LegacyMessage) -> ConnectorResult<LegacyMessage>;
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_protocol_names() {
        assert_eq!(ConnectorProtocol::SapRfc.name(), "SAP RFC");
        assert_eq!(ConnectorProtocol::SwiftMt.name(), "SWIFT MT");
        assert_eq!(ConnectorProtocol::Sql.name(), "SQL/JDBC");
    }

    #[test]
    fn test_connector_protocol_enterprise() {
        assert!(!ConnectorProtocol::Sql.requires_enterprise());
        assert!(ConnectorProtocol::SapRfc.requires_enterprise());
        assert!(ConnectorProtocol::SwiftMt.requires_enterprise());
    }

    #[test]
    fn test_connector_config_default() {
        let config = ConnectorConfig::default();
        assert_eq!(config.protocol, ConnectorProtocol::Sql);
        assert_eq!(config.timeout_ms, 30_000);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_connector_health() {
        let healthy = ConnectorHealth::healthy();
        assert!(healthy.healthy);
        assert!(healthy.last_success_ms.is_some());
        
        let unhealthy = ConnectorHealth::unhealthy("Connection refused");
        assert!(!unhealthy.healthy);
        assert_eq!(unhealthy.message, Some("Connection refused".to_string()));
    }

    #[test]
    fn test_a2a_task_payload() {
        let task = A2ATaskPayload {
            id: "task-1".to_string(),
            method: "query".to_string(),
            params: serde_json::json!({"sql": "SELECT * FROM users"}),
            source_agent: Some("agent-a".to_string()),
            target_agent: None,
        };
        
        assert_eq!(task.method, "query");
    }

    #[test]
    fn test_legacy_message() {
        let msg = LegacyMessage {
            data: b"MT103 test".to_vec(),
            message_type: "MT103".to_string(),
            metadata: HashMap::new(),
        };
        
        assert_eq!(msg.message_type, "MT103");
    }
}
