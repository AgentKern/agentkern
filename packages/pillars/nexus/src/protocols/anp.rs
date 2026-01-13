use crate::error::NexusError;
use crate::protocols::adapter::ProtocolAdapter;
use crate::types::{NexusMessage, Protocol};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// W3C Agent Negotiation Protocol (ANP) Adapter.
///
/// Implements the draft W3C ANP 1.0 specification for agent negotiation.
/// Supports the basic negotiation cycle: Propose -> Counter -> Accept/Reject.
#[derive(Default)]
pub struct ANPAdapter;

#[derive(Debug, Serialize, Deserialize)]
struct AnpMessage {
    #[serde(rename = "type")]
    msg_type: String,
    id: String,
    thread_id: Option<String>,
    from: String,
    to: Option<String>,
    created_time: Option<u64>,
    body: serde_json::Value,
}

impl ANPAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ProtocolAdapter for ANPAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::W3cANP
    }

    fn detect(&self, raw: &[u8]) -> bool {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(raw) {
            if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
                return msg_type.starts_with("https://w3id.org/anp/");
            }
        }
        false
    }

    async fn parse(&self, raw: &[u8]) -> Result<NexusMessage, NexusError> {
        let anp: AnpMessage = serde_json::from_slice(raw).map_err(|e| NexusError::ParseError {
            message: format!("ANP parse error: {}", e),
        })?;

        // Map ANP types to Nexus methods
        // https://w3id.org/anp/1.0/propose -> anp.propose
        let method = anp
            .msg_type
            .replace("https://w3id.org/anp/1.0/", "anp.")
            .replace('/', ".");

        let mut msg = NexusMessage::new(method, anp.body)
            .from_agent(anp.from)
            .with_metadata("anp_id", json!(anp.id));

        if let Some(to) = anp.to {
            msg = msg.to_agent(to);
        }

        if let Some(thread) = anp.thread_id {
            msg.correlation_id = Some(thread.clone());
            msg = msg.with_metadata("thread_id", json!(thread));
        }

        msg.source_protocol = Protocol::W3cANP;

        Ok(msg)
    }

    async fn serialize(&self, msg: &NexusMessage) -> Result<Vec<u8>, NexusError> {
        // Map Nexus method back to ANP type
        // anp.propose -> https://w3id.org/anp/1.0/propose
        let msg_type = if msg.method.starts_with("anp.") {
            msg.method.replace("anp.", "https://w3id.org/anp/1.0/")
        } else {
            // Default fallback for generic messages
            format!("https://w3id.org/anp/1.0/{}", msg.method)
        };

        let anp = AnpMessage {
            msg_type,
            id: msg.id.clone(),
            thread_id: msg.correlation_id.clone(),
            from: msg
                .source_agent
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            to: msg.target_agent.clone(),
            created_time: Some(chrono::Utc::now().timestamp() as u64),
            body: msg.params.clone(),
        };

        serde_json::to_vec(&anp).map_err(|e| NexusError::SerializeError {
            message: format!("ANP serialization error: {}", e),
        })
    }
}
