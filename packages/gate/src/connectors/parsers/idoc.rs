//! SAP IDOC Parser - Parse SAP Intermediate Document format
//!
//! IDOCs are SAP's standard format for exchanging business data.
//! Common types: ORDERS05 (Orders), INVOIC02 (Invoices), MATMAS05 (Materials)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IDOC segment (data record).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDocSegment {
    /// Segment name (e.g., "E1EDK01", "E1EDP01")
    pub name: String,
    /// Segment number in hierarchy
    pub number: u32,
    /// Parent segment number
    pub parent: Option<u32>,
    /// Segment level (1 = header, 2+ = detail)
    pub level: u32,
    /// Field values
    pub fields: HashMap<String, String>,
}

impl IDocSegment {
    /// Get field value.
    pub fn get(&self, field: &str) -> Option<&String> {
        self.fields.get(field)
    }
    
    /// Check if segment is header level.
    pub fn is_header(&self) -> bool {
        self.level == 1
    }
}

/// Parsed SAP IDOC message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IDocMessage {
    /// IDOC type (e.g., "ORDERS05", "INVOIC02")
    pub idoc_type: String,
    /// IDOC number
    pub idoc_number: String,
    /// Direction (1 = outbound, 2 = inbound)
    pub direction: u8,
    /// Sender partner
    pub sender: Option<String>,
    /// Receiver partner
    pub receiver: Option<String>,
    /// Message type
    pub message_type: Option<String>,
    /// All segments
    pub segments: Vec<IDocSegment>,
    /// Control record fields
    pub control: HashMap<String, String>,
}

impl IDocMessage {
    /// Get segments by name.
    pub fn get_segments(&self, name: &str) -> Vec<&IDocSegment> {
        self.segments.iter().filter(|s| s.name == name).collect()
    }
    
    /// Get header segment.
    pub fn get_header(&self) -> Option<&IDocSegment> {
        self.segments.iter().find(|s| s.is_header())
    }
    
    /// Get all detail segments.
    pub fn get_details(&self) -> Vec<&IDocSegment> {
        self.segments.iter().filter(|s| !s.is_header()).collect()
    }
}

/// SAP IDOC parser.
pub struct IDocParser;

impl IDocParser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self
    }
    
    /// Parse IDOC from raw text (flat file format).
    pub fn parse(&self, raw: &str) -> Result<IDocMessage, IDocParseError> {
        let lines: Vec<&str> = raw.lines().collect();
        
        if lines.is_empty() {
            return Err(IDocParseError::EmptyDocument);
        }
        
        // Parse control record (first line, usually 524 chars)
        let control = self.parse_control_record(lines.get(0).unwrap_or(&""))?;
        
        // Extract basic info from control
        let idoc_type = control.get("IDOCTYP")
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let idoc_number = control.get("DOCNUM")
            .cloned()
            .unwrap_or_else(|| "0".to_string());
        let direction = control.get("DIRECT")
            .and_then(|d| d.parse().ok())
            .unwrap_or(1);
        let sender = control.get("SNDPRN").cloned();
        let receiver = control.get("RCVPRN").cloned();
        let message_type = control.get("MESTYP").cloned();
        
        // Parse data segments (remaining lines)
        let segments = self.parse_segments(&lines[1..])?;
        
        Ok(IDocMessage {
            idoc_type,
            idoc_number,
            direction,
            sender,
            receiver,
            message_type,
            segments,
            control,
        })
    }
    
    /// Parse control record.
    fn parse_control_record(&self, line: &str) -> Result<HashMap<String, String>, IDocParseError> {
        let mut fields = HashMap::new();
        
        // Control record layout (simplified for common fields)
        // Real IDOCs have fixed-width fields
        if line.len() >= 10 {
            // Parse as space/tab separated for simplicity
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            // Common control fields
            for (i, part) in parts.iter().enumerate() {
                if part.contains('=') {
                    let kv: Vec<&str> = part.splitn(2, '=').collect();
                    if kv.len() == 2 {
                        fields.insert(kv[0].to_string(), kv[1].to_string());
                    }
                } else {
                    // Positional fields
                    match i {
                        0 => { fields.insert("TABNAM".to_string(), part.to_string()); }
                        1 => { fields.insert("DOCNUM".to_string(), part.to_string()); }
                        2 => { fields.insert("IDOCTYP".to_string(), part.to_string()); }
                        3 => { fields.insert("MESTYP".to_string(), part.to_string()); }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(fields)
    }
    
    /// Parse data segments.
    fn parse_segments(&self, lines: &[&str]) -> Result<Vec<IDocSegment>, IDocParseError> {
        let mut segments = Vec::new();
        let mut segment_num = 0u32;
        
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            
            segment_num += 1;
            let segment = self.parse_segment_line(line, segment_num)?;
            segments.push(segment);
        }
        
        Ok(segments)
    }
    
    /// Parse a single segment line.
    fn parse_segment_line(&self, line: &str, number: u32) -> Result<IDocSegment, IDocParseError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(IDocParseError::InvalidSegment("Empty segment".into()));
        }
        
        let name = parts[0].to_string();
        
        // Determine level from segment name pattern
        // E1xxx = level 1, E2xxx = level 2, etc.
        let level = if name.starts_with("E1") {
            1
        } else if name.starts_with("E2") {
            2
        } else {
            name.chars()
                .nth(1)
                .and_then(|c| c.to_digit(10))
                .unwrap_or(1)
        };
        
        // Parse fields from remaining parts
        let mut fields = HashMap::new();
        for part in &parts[1..] {
            if part.contains('=') {
                let kv: Vec<&str> = part.splitn(2, '=').collect();
                if kv.len() == 2 {
                    fields.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }
        
        Ok(IDocSegment {
            name,
            number,
            parent: if number > 1 { Some(1) } else { None },
            level,
            fields,
        })
    }
    
    /// Parse from XML format (used in newer SAP systems).
    pub fn parse_xml(&self, xml: &str) -> Result<IDocMessage, IDocParseError> {
        // Simplified XML parsing - in production use proper XML parser
        let mut fields: HashMap<String, String> = HashMap::new();
        let segments = Vec::new();
        
        // Extract IDOC type from root element
        let idoc_type = if let Some(start) = xml.find("<IDOC BEGIN=\"1\"") {
            let remaining = &xml[start..];
            if let Some(type_start) = remaining.find("IDOCTYP=\"") {
                let type_content = &remaining[type_start + 9..];
                if let Some(end) = type_content.find('"') {
                    type_content[..end].to_string()
                } else {
                    "UNKNOWN".to_string()
                }
            } else {
                "UNKNOWN".to_string()
            }
        } else {
            "UNKNOWN".to_string()
        };
        
        fields.insert("IDOCTYP".to_string(), idoc_type.clone());
        
        Ok(IDocMessage {
            idoc_type,
            idoc_number: "0".to_string(),
            direction: 1,
            sender: None,
            receiver: None,
            message_type: None,
            segments,
            control: fields,
        })
    }
}

impl Default for IDocParser {
    fn default() -> Self {
        Self::new()
    }
}

/// IDOC parse errors.
#[derive(Debug, Clone)]
pub enum IDocParseError {
    EmptyDocument,
    InvalidControlRecord(String),
    InvalidSegment(String),
}

impl std::fmt::Display for IDocParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDocument => write!(f, "Empty IDOC document"),
            Self::InvalidControlRecord(msg) => write!(f, "Invalid control record: {}", msg),
            Self::InvalidSegment(msg) => write!(f, "Invalid segment: {}", msg),
        }
    }
}

impl std::error::Error for IDocParseError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const IDOC_SAMPLE: &str = r#"EDI_DC40 0000000001 ORDERS05 ORDERS
E1EDK01 BELNR=4500000001 BSARK=NB CURCY=EUR
E1EDP01 POSNR=000010 MATNR=MAT001 MENGE=100
E1EDP01 POSNR=000020 MATNR=MAT002 MENGE=50"#;

    #[test]
    fn test_parse_idoc() {
        let parser = IDocParser::new();
        let idoc = parser.parse(IDOC_SAMPLE).unwrap();
        
        assert_eq!(idoc.idoc_type, "ORDERS05");
        assert_eq!(idoc.segments.len(), 3);
    }

    #[test]
    fn test_get_segments() {
        let parser = IDocParser::new();
        let idoc = parser.parse(IDOC_SAMPLE).unwrap();
        
        let header = idoc.get_segments("E1EDK01");
        assert_eq!(header.len(), 1);
        
        let items = idoc.get_segments("E1EDP01");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_segment_fields() {
        let parser = IDocParser::new();
        let idoc = parser.parse(IDOC_SAMPLE).unwrap();
        
        let header = idoc.get_header().unwrap();
        assert_eq!(header.get("BELNR"), Some(&"4500000001".to_string()));
        assert_eq!(header.get("CURCY"), Some(&"EUR".to_string()));
    }

    #[test]
    fn test_segment_levels() {
        let parser = IDocParser::new();
        let idoc = parser.parse(IDOC_SAMPLE).unwrap();
        
        let header = idoc.get_header().unwrap();
        assert!(header.is_header());
        assert_eq!(header.level, 1);
    }

    #[test]
    fn test_empty_document() {
        let parser = IDocParser::new();
        let result = parser.parse("");
        assert!(result.is_err());
    }
}
