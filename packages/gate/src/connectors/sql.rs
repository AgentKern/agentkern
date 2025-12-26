//! SQL Connector - Generic SQL/JDBC bridge (Open Source)
//!
//! Per Licensing: This is the FREE connector included in Community tier.
//! Supports basic SQL operations through A2A protocol translation.

use super::sdk::{
    LegacyConnector, ConnectorProtocol, ConnectorConfig, ConnectorHealth,
    ConnectorResult, ConnectorError, A2ATaskPayload, LegacyMessage,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SQL query types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqlQueryType {
    Select,
    Insert,
    Update,
    Delete,
    Execute, // Stored procedure
}

/// SQL query parsed from A2A task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQuery {
    pub query_type: SqlQueryType,
    pub sql: String,
    pub params: Vec<serde_json::Value>,
    pub timeout_ms: Option<u64>,
}

/// SQL result for A2A response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlResult {
    pub rows_affected: u64,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub execution_time_ms: u64,
}

/// Generic SQL connector.
pub struct SqlConnector {
    config: ConnectorConfig,
}

impl SqlConnector {
    /// Create a new SQL connector.
    pub fn new(endpoint: impl Into<String>) -> Self {
        let mut config = ConnectorConfig::default();
        config.name = "SQL Connector".to_string();
        config.protocol = ConnectorProtocol::Sql;
        config.endpoint = endpoint.into();
        
        Self { config }
    }
    
    /// Create with custom config.
    pub fn with_config(config: ConnectorConfig) -> Self {
        Self { config }
    }
    
    /// Parse SQL from A2A task params.
    fn parse_query(&self, params: &serde_json::Value) -> ConnectorResult<SqlQuery> {
        // Extract SQL string
        let sql = params.get("sql")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::ParseError("Missing 'sql' parameter".into()))?
            .to_string();
        
        // Determine query type
        let query_type = self.detect_query_type(&sql);
        
        // Extract optional params
        let query_params = params.get("params")
            .and_then(|v| v.as_array())
            .map(|arr| arr.clone())
            .unwrap_or_default();
        
        let timeout = params.get("timeout_ms")
            .and_then(|v| v.as_u64());
        
        Ok(SqlQuery {
            query_type,
            sql,
            params: query_params,
            timeout_ms: timeout,
        })
    }
    
    /// Detect query type from SQL.
    fn detect_query_type(&self, sql: &str) -> SqlQueryType {
        let upper = sql.trim().to_uppercase();
        if upper.starts_with("SELECT") {
            SqlQueryType::Select
        } else if upper.starts_with("INSERT") {
            SqlQueryType::Insert
        } else if upper.starts_with("UPDATE") {
            SqlQueryType::Update
        } else if upper.starts_with("DELETE") {
            SqlQueryType::Delete
        } else {
            SqlQueryType::Execute
        }
    }
}

impl LegacyConnector for SqlConnector {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn protocol(&self) -> ConnectorProtocol {
        ConnectorProtocol::Sql
    }
    
    fn config(&self) -> &ConnectorConfig {
        &self.config
    }
    
    fn health_check(&self) -> ConnectorResult<ConnectorHealth> {
        // In production, this would test the connection
        // For now, return healthy
        Ok(ConnectorHealth::healthy())
    }
    
    fn translate_to_legacy(&self, task: &A2ATaskPayload) -> ConnectorResult<LegacyMessage> {
        let query = self.parse_query(&task.params)?;
        
        let data = serde_json::to_vec(&query)
            .map_err(|e| ConnectorError::ParseError(e.to_string()))?;
        
        let mut metadata = HashMap::new();
        metadata.insert("query_type".to_string(), format!("{:?}", query.query_type));
        metadata.insert("task_id".to_string(), task.id.clone());
        
        Ok(LegacyMessage {
            data,
            message_type: format!("SQL_{:?}", query.query_type).to_uppercase(),
            metadata,
        })
    }
    
    fn translate_from_legacy(&self, msg: &LegacyMessage) -> ConnectorResult<A2ATaskPayload> {
        // Parse SQL result from legacy message
        let result: SqlResult = serde_json::from_slice(&msg.data)
            .map_err(|e| ConnectorError::ParseError(e.to_string()))?;
        
        Ok(A2ATaskPayload {
            id: msg.metadata.get("task_id").cloned().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            method: "sql_result".to_string(),
            params: serde_json::to_value(&result)
                .map_err(|e| ConnectorError::ParseError(e.to_string()))?,
            source_agent: None,
            target_agent: None,
        })
    }
    
    fn execute(&self, msg: &LegacyMessage) -> ConnectorResult<LegacyMessage> {
        // Parse query
        let query: SqlQuery = serde_json::from_slice(&msg.data)
            .map_err(|e| ConnectorError::ParseError(e.to_string()))?;
        
        // In production, this would execute against a real database
        // For now, return a mock result
        let result = SqlResult {
            rows_affected: 0,
            columns: vec![],
            rows: vec![],
            execution_time_ms: 1,
        };
        
        let data = serde_json::to_vec(&result)
            .map_err(|e| ConnectorError::Internal(e.to_string()))?;
        
        Ok(LegacyMessage {
            data,
            message_type: "SQL_RESULT".to_string(),
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
    fn test_sql_connector_create() {
        let connector = SqlConnector::new("localhost:5432");
        assert_eq!(connector.protocol(), ConnectorProtocol::Sql);
        assert!(!connector.protocol().requires_enterprise());
    }

    #[test]
    fn test_detect_query_type() {
        let connector = SqlConnector::new("localhost");
        
        assert_eq!(connector.detect_query_type("SELECT * FROM users"), SqlQueryType::Select);
        assert_eq!(connector.detect_query_type("INSERT INTO users"), SqlQueryType::Insert);
        assert_eq!(connector.detect_query_type("UPDATE users SET"), SqlQueryType::Update);
        assert_eq!(connector.detect_query_type("DELETE FROM users"), SqlQueryType::Delete);
        assert_eq!(connector.detect_query_type("CALL stored_proc()"), SqlQueryType::Execute);
    }

    #[test]
    fn test_translate_to_legacy() {
        let connector = SqlConnector::new("localhost");
        let task = A2ATaskPayload {
            id: "task-1".into(),
            method: "query".into(),
            params: serde_json::json!({
                "sql": "SELECT * FROM users WHERE id = ?",
                "params": [1]
            }),
            source_agent: None,
            target_agent: None,
        };
        
        let msg = connector.translate_to_legacy(&task).unwrap();
        assert_eq!(msg.message_type, "SQL_SELECT");
        assert_eq!(msg.metadata.get("task_id"), Some(&"task-1".to_string()));
    }

    #[test]
    fn test_translate_missing_sql() {
        let connector = SqlConnector::new("localhost");
        let task = A2ATaskPayload {
            id: "task-1".into(),
            method: "query".into(),
            params: serde_json::json!({}), // Missing sql
            source_agent: None,
            target_agent: None,
        };
        
        let result = connector.translate_to_legacy(&task);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute() {
        let connector = SqlConnector::new("localhost");
        let query = SqlQuery {
            query_type: SqlQueryType::Select,
            sql: "SELECT 1".into(),
            params: vec![],
            timeout_ms: None,
        };
        
        let msg = LegacyMessage {
            data: serde_json::to_vec(&query).unwrap(),
            message_type: "SQL_SELECT".into(),
            metadata: HashMap::new(),
        };
        
        let result = connector.execute(&msg).unwrap();
        assert_eq!(result.message_type, "SQL_RESULT");
    }

    #[test]
    fn test_health_check() {
        let connector = SqlConnector::new("localhost");
        let health = connector.health_check().unwrap();
        assert!(health.healthy);
    }
}
