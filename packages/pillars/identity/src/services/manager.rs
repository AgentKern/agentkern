use crate::models::{AgentRecord, AgentStatus, AgentBudget, AgentUsage, AgentReputation};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, FromRow};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Agent not found: {0}")]
    NotFound(String),
    #[error("Agent already exists: {0}")]
    AlreadyExists(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Agent is terminated")]
    Terminated,
}

/// Configuration for new agents
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_tokens: u64,
    pub max_api_calls: u32,
    pub max_cost_usd: f64,
    pub period_seconds: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_tokens: 1_000_000,   // 1M tokens per day
            max_api_calls: 10_000,   // 10k API calls per day
            max_cost_usd: 100.0,     // $100 per day
            period_seconds: 86400,   // 24 hours
        }
    }
}

/// Agent Manager - CRUD operations for agent lifecycle
pub struct AgentManager {
    pool: PgPool,
    default_config: AgentConfig,
}

impl AgentManager {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            default_config: AgentConfig::default(),
        }
    }

    pub fn with_config(pool: PgPool, config: AgentConfig) -> Self {
        Self {
            pool,
            default_config: config,
        }
    }

    /// Register a new agent
    pub async fn register(
        &self,
        agent_id: &str,
        name: &str,
        version: &str,
        namespace: Option<&str>,
    ) -> Result<AgentRecord, ManagerError> {
        let namespace = namespace.unwrap_or("default");
        let now = Utc::now();

        // Check if exists
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM agent_records WHERE id = $1"
        )
        .bind(agent_id)
        .fetch_one(&self.pool)
        .await?;

        if existing > 0 {
            return Err(ManagerError::AlreadyExists(agent_id.to_string()));
        }

        let budget = AgentBudget {
            max_daily_spend: self.default_config.max_cost_usd,
            max_tokens_per_minute: (self.default_config.max_tokens / 1440) as u32, // per minute
            remaining_credits: self.default_config.max_cost_usd,
        };

        let usage = AgentUsage {
            total_requests: 0,
            total_tokens: 0,
            last_24h_spend: 0.0,
        };

        let reputation = AgentReputation {
            score: 50, // Start at 50/100
            trust_level: "neutral".to_string(),
            flags: vec![],
        };

        sqlx::query(
            r#"
            INSERT INTO agent_records 
            (id, name, namespace, version, status, budget, usage, reputation, created_at, last_active_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(agent_id)
        .bind(name)
        .bind(namespace)
        .bind(version)
        .bind("active")
        .bind(serde_json::to_value(&budget).unwrap())
        .bind(serde_json::to_value(&usage).unwrap())
        .bind(serde_json::to_value(&reputation).unwrap())
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(AgentRecord {
            id: agent_id.to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
            version: version.to_string(),
            status: AgentStatus::Active,
            budget,
            usage,
            reputation,
            created_at: now,
            last_active_at: now,
            terminated_at: None,
            termination_reason: None,
        })
    }

    /// Get an agent by ID
    pub async fn get(&self, agent_id: &str) -> Result<AgentRecord, ManagerError> {
        let row = sqlx::query_as::<_, AgentRecordRow>(
            "SELECT * FROM agent_records WHERE id = $1"
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ManagerError::NotFound(agent_id.to_string()))?;

        Ok(row.into())
    }

    /// List all agents in a namespace
    pub async fn list(&self, namespace: Option<&str>) -> Result<Vec<AgentRecord>, ManagerError> {
        let rows = if let Some(ns) = namespace {
            sqlx::query_as::<_, AgentRecordRow>(
                "SELECT * FROM agent_records WHERE namespace = $1 ORDER BY created_at DESC"
            )
            .bind(ns)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, AgentRecordRow>(
                "SELECT * FROM agent_records ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Update agent status
    pub async fn update_status(
        &self,
        agent_id: &str,
        status: AgentStatus,
        reason: Option<&str>,
    ) -> Result<(), ManagerError> {
        let now = Utc::now();
        let status_str = match status {
            AgentStatus::Active => "active",
            AgentStatus::Suspended => "suspended",
            AgentStatus::Terminated => "terminated",
            AgentStatus::Pending => "pending",
        };

        let result = if status == AgentStatus::Terminated {
            sqlx::query(
                r#"
                UPDATE agent_records 
                SET status = $1, terminated_at = $2, termination_reason = $3, last_active_at = $4
                WHERE id = $5
                "#
            )
            .bind(status_str)
            .bind(now)
            .bind(reason)
            .bind(now)
            .bind(agent_id)
            .execute(&self.pool)
            .await?
        } else {
            sqlx::query(
                "UPDATE agent_records SET status = $1, last_active_at = $2 WHERE id = $3"
            )
            .bind(status_str)
            .bind(now)
            .bind(agent_id)
            .execute(&self.pool)
            .await?
        };

        if result.rows_affected() == 0 {
            return Err(ManagerError::NotFound(agent_id.to_string()));
        }

        Ok(())
    }

    /// Delete an agent permanently
    pub async fn delete(&self, agent_id: &str) -> Result<(), ManagerError> {
        let result = sqlx::query("DELETE FROM agent_records WHERE id = $1")
            .bind(agent_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ManagerError::NotFound(agent_id.to_string()));
        }

        Ok(())
    }

    /// Record a successful action (updates usage and reputation)
    pub async fn record_success(&self, agent_id: &str, tokens_used: u64) -> Result<(), ManagerError> {
        sqlx::query(
            r#"
            UPDATE agent_records 
            SET 
                usage = jsonb_set(
                    jsonb_set(usage, '{total_requests}', to_jsonb((usage->>'total_requests')::bigint + 1)),
                    '{total_tokens}', to_jsonb((usage->>'total_tokens')::bigint + $1)
                ),
                reputation = jsonb_set(reputation, '{score}', to_jsonb(LEAST(100, (reputation->>'score')::int + 1))),
                last_active_at = $2
            WHERE id = $3
            "#
        )
        .bind(tokens_used as i64)
        .bind(Utc::now())
        .bind(agent_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Record a failed action (updates reputation)
    pub async fn record_failure(&self, agent_id: &str) -> Result<(), ManagerError> {
        sqlx::query(
            r#"
            UPDATE agent_records 
            SET 
                reputation = jsonb_set(reputation, '{score}', to_jsonb(GREATEST(0, (reputation->>'score')::int - 10))),
                last_active_at = $1
            WHERE id = $2
            "#
        )
        .bind(Utc::now())
        .bind(agent_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Internal row structure for SQLx
#[derive(FromRow)]
struct AgentRecordRow {
    id: String,
    name: String,
    namespace: String,
    version: String,
    status: String,
    budget: serde_json::Value,
    usage: serde_json::Value,
    reputation: serde_json::Value,
    created_at: DateTime<Utc>,
    last_active_at: DateTime<Utc>,
    terminated_at: Option<DateTime<Utc>>,
    termination_reason: Option<String>,
}

impl From<AgentRecordRow> for AgentRecord {
    fn from(row: AgentRecordRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            namespace: row.namespace,
            version: row.version,
            status: match row.status.as_str() {
                "active" => AgentStatus::Active,
                "suspended" => AgentStatus::Suspended,
                "terminated" => AgentStatus::Terminated,
                "pending" => AgentStatus::Pending,
                _ => AgentStatus::Active,
            },
            budget: serde_json::from_value(row.budget).unwrap_or_default(),
            usage: serde_json::from_value(row.usage).unwrap_or_default(),
            reputation: serde_json::from_value(row.reputation).unwrap_or_default(),
            created_at: row.created_at,
            last_active_at: row.last_active_at,
            terminated_at: row.terminated_at,
            termination_reason: row.termination_reason,
        }
    }
}
