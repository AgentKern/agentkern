use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Agent account status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_status", rename_all = "lowercase")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] // Matches Node.js enum formatting if needed, or lowercase
pub enum AgentStatus {
    Active,
    Suspended,
    Terminated,
    Pending,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Agent financial budget (JSONB)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentBudget {
    pub max_daily_spend: f64,
    pub max_tokens_per_minute: u32,
    pub remaining_credits: f64,
}

/// Agent usage statistics (JSONB)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentUsage {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub last_24h_spend: f64,
}

/// Agent reputation metrics (JSONB)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentReputation {
    pub score: u8,
    pub trust_level: String,
    pub flags: Vec<String>,
}

/// The Agent Identity Record
///
/// Corresponds to `agent_records` table.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentRecord {
    /// Unique Agent ID (UUID or String)
    pub id: String,

    /// Display Name
    pub name: String,

    /// Namespace isolation
    pub namespace: String,

    /// Semantic Version
    pub version: String,

    /// Lifecycle Status
    pub status: AgentStatus,

    /// Resource Budget (JSONB)
    #[sqlx(json)]
    pub budget: AgentBudget,

    /// Usage Stats (JSONB)
    #[sqlx(json)]
    pub usage: AgentUsage,

    /// Reputation & Trust (JSONB)
    #[sqlx(json)]
    pub reputation: AgentReputation,

    /// Creation Timestamp
    pub created_at: DateTime<Utc>,

    /// Last Active Timestamp
    pub last_active_at: DateTime<Utc>,

    /// Termination info
    pub terminated_at: Option<DateTime<Utc>>,
    pub termination_reason: Option<String>,
}
