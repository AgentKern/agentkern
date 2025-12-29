//! Agent Card - A2A Compatible Agent Discovery
//!
//! Per A2A Spec: Agents publish capabilities at `/.well-known/agent.json`
//!
//! This module provides a unified Agent Card format compatible with:
//! - Google A2A Agent Cards
//! - OpenAPI-style capability descriptions
//! - AgentKern extensions

use crate::types::{Capability, Modality, Skill};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent Card - Universal agent discovery format.
///
/// Compatible with Google A2A Agent Cards spec.
/// Published at `/.well-known/agent.json` for discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCard {
    /// Unique agent identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of what this agent does
    pub description: String,

    /// Base URL for agent API
    pub url: String,

    /// Agent version
    pub version: String,

    /// Provider/owner
    #[serde(default)]
    pub provider: Option<Provider>,

    /// Capabilities this agent supports
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// Skills this agent can perform
    #[serde(default)]
    pub skills: Vec<Skill>,

    /// Default input modalities
    #[serde(default)]
    pub default_input_modes: Vec<Modality>,

    /// Default output modalities
    #[serde(default)]
    pub default_output_modes: Vec<Modality>,

    /// Authentication info
    #[serde(default)]
    pub authentication: AuthInfo,

    /// Supported protocols
    #[serde(default)]
    pub protocols: Vec<ProtocolSupport>,

    /// AgentKern-specific extensions
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl Default for AgentCard {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            url: String::new(),
            version: "1.0.0".into(),
            provider: None,
            capabilities: vec![],
            skills: vec![],
            default_input_modes: vec![Modality::Text],
            default_output_modes: vec![Modality::Text],
            authentication: AuthInfo::default(),
            protocols: vec![],
            extensions: HashMap::new(),
        }
    }
}

impl AgentCard {
    /// Create a new agent card.
    pub fn new(id: impl Into<String>, name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a skill.
    pub fn with_skill(mut self, skill: Skill) -> Self {
        self.skills.push(skill);
        self
    }

    /// Add a capability.
    pub fn with_capability(mut self, cap: Capability) -> Self {
        self.capabilities.push(cap);
        self
    }

    /// Add protocol support.
    pub fn supports_protocol(mut self, protocol: ProtocolSupport) -> Self {
        self.protocols.push(protocol);
        self
    }

    /// Check if agent has a specific skill.
    pub fn has_skill(&self, skill_id: &str) -> bool {
        self.skills.iter().any(|s| s.id == skill_id)
    }

    /// Check if agent supports a skill tag.
    pub fn has_skill_tag(&self, tag: &str) -> bool {
        self.skills
            .iter()
            .any(|s| s.tags.contains(&tag.to_string()))
    }

    /// Calculate skill match score (0-100).
    pub fn skill_match_score(&self, required_skills: &[String]) -> u8 {
        if required_skills.is_empty() {
            return 100;
        }

        let matched = required_skills
            .iter()
            .filter(|s| self.has_skill(s) || self.has_skill_tag(s))
            .count();

        ((matched as f64 / required_skills.len() as f64) * 100.0) as u8
    }

    /// Serialize to JSON for well-known endpoint.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Agent provider info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Organization name
    pub organization: String,
    /// Contact URL
    pub url: Option<String>,
}

/// Authentication information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthInfo {
    /// Supported auth schemes
    #[serde(default)]
    pub schemes: Vec<AuthScheme>,
    /// OAuth configuration
    pub oauth: Option<OAuthConfig>,
}

/// Authentication scheme.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthScheme {
    None,
    ApiKey,
    Bearer,
    OAuth2,
    Mtls,
    Did,
}

/// OAuth configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// Authorization URL
    pub authorization_url: String,
    /// Token URL
    pub token_url: String,
    /// Scopes
    pub scopes: Vec<String>,
}

/// Protocol support declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSupport {
    /// Protocol name
    pub name: String,
    /// Version
    pub version: String,
    /// Endpoint (if different from main URL)
    pub endpoint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_card_creation() {
        let card = AgentCard::new("agent-1", "Test Agent", "https://api.example.com")
            .with_description("A helpful agent")
            .with_skill(Skill {
                id: "summarize".into(),
                name: "Summarization".into(),
                description: "Summarize text".into(),
                tags: vec!["nlp".into(), "text".into()],
                input_schema: None,
                output_schema: None,
            });

        assert_eq!(card.id, "agent-1");
        assert_eq!(card.skills.len(), 1);
        assert!(card.has_skill("summarize"));
        assert!(card.has_skill_tag("nlp"));
    }

    #[test]
    fn test_skill_match_score() {
        let card = AgentCard::new("agent", "Agent", "http://localhost")
            .with_skill(Skill {
                id: "nlp".into(),
                name: "NLP".into(),
                description: "".into(),
                tags: vec![],
                input_schema: None,
                output_schema: None,
            })
            .with_skill(Skill {
                id: "vision".into(),
                name: "Vision".into(),
                description: "".into(),
                tags: vec![],
                input_schema: None,
                output_schema: None,
            });

        let required = vec!["nlp".into(), "vision".into(), "audio".into()];
        let score = card.skill_match_score(&required);

        // 2 out of 3 = 66%
        assert!(score >= 66 && score <= 67);
    }

    #[test]
    fn test_agent_card_serialization() {
        let card = AgentCard::new("test", "Test", "https://example.com").supports_protocol(
            ProtocolSupport {
                name: "a2a".into(),
                version: "0.3".into(),
                endpoint: None,
            },
        );

        let json = card.to_json().unwrap();
        assert!(json.contains("\"name\": \"Test\""));

        let parsed = AgentCard::from_json(&json).unwrap();
        assert_eq!(parsed.id, "test");
    }

    #[test]
    fn test_well_known_format() {
        let card = AgentCard::new("my-agent", "My Agent", "https://agent.example.com")
            .with_description("An example agent")
            .supports_protocol(ProtocolSupport {
                name: "a2a".into(),
                version: "0.3".into(),
                endpoint: None,
            })
            .supports_protocol(ProtocolSupport {
                name: "mcp".into(),
                version: "2025-06-18".into(),
                endpoint: Some("/mcp".into()),
            });

        let json = card.to_json().unwrap();

        // Should be valid for /.well-known/agent.json
        assert!(json.contains("my-agent"));
        assert!(json.contains("a2a"));
        assert!(json.contains("mcp"));
    }

    #[test]
    fn test_empty_skills_score() {
        let card = AgentCard::new("agent", "Agent", "http://localhost");

        // No skills required = 100% match
        assert_eq!(card.skill_match_score(&[]), 100);

        // Skills required but agent has none = 0%
        let required = vec!["nlp".into()];
        assert_eq!(card.skill_match_score(&required), 0);
    }

    #[test]
    fn test_capability_builder() {
        let card =
            AgentCard::new("agent", "Agent", "http://localhost").with_capability(Capability {
                name: "streaming".into(),
                input_modes: vec![Modality::Text],
                output_modes: vec![Modality::Text, Modality::Audio],
                rate_limit: Some(60),
            });

        assert_eq!(card.capabilities.len(), 1);
        assert_eq!(card.capabilities[0].name, "streaming");
    }

    #[test]
    fn test_auth_schemes() {
        let mut card = AgentCard::new("secure-agent", "Secure", "https://secure.example.com");
        card.authentication = AuthInfo {
            schemes: vec![AuthScheme::OAuth2, AuthScheme::ApiKey],
            oauth: Some(OAuthConfig {
                authorization_url: "https://auth.example.com/authorize".into(),
                token_url: "https://auth.example.com/token".into(),
                scopes: vec!["read".into(), "write".into()],
            }),
        };

        let json = card.to_json().unwrap();
        assert!(json.contains("oauth2"));
        assert!(json.contains("authorization_url"));
    }

    #[test]
    fn test_provider_info() {
        let mut card = AgentCard::new("corp-agent", "Corp Agent", "https://corp.example.com");
        card.provider = Some(Provider {
            organization: "AgentKern Inc.".into(),
            url: Some("https://agentkern.io".into()),
        });

        let json = card.to_json().unwrap();
        assert!(json.contains("AgentKern Inc."));
    }

    #[test]
    fn test_extensions() {
        let mut card = AgentCard::new("extended", "Extended Agent", "http://localhost");
        card.extensions
            .insert("custom_field".into(), serde_json::json!({"key": "value"}));
        card.extensions
            .insert("version_info".into(), serde_json::json!("v2.0"));

        let json = card.to_json().unwrap();
        let parsed = AgentCard::from_json(&json).unwrap();

        assert!(parsed.extensions.contains_key("custom_field"));
        assert!(parsed.extensions.contains_key("version_info"));
    }
}
