//! COBOL Copybook Parser - Parse COBOL data structures
//!
//! Copybooks define fixed-width record layouts used in mainframe systems.
//! This parser extracts field definitions and parses binary/text data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// COBOL field data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopybookFieldType {
    /// Alphanumeric (PIC X)
    Alphanumeric,
    /// Numeric display (PIC 9)
    NumericDisplay,
    /// Numeric signed (PIC S9)
    NumericSigned,
    /// Packed decimal (COMP-3)
    PackedDecimal,
    /// Binary (COMP)
    Binary,
    /// Group (contains subfields)
    Group,
}

/// COBOL field definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopybookField {
    /// Field name
    pub name: String,
    /// Level number (01-49, 66, 77, 88)
    pub level: u8,
    /// Data type
    pub field_type: CopybookFieldType,
    /// Length in bytes
    pub length: usize,
    /// Decimal places (for numeric)
    pub decimals: usize,
    /// Start position (0-indexed)
    pub offset: usize,
    /// PIC clause
    pub pic: String,
    /// Parent field name (for nested structures)
    pub parent: Option<String>,
}

impl CopybookField {
    /// Check if field is a group.
    pub fn is_group(&self) -> bool {
        self.field_type == CopybookFieldType::Group
    }

    /// Get end position.
    pub fn end_offset(&self) -> usize {
        self.offset + self.length
    }
}

/// Parsed record from copybook layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopybookRecord {
    /// Field values
    pub fields: HashMap<String, String>,
    /// Raw data
    pub raw: Vec<u8>,
}

impl CopybookRecord {
    /// Get field value.
    pub fn get(&self, field: &str) -> Option<&String> {
        self.fields.get(field)
    }

    /// Get numeric value.
    pub fn get_numeric(&self, field: &str) -> Option<f64> {
        self.fields.get(field).and_then(|v| v.trim().parse().ok())
    }
}

/// COBOL Copybook parser.
pub struct CopybookParser {
    /// Parsed field definitions
    fields: Vec<CopybookField>,
    /// Total record length
    record_length: usize,
}

impl CopybookParser {
    /// Create parser from copybook definition.
    pub fn new(copybook: &str) -> Result<Self, CopybookParseError> {
        let fields = Self::parse_copybook(copybook)?;
        let record_length = fields
            .iter()
            .filter(|f| !f.is_group())
            .map(|f| f.end_offset())
            .max()
            .unwrap_or(0);

        Ok(Self {
            fields,
            record_length,
        })
    }

    /// Parse copybook definition to extract field layouts.
    fn parse_copybook(copybook: &str) -> Result<Vec<CopybookField>, CopybookParseError> {
        let mut fields = Vec::new();
        let mut offset = 0usize;
        let mut parent_stack: Vec<(u8, String)> = Vec::new();

        for line in copybook.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('*') {
                continue;
            }

            if let Some(field) = Self::parse_field_line(line, &mut offset, &parent_stack)? {
                // Update parent stack
                while !parent_stack.is_empty() && parent_stack.last().unwrap().0 >= field.level {
                    parent_stack.pop();
                }

                if field.is_group() {
                    parent_stack.push((field.level, field.name.clone()));
                }

                fields.push(field);
            }
        }

        Ok(fields)
    }

    /// Parse a single copybook line.
    fn parse_field_line(
        line: &str,
        offset: &mut usize,
        parent_stack: &[(u8, String)],
    ) -> Result<Option<CopybookField>, CopybookParseError> {
        // Remove trailing period
        let line = line.trim_end_matches('.');
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            return Ok(None);
        }

        // Parse level number
        let level: u8 = parts[0]
            .parse()
            .map_err(|_| CopybookParseError::InvalidLevel(parts[0].to_string()))?;

        // Parse field name
        let name = parts[1].to_string();

        // Find PIC clause
        let (field_type, length, decimals, pic) = if let Some(pic_idx) = parts
            .iter()
            .position(|&p| p.eq_ignore_ascii_case("PIC") || p.eq_ignore_ascii_case("PICTURE"))
        {
            if pic_idx + 1 < parts.len() {
                Self::parse_pic_clause(parts[pic_idx + 1])?
            } else {
                return Err(CopybookParseError::InvalidPic("Missing PIC value".into()));
            }
        } else {
            // Group field (no PIC)
            (CopybookFieldType::Group, 0, 0, String::new())
        };

        let parent = parent_stack.last().map(|(_, n)| n.clone());

        let field = CopybookField {
            name,
            level,
            field_type,
            length,
            decimals,
            offset: *offset,
            pic,
            parent,
        };

        // Update offset for non-group fields
        if !field.is_group() {
            *offset += length;
        }

        Ok(Some(field))
    }

    /// Parse PIC clause to determine type and length.
    fn parse_pic_clause(
        pic: &str,
    ) -> Result<(CopybookFieldType, usize, usize, String), CopybookParseError> {
        let pic_upper = pic.to_uppercase();
        let mut field_type = CopybookFieldType::Alphanumeric;
        let mut length = 0usize;
        let mut decimals = 0usize;

        // Count characters and determine type
        let mut in_decimal = false;
        let mut i = 0;
        let chars: Vec<char> = pic_upper.chars().collect();

        while i < chars.len() {
            let c = chars[i];

            match c {
                'X' => {
                    field_type = CopybookFieldType::Alphanumeric;
                    length += Self::get_repeat_count(&chars, &mut i);
                }
                '9' => {
                    if field_type == CopybookFieldType::Alphanumeric {
                        field_type = CopybookFieldType::NumericDisplay;
                    }
                    let count = Self::get_repeat_count(&chars, &mut i);
                    if in_decimal {
                        decimals += count;
                    }
                    length += count;
                }
                'S' => {
                    field_type = CopybookFieldType::NumericSigned;
                }
                'V' => {
                    in_decimal = true;
                }
                '(' => {
                    // Skip, handled by get_repeat_count
                }
                _ => {}
            }
            i += 1;
        }

        Ok((field_type, length, decimals, pic.to_string()))
    }

    /// Get repeat count from (n) notation.
    fn get_repeat_count(chars: &[char], i: &mut usize) -> usize {
        if *i + 1 < chars.len() && chars[*i + 1] == '(' {
            // Find matching )
            if let Some(end) = chars[*i + 2..].iter().position(|&c| c == ')') {
                let num_str: String = chars[*i + 2..*i + 2 + end].iter().collect();
                if let Ok(n) = num_str.parse::<usize>() {
                    *i += 2 + end;
                    return n;
                }
            }
        }
        1
    }

    /// Parse data using the copybook layout.
    pub fn parse_record(&self, data: &[u8]) -> Result<CopybookRecord, CopybookParseError> {
        let mut fields = HashMap::new();

        for field in &self.fields {
            if field.is_group() {
                continue;
            }

            if field.offset + field.length > data.len() {
                continue; // Skip if data is shorter
            }

            let raw_value = &data[field.offset..field.offset + field.length];
            let value = self.extract_field_value(field, raw_value)?;
            fields.insert(field.name.clone(), value);
        }

        Ok(CopybookRecord {
            fields,
            raw: data.to_vec(),
        })
    }

    /// Extract field value based on type.
    fn extract_field_value(
        &self,
        field: &CopybookField,
        data: &[u8],
    ) -> Result<String, CopybookParseError> {
        match field.field_type {
            CopybookFieldType::Alphanumeric | CopybookFieldType::Group => {
                // Convert EBCDIC or ASCII to string
                Ok(String::from_utf8_lossy(data).trim().to_string())
            }
            CopybookFieldType::NumericDisplay | CopybookFieldType::NumericSigned => {
                let s = String::from_utf8_lossy(data).trim().to_string();
                if field.decimals > 0 && s.len() > field.decimals {
                    // Insert decimal point
                    let pos = s.len() - field.decimals;
                    Ok(format!("{}.{}", &s[..pos], &s[pos..]))
                } else {
                    Ok(s)
                }
            }
            CopybookFieldType::PackedDecimal => {
                // COMP-3: each byte = 2 digits, last nibble = sign
                let mut result = String::new();
                for (i, &byte) in data.iter().enumerate() {
                    let high = (byte >> 4) & 0x0F;
                    let low = byte & 0x0F;

                    result.push_str(&high.to_string());
                    if i < data.len() - 1 {
                        result.push_str(&low.to_string());
                    } else {
                        // Last nibble is sign
                        if low == 0x0D {
                            result.insert(0, '-');
                        }
                    }
                }
                Ok(result)
            }
            CopybookFieldType::Binary => {
                // COMP: big-endian binary
                let mut value = 0i64;
                for &byte in data {
                    value = (value << 8) | (byte as i64);
                }
                Ok(value.to_string())
            }
        }
    }

    /// Get field definitions.
    pub fn fields(&self) -> &[CopybookField] {
        &self.fields
    }

    /// Get total record length.
    pub fn record_length(&self) -> usize {
        self.record_length
    }
}

/// Copybook parse errors.
#[derive(Debug, Clone)]
pub enum CopybookParseError {
    InvalidLevel(String),
    InvalidPic(String),
    InvalidData(String),
}

impl std::fmt::Display for CopybookParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLevel(msg) => write!(f, "Invalid level: {}", msg),
            Self::InvalidPic(msg) => write!(f, "Invalid PIC: {}", msg),
            Self::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for CopybookParseError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const COPYBOOK_SAMPLE: &str = r#"
       01  CUSTOMER-RECORD.
           05  CUST-ID            PIC 9(10).
           05  CUST-NAME          PIC X(30).
           05  CUST-BALANCE       PIC S9(9)V99.
           05  CUST-STATUS        PIC X(1).
    "#;

    #[test]
    fn test_parse_copybook() {
        let parser = CopybookParser::new(COPYBOOK_SAMPLE).unwrap();

        assert_eq!(parser.fields().len(), 5); // Including group
        assert_eq!(parser.record_length(), 52); // 10 + 30 + 11 + 1
    }

    #[test]
    fn test_field_types() {
        let parser = CopybookParser::new(COPYBOOK_SAMPLE).unwrap();

        let cust_id = parser
            .fields()
            .iter()
            .find(|f| f.name == "CUST-ID")
            .unwrap();
        assert_eq!(cust_id.field_type, CopybookFieldType::NumericDisplay);
        assert_eq!(cust_id.length, 10);

        let cust_name = parser
            .fields()
            .iter()
            .find(|f| f.name == "CUST-NAME")
            .unwrap();
        assert_eq!(cust_name.field_type, CopybookFieldType::Alphanumeric);
        assert_eq!(cust_name.length, 30);
    }

    #[test]
    fn test_parse_record() {
        let parser = CopybookParser::new(COPYBOOK_SAMPLE).unwrap();

        // Create sample data
        let mut data = vec![b' '; 52];
        // CUST-ID: "0000000001"
        data[0..10].copy_from_slice(b"0000000001");
        // CUST-NAME: "JOHN DOE"
        data[10..18].copy_from_slice(b"JOHN DOE");
        // CUST-BALANCE: "00000100050" (1000.50)
        data[40..51].copy_from_slice(b"00000100050");
        // CUST-STATUS: "A"
        data[51] = b'A';

        let record = parser.parse_record(&data).unwrap();

        assert_eq!(record.get("CUST-ID"), Some(&"0000000001".to_string()));
        assert_eq!(record.get("CUST-STATUS"), Some(&"A".to_string()));
    }

    #[test]
    fn test_group_field() {
        let parser = CopybookParser::new(COPYBOOK_SAMPLE).unwrap();

        let group = parser
            .fields()
            .iter()
            .find(|f| f.name == "CUSTOMER-RECORD")
            .unwrap();
        assert!(group.is_group());
        assert_eq!(group.level, 1);
    }

    #[test]
    fn test_decimal_handling() {
        let parser = CopybookParser::new(COPYBOOK_SAMPLE).unwrap();

        let balance = parser
            .fields()
            .iter()
            .find(|f| f.name == "CUST-BALANCE")
            .unwrap();
        assert_eq!(balance.decimals, 2);
    }
}
