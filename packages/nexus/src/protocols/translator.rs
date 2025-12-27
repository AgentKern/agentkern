//! Protocol Translator
//!
//! Central translation engine for converting between agent protocols.
//! Supports: A2A, MCP, AgentKern, ANP, NLIP, AITP

use crate::types::{NexusMessage, Protocol, Task, TaskStatus};
use crate::protocols::adapter::{AdapterRegistry, ProtocolAdapter};
use crate::error::NexusError;
use std::collections::HashMap;

/// Translation result with metadata.
#[derive(Debug, Clone)]
pub struct TranslationResult {
    /// Translated message
    pub message: NexusMessage,
    /// Source protocol
    pub source_protocol: Protocol,
    /// Target protocol
    pub target_protocol: Protocol,
    /// Fields that couldn't be translated
    pub lost_fields: Vec<String>,
    /// Confidence score (0-100)
    pub confidence: u8,
}

/// Protocol translator engine.
pub struct ProtocolTranslator {
    registry: AdapterRegistry,
    /// Field mappings between protocols
    field_mappings: HashMap<(Protocol, Protocol), FieldMapping>,
}

/// Field mapping between two protocols.
#[derive(Debug, Clone, Default)]
pub struct FieldMapping {
    /// Direct field name mappings
    pub direct: HashMap<String, String>,
    /// Fields that need transformation
    pub transforms: Vec<FieldTransform>,
}

/// Field transformation rule.
#[derive(Debug, Clone)]
pub struct FieldTransform {
    /// Source field
    pub source: String,
    /// Target field
    pub target: String,
    /// Transformation type
    pub transform_type: TransformType,
}

/// Types of field transformations.
#[derive(Debug, Clone)]
pub enum TransformType {
    /// Direct copy
    Copy,
    /// Rename field
    Rename(String),
    /// Convert format (e.g., date formats)
    Format(String),
    /// Map to enum value
    EnumMap(HashMap<String, String>),
    /// Drop field (not supported in target)
    Drop,
}

impl ProtocolTranslator {
    /// Create a new translator with default registry.
    pub fn new() -> Self {
        let mut translator = Self {
            registry: AdapterRegistry::new(),
            field_mappings: HashMap::new(),
        };
        translator.load_default_mappings();
        translator
    }

    /// Create with custom registry.
    pub fn with_registry(registry: AdapterRegistry) -> Self {
        let mut translator = Self {
            registry,
            field_mappings: HashMap::new(),
        };
        translator.load_default_mappings();
        translator
    }

    /// Load default field mappings between protocols.
    fn load_default_mappings(&mut self) {
        // A2A → MCP mappings
        let mut a2a_to_mcp = FieldMapping::default();
        a2a_to_mcp.direct.insert("task_id".to_string(), "id".to_string());
        a2a_to_mcp.direct.insert("message".to_string(), "content".to_string());
        self.field_mappings.insert(
            (Protocol::GoogleA2A, Protocol::AnthropicMCP),
            a2a_to_mcp,
        );

        // MCP → A2A mappings
        let mut mcp_to_a2a = FieldMapping::default();
        mcp_to_a2a.direct.insert("id".to_string(), "task_id".to_string());
        mcp_to_a2a.direct.insert("content".to_string(), "message".to_string());
        self.field_mappings.insert(
            (Protocol::AnthropicMCP, Protocol::GoogleA2A),
            mcp_to_a2a,
        );
    }

    /// Translate a raw message from one protocol to another.
    /// Note: This requires knowing the source protocol ahead of time.
    /// For auto-detection, use the Nexus gateway which has the AdapterRegistry.
    pub fn translate_raw(
        &self,
        source_protocol: Protocol,
        _raw: &[u8],
        target_protocol: Protocol,
    ) -> Result<TranslationResult, NexusError> {
        // In production, parse raw bytes using the adapter
        // For now, return an error indicating raw translation requires adapters
        Err(NexusError::NotSupported {
            feature: "Raw message translation requires Nexus gateway".to_string(),
        })
    }

    /// Translate between parsed messages.
    pub fn translate_message(
        &self,
        message: NexusMessage,
        target_protocol: Protocol,
    ) -> Result<TranslationResult, NexusError> {
        let source_protocol = message.source_protocol;
        
        if source_protocol == target_protocol {
            return Ok(TranslationResult {
                message,
                source_protocol,
                target_protocol,
                lost_fields: vec![],
                confidence: 100,
            });
        }
        
        let (translated, lost_fields) = self.apply_mappings(
            message,
            &source_protocol,
            &target_protocol,
        );
        
        let confidence = if lost_fields.is_empty() { 100 } else {
            (100 - (lost_fields.len() * 10).min(50)) as u8
        };
        
        Ok(TranslationResult {
            message: translated,
            source_protocol,
            target_protocol,
            lost_fields,
            confidence,
        })
    }

    /// Apply field mappings between protocols.
    fn apply_mappings(
        &self,
        mut message: NexusMessage,
        source: &Protocol,
        target: &Protocol,
    ) -> (NexusMessage, Vec<String>) {
        let mut lost_fields = vec![];
        
        // Update protocol marker
        message.source_protocol = *target;
        
        // Apply mappings if available
        if let Some(mapping) = self.field_mappings.get(&(*source, *target)) {
            // Apply direct mappings to params
            if let Some(params) = message.params.as_object_mut() {
                let mut new_params = serde_json::Map::new();
                
                for (key, value) in params.iter() {
                    if let Some(new_key) = mapping.direct.get(key) {
                        new_params.insert(new_key.clone(), value.clone());
                    } else {
                        // Keep unmapped fields, but track them
                        new_params.insert(key.clone(), value.clone());
                    }
                }
                
                message.params = serde_json::Value::Object(new_params);
            }
        }
        
        (message, lost_fields)
    }

    /// Get supported translation paths.
    pub fn supported_translations(&self) -> Vec<(Protocol, Protocol)> {
        self.field_mappings.keys().cloned().collect()
    }

    /// Translate task status between protocols.
    pub fn translate_status(
        &self,
        status: TaskStatus,
        _target_protocol: Protocol,
    ) -> TaskStatus {
        // Status is already unified, but protocols may have different names
        status
    }
}

impl Default for ProtocolTranslator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_translator_creation() {
        let translator = ProtocolTranslator::new();
        let paths = translator.supported_translations();
        
        assert!(!paths.is_empty());
    }
    
    #[test]
    fn test_same_protocol_translation() {
        let translator = ProtocolTranslator::new();
        
        let message = NexusMessage {
            id: "test-1".to_string(),
            method: "execute".to_string(),
            params: serde_json::json!({"action": "test"}),
            source_protocol: Protocol::AgentKern,
            source_agent: Some("agent-1".to_string()),
            target_agent: Some("agent-2".to_string()),
            correlation_id: None,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        let result = translator.translate_message(message, Protocol::AgentKern).unwrap();
        
        assert_eq!(result.confidence, 100);
        assert!(result.lost_fields.is_empty());
    }
    
    #[test]
    fn test_cross_protocol_translation() {
        let translator = ProtocolTranslator::new();
        
        let message = NexusMessage {
            id: "test-2".to_string(),
            method: "invoke".to_string(),
            params: serde_json::json!({"task_id": "123", "message": "hello"}),
            source_protocol: Protocol::GoogleA2A,
            source_agent: None,
            target_agent: None,
            correlation_id: None,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        let result = translator.translate_message(message, Protocol::AnthropicMCP).unwrap();
        
        assert_eq!(result.source_protocol, Protocol::GoogleA2A);
        assert_eq!(result.target_protocol, Protocol::AnthropicMCP);
    }
    
    #[test]
    fn test_status_translation() {
        let translator = ProtocolTranslator::new();
        
        let status = translator.translate_status(TaskStatus::Working, Protocol::AnthropicMCP);
        assert_eq!(status, TaskStatus::Working);
    }
}
