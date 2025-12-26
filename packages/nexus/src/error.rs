//! Nexus Error Types

use thiserror::Error;
use crate::types::Protocol;

/// Nexus errors.
#[derive(Debug, Error)]
pub enum NexusError {
    #[error("Unknown protocol: could not detect protocol from message")]
    UnknownProtocol,

    #[error("Protocol not supported: {protocol:?}")]
    ProtocolNotSupported { protocol: Protocol },

    #[error("Adapter not registered for protocol: {protocol:?}")]
    AdapterNotRegistered { protocol: Protocol },

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("Serialization error: {message}")]
    SerializeError { message: String },

    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: String },

    #[error("Agent already registered: {agent_id}")]
    AgentAlreadyExists { agent_id: String },

    #[error("No matching agent for task: {task_type}")]
    NoMatchingAgent { task_type: String },

    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },

    #[error("Task failed: {reason}")]
    TaskFailed { reason: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Rate limited")]
    RateLimited,

    #[error("Timeout")]
    Timeout,
}

impl From<serde_json::Error> for NexusError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError { message: e.to_string() }
    }
}

impl From<reqwest::Error> for NexusError {
    fn from(e: reqwest::Error) -> Self {
        Self::NetworkError { message: e.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NexusError::AgentNotFound { agent_id: "agent-1".into() };
        assert_eq!(err.to_string(), "Agent not found: agent-1");
    }
}
