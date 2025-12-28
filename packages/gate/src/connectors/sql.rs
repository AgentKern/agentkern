//! SQL Connector - Generic SQL/JDBC bridge (Open Source)
//!
//! Per Licensing: This is the FREE connector included in Community tier.
//! Supports basic SQL operations through A2A protocol translation.
//! Default: Uses `sqlx::AnyPool` for database-agnostic connectivity.

use super::sdk::{
    A2ATaskPayload, ConnectorConfig, ConnectorError, ConnectorHealth, ConnectorProtocol,
    ConnectorResult, LegacyConnector, LegacyMessage,
};
use serde::{Deserialize, Serialize};
use sqlx::{AnyPool, Column, Row, TypeInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    pool: Arc<RwLock<Option<AnyPool>>>,
}

impl SqlConnector {
    /// Create a new SQL connector.
    pub fn new(endpoint: impl Into<String>) -> Self {
        let mut config = ConnectorConfig::default();
        config.name = "SQL Connector".to_string();
        config.protocol = ConnectorProtocol::Sql;
        config.endpoint = endpoint.into();

        Self {
            config,
            pool: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: ConnectorConfig) -> Self {
        Self {
            config,
            pool: Arc::new(RwLock::new(None)),
        }
    }

    /// Connect to the database.
    pub async fn connect(&self) -> ConnectorResult<()> {
        let endpoint = &self.config.endpoint;
        if endpoint.is_empty() {
            return Ok(());
        }

        // sqlx::AnyPool requires protocol prefix (postgres://, mysql://, sqlite://)
        let pool = AnyPool::connect(endpoint)
            .await
            .map_err(|e| ConnectorError::Internal(format!("Connection error: {}", e)))?;

        let mut lock = self.pool.write().await;
        *lock = Some(pool);

        Ok(())
    }

    /// Parse SQL from A2A task params.
    fn parse_query(&self, params: &serde_json::Value) -> ConnectorResult<SqlQuery> {
        // Extract SQL string
        let sql = params
            .get("sql")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConnectorError::ParseError("Missing 'sql' parameter".into()))?
            .to_string();

        // Determine query type
        let query_type = self.detect_query_type(&sql);

        // Extract optional params
        let query_params = params
            .get("params")
            .and_then(|v| v.as_array())
            .map(|arr| arr.clone())
            .unwrap_or_default();

        let timeout = params.get("timeout_ms").and_then(|v| v.as_u64());

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

    /// Map sqlx row to vector of JSON values.
    fn row_to_json(row: &sqlx::any::AnyRow, columns: &[String]) -> Vec<serde_json::Value> {
        let mut values = Vec::new();
        for (i, _) in columns.iter().enumerate() {
            // Very basic mapping - improve with types
            if let Ok(v) = row.try_get::<i64, _>(i) {
                values.push(serde_json::Value::Number(v.into()));
            } else if let Ok(v) = row.try_get::<f64, _>(i) {
                if let Some(n) = serde_json::Number::from_f64(v) {
                    values.push(serde_json::Value::Number(n));
                } else {
                    values.push(serde_json::Value::Null);
                }
            } else if let Ok(v) = row.try_get::<bool, _>(i) {
                values.push(serde_json::Value::Bool(v));
            } else if let Ok(v) = row.try_get::<String, _>(i) {
                values.push(serde_json::Value::String(v));
            } else {
                // Fallback to null (or maybe string?)
                values.push(serde_json::Value::Null);
            }
        }
        values
    }
}

#[async_trait::async_trait]
impl LegacyConnector for SqlConnector {
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
        let pool = self.pool.read().await;

        if let Some(pool) = pool.as_ref() {
            // Execute simple query to test connection
            match sqlx::query("SELECT 1").execute(pool).await {
                Ok(_) => Ok(ConnectorHealth::healthy()),
                Err(e) => Ok(ConnectorHealth::unhealthy(format!("DB Ping failed: {}", e))),
            }
        } else {
            // Not connected
            Ok(ConnectorHealth {
                healthy: false,
                last_success_ms: None,
                last_failure_ms: None,
                latency_ms: None,
                error_count: 0,
                message: Some("Not connected. Call connect() first.".into()),
            })
        }
    }

    fn translate_to_legacy(&self, task: &A2ATaskPayload) -> ConnectorResult<LegacyMessage> {
        let query = self.parse_query(&task.params)?;

        let data =
            serde_json::to_vec(&query).map_err(|e| ConnectorError::ParseError(e.to_string()))?;

        let mut metadata = HashMap::new();
        metadata.insert("task_id".to_string(), task.id.clone());

        Ok(LegacyMessage {
            data,
            message_type: match query.query_type {
                SqlQueryType::Select => "SQL_SELECT".into(),
                _ => "SQL_EXEC".into(),
            },
            metadata,
        })
    }

    fn translate_from_legacy(&self, msg: &LegacyMessage) -> ConnectorResult<A2ATaskPayload> {
        // SQL usually returns results, this might be a notification?
        // Basic impl: wrap message data as params
        Ok(A2ATaskPayload {
            id: msg.metadata.get("task_id").cloned().unwrap_or_default(),
            method: "sql_result".into(),
            params: serde_json::from_slice(&msg.data)
                .map_err(|e| ConnectorError::ParseError(e.to_string()))?,
            source_agent: None,
            target_agent: None,
        })
    }

    async fn execute(&self, msg: &LegacyMessage) -> ConnectorResult<LegacyMessage> {
        // Deserialize Query
        let query: SqlQuery = serde_json::from_slice(&msg.data)
            .map_err(|e| ConnectorError::ParseError(e.to_string()))?;

        // Ensure connected
        let pool_guard = self.pool.read().await;
        let pool = pool_guard
            .as_ref()
            .ok_or_else(|| ConnectorError::Internal("Database not connected".into()))?;

        let start = std::time::Instant::now();

        // P1 Fix: Parameterized Queries to prevent SQL injection
        let mut query_builder = sqlx::query(&query.sql);

        // Bind parameters dynamically
        for param in &query.params {
            match param {
                serde_json::Value::String(s) => {
                    query_builder = query_builder.bind(s);
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query_builder = query_builder.bind(i);
                    } else if let Some(f) = n.as_f64() {
                        query_builder = query_builder.bind(f);
                    }
                }
                serde_json::Value::Bool(b) => {
                    query_builder = query_builder.bind(b);
                }
                _ => {
                    // Skip unsupported types for now or bind as Null if supported by driver
                }
            }
        }

        let rows = query_builder
            .fetch_all(pool)
            .await
            .map_err(|e| ConnectorError::Internal(format!("Execution error: {}", e)))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Process results
        let mut result_rows = Vec::new();
        let mut columns = Vec::new();

        if let Some(first_row) = rows.first() {
            for col in first_row.columns() {
                columns.push(col.name().to_string());
            }
        }

        for row in rows {
            result_rows.push(Self::row_to_json(&row, &columns));
        }

        let result = SqlResult {
            rows_affected: result_rows.len() as u64, // simplistic
            columns,
            rows: result_rows,
            execution_time_ms,
        };

        let data = serde_json::to_vec(&result)
            .map_err(|e| ConnectorError::Internal(format!("Serialization error: {}", e)))?;

        Ok(LegacyMessage {
            data,
            message_type: "SQL_RESULT".into(),
            metadata: msg.metadata.clone(),
        })
    }
}
