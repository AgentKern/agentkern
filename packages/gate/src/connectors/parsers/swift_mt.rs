//! SWIFT MT Parser - Parse SWIFT FIN message formats
//!
//! Supports MT message types used in cross-border payments:
//! - MT103: Single Customer Credit Transfer
//! - MT202: General Financial Institution Transfer
//! - MT940: Customer Statement Message
//!
//! Per Strategic Roadmap: ISO 20022 migration completes Nov 2025

use serde::{Deserialize, Serialize};

/// SWIFT field extracted from message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwiftField {
    /// Field tag (e.g., "20", "32A", "50K")
    pub tag: String,
    /// Field value
    pub value: String,
    /// Sub-fields if applicable
    pub subfields: Vec<String>,
}

/// Parsed SWIFT MT message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwiftMtMessage {
    /// Message type (e.g., "103", "202")
    pub message_type: String,
    /// Sender BIC
    pub sender_bic: Option<String>,
    /// Receiver BIC
    pub receiver_bic: Option<String>,
    /// Transaction reference (field 20)
    pub reference: Option<String>,
    /// All parsed fields
    pub fields: Vec<SwiftField>,
    /// Raw message text
    pub raw: String,
}

impl SwiftMtMessage {
    /// Get field by tag.
    pub fn get_field(&self, tag: &str) -> Option<&SwiftField> {
        self.fields.iter().find(|f| f.tag == tag)
    }
    
    /// Get amount from field 32A (for MT103).
    pub fn get_amount(&self) -> Option<(String, f64)> {
        self.get_field("32A").and_then(|f| {
            // Format: YYMMDDCCCNNNNN,NN
            // Date (6) + Currency (3) + Amount
            if f.value.len() >= 12 {
                let currency = f.value[6..9].to_string();
                let amount_str = f.value[9..].replace(',', ".");
                amount_str.parse::<f64>().ok().map(|amt| (currency, amt))
            } else {
                None
            }
        })
    }
    
    /// Get beneficiary from field 59 (for MT103).
    pub fn get_beneficiary(&self) -> Option<String> {
        self.get_field("59").or_else(|| self.get_field("59A"))
            .map(|f| f.value.clone())
    }
}

/// SWIFT MT message parser.
pub struct SwiftMtParser;

impl SwiftMtParser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self
    }
    
    /// Parse a SWIFT MT message from raw text.
    pub fn parse(&self, raw: &str) -> Result<SwiftMtMessage, SwiftParseError> {
        let lines: Vec<&str> = raw.lines().collect();
        
        if lines.is_empty() {
            return Err(SwiftParseError::EmptyMessage);
        }
        
        // Extract message type from {2: block
        let message_type = self.extract_message_type(raw)?;
        
        // Extract BICs from headers
        let sender_bic = self.extract_sender_bic(raw);
        let receiver_bic = self.extract_receiver_bic(raw);
        
        // Parse fields from {4: block
        let fields = self.parse_fields(raw)?;
        
        // Extract reference (field 20)
        let reference = fields.iter()
            .find(|f| f.tag == "20")
            .map(|f| f.value.clone());
        
        Ok(SwiftMtMessage {
            message_type,
            sender_bic,
            receiver_bic,
            reference,
            fields,
            raw: raw.to_string(),
        })
    }
    
    /// Extract message type (e.g., "103" from MT103).
    fn extract_message_type(&self, raw: &str) -> Result<String, SwiftParseError> {
        // Look for {2:... block containing message type
        // Format: {2:I103... or {2:O103...
        if let Some(start) = raw.find("{2:") {
            let remaining = &raw[start + 3..];
            if remaining.len() >= 4 {
                // Skip I/O indicator, get 3-digit type
                let mt = &remaining[1..4];
                if mt.chars().all(|c| c.is_ascii_digit()) {
                    return Ok(mt.to_string());
                }
            }
        }
        
        // Fallback: look for MT\d{3} pattern
        for i in 0..raw.len().saturating_sub(4) {
            if &raw[i..i+2] == "MT" {
                let mt = &raw[i+2..i+5];
                if mt.len() == 3 && mt.chars().all(|c| c.is_ascii_digit()) {
                    return Ok(mt.to_string());
                }
            }
        }
        
        Err(SwiftParseError::MissingMessageType)
    }
    
    /// Extract sender BIC from {1: block.
    fn extract_sender_bic(&self, raw: &str) -> Option<String> {
        // {1:F01BANKXXXXXX...
        if let Some(start) = raw.find("{1:") {
            let remaining = &raw[start + 3..];
            if remaining.len() >= 12 {
                return Some(remaining[3..11].to_string());
            }
        }
        None
    }
    
    /// Extract receiver BIC from {2: block.
    fn extract_receiver_bic(&self, raw: &str) -> Option<String> {
        // {2:I103BANKXXXXXX...
        if let Some(start) = raw.find("{2:") {
            let remaining = &raw[start + 3..];
            if remaining.len() >= 15 {
                return Some(remaining[4..12].to_string());
            }
        }
        None
    }
    
    /// Parse field blocks from {4: section.
    fn parse_fields(&self, raw: &str) -> Result<Vec<SwiftField>, SwiftParseError> {
        let mut fields = Vec::new();
        
        // Find {4: block
        let block4_start = raw.find("{4:").or_else(|| raw.find(":20:"))
            .unwrap_or(0);
        let content = &raw[block4_start..];
        
        // Parse :XX: fields
        let mut current_tag: Option<String> = None;
        let mut current_value = String::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Check if line starts a new field
            if let Some(field_match) = self.extract_field_tag(trimmed) {
                // Save previous field if exists
                if let Some(tag) = current_tag.take() {
                    fields.push(SwiftField {
                        tag,
                        value: current_value.trim().to_string(),
                        subfields: self.parse_subfields(&current_value),
                    });
                }
                
                current_tag = Some(field_match.0);
                current_value = field_match.1.to_string();
            } else if current_tag.is_some() {
                // Continuation of current field
                if !trimmed.is_empty() && !trimmed.starts_with('-') && !trimmed.starts_with('}') {
                    current_value.push('\n');
                    current_value.push_str(trimmed);
                }
            }
        }
        
        // Save last field
        if let Some(tag) = current_tag {
            fields.push(SwiftField {
                tag,
                value: current_value.trim().to_string(),
                subfields: self.parse_subfields(&current_value),
            });
        }
        
        Ok(fields)
    }
    
    /// Extract field tag from line (e.g., ":20:" -> ("20", "value")).
    fn extract_field_tag<'a>(&self, line: &'a str) -> Option<(String, &'a str)> {
        if line.starts_with(':') && line.len() > 3 {
            if let Some(end) = line[1..].find(':') {
                let tag = &line[1..end+1];
                let value = &line[end+2..];
                return Some((tag.to_string(), value));
            }
        }
        None
    }
    
    /// Parse subfields from multiline value.
    fn parse_subfields(&self, value: &str) -> Vec<String> {
        value.lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }
}

impl Default for SwiftMtParser {
    fn default() -> Self {
        Self::new()
    }
}

/// SWIFT parse errors.
#[derive(Debug, Clone)]
pub enum SwiftParseError {
    EmptyMessage,
    MissingMessageType,
    InvalidFormat(String),
}

impl std::fmt::Display for SwiftParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMessage => write!(f, "Empty SWIFT message"),
            Self::MissingMessageType => write!(f, "Missing message type"),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for SwiftParseError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const MT103_SAMPLE: &str = r#"{1:F01BANKBEBBAXXX0000000000}{2:I103BANKDEFFXXXXN}{4:
:20:REF123456789
:23B:CRED
:32A:231215EUR1000,50
:50K:/12345678
JOHN DOE
123 MAIN STREET
:59:/98765432
JANE DOE
456 OAK AVENUE
:71A:SHA
-}"#;

    #[test]
    fn test_parse_mt103() {
        let parser = SwiftMtParser::new();
        let msg = parser.parse(MT103_SAMPLE).unwrap();
        
        assert_eq!(msg.message_type, "103");
        assert_eq!(msg.reference, Some("REF123456789".to_string()));
    }

    #[test]
    fn test_extract_bics() {
        let parser = SwiftMtParser::new();
        let msg = parser.parse(MT103_SAMPLE).unwrap();
        
        assert_eq!(msg.sender_bic, Some("BANKBEBB".to_string()));
        assert_eq!(msg.receiver_bic, Some("BANKDEFF".to_string()));
    }

    #[test]
    fn test_get_amount() {
        let parser = SwiftMtParser::new();
        let msg = parser.parse(MT103_SAMPLE).unwrap();
        
        let (currency, amount) = msg.get_amount().unwrap();
        assert_eq!(currency, "EUR");
        assert!((amount - 1000.50).abs() < 0.01);
    }

    #[test]
    fn test_get_field() {
        let parser = SwiftMtParser::new();
        let msg = parser.parse(MT103_SAMPLE).unwrap();
        
        let field_20 = msg.get_field("20").unwrap();
        assert_eq!(field_20.value, "REF123456789");
        
        let field_71a = msg.get_field("71A").unwrap();
        assert_eq!(field_71a.value, "SHA");
    }

    #[test]
    fn test_empty_message() {
        let parser = SwiftMtParser::new();
        let result = parser.parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiline_field() {
        let parser = SwiftMtParser::new();
        let msg = parser.parse(MT103_SAMPLE).unwrap();
        
        let field_50k = msg.get_field("50K").unwrap();
        assert!(field_50k.subfields.len() >= 2);
    }
}
