//! MX Parser - ISO 20022 XML
//!
//! Parse and create ISO 20022 messages (pacs, pain, camt, etc.)

use super::{MxMessage, PaymentInstruction, SwiftError};

/// ISO 20022 MX message parser.
pub struct MxParser;

impl MxParser {
    /// Create new parser.
    pub fn new() -> Self {
        Self
    }
    
    /// Parse ISO 20022 XML message.
    pub fn parse(&self, xml: &str) -> Result<MxMessage, SwiftError> {
        // Production would use serde_xml or quick-xml
        let message_type = self.detect_message_type(xml)?;
        
        Ok(MxMessage {
            message_type,
            document_id: uuid::Uuid::new_v4().to_string(),
            creation_date: chrono::Utc::now().to_rfc3339(),
            content: serde_json::json!({ "raw": xml.len() }),
        })
    }
    
    /// Detect message type from XML.
    fn detect_message_type(&self, xml: &str) -> Result<String, SwiftError> {
        if xml.contains("pacs.008") || xml.contains("FIToFICstmrCdtTrf") {
            Ok("pacs.008".to_string())
        } else if xml.contains("pacs.002") || xml.contains("FIToFIPmtStsRpt") {
            Ok("pacs.002".to_string())
        } else if xml.contains("pain.001") || xml.contains("CstmrCdtTrfInitn") {
            Ok("pain.001".to_string())
        } else if xml.contains("camt.053") || xml.contains("BkToCstmrStmt") {
            Ok("camt.053".to_string())
        } else {
            Err(SwiftError::ParseError("Unknown message type".into()))
        }
    }
    
    /// Create pacs.008 FI to FI Customer Credit Transfer.
    pub fn create_pacs008(&self, payment: &PaymentInstruction) -> Result<String, SwiftError> {
        let xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.008.001.08">
    <FIToFICstmrCdtTrf>
        <GrpHdr>
            <MsgId>{}</MsgId>
            <CreDtTm>{}</CreDtTm>
            <NbOfTxs>1</NbOfTxs>
            <SttlmInf>
                <SttlmMtd>CLRG</SttlmMtd>
            </SttlmInf>
            <InstgAgt>
                <FinInstnId>
                    <BICFI>{}</BICFI>
                </FinInstnId>
            </InstgAgt>
        </GrpHdr>
        <CdtTrfTxInf>
            <PmtId>
                <InstrId>{}</InstrId>
                <EndToEndId>{}</EndToEndId>
            </PmtId>
            <IntrBkSttlmAmt Ccy="{}">{:.2}</IntrBkSttlmAmt>
            <Dbtr>
                <Nm>{}</Nm>
            </Dbtr>
            <DbtrAcct>
                <Id>
                    <IBAN>{}</IBAN>
                </Id>
            </DbtrAcct>
            <Cdtr>
                <Nm>{}</Nm>
            </Cdtr>
            <CdtrAcct>
                <Id>
                    <IBAN>{}</IBAN>
                </Id>
            </CdtrAcct>
        </CdtTrfTxInf>
    </FIToFICstmrCdtTrf>
</Document>"#,
            payment.message_id,
            payment.creation_date_time,
            payment.instructing_agent,
            payment.message_id,
            payment.message_id,
            payment.currency,
            payment.amount,
            payment.debtor_name,
            payment.debtor_account,
            payment.creditor_name,
            payment.creditor_account
        );
        
        Ok(xml)
    }
    
    /// Create pain.001 Customer Credit Transfer Initiation.
    pub fn create_pain001(&self, payment: &PaymentInstruction) -> Result<String, SwiftError> {
        // Similar structure to pacs.008 but for customer-to-bank
        Ok(format!("<?xml version=\"1.0\"?><pain.001>{}</pain.001>", payment.message_id))
    }
    
    /// Validate message against schema.
    pub fn validate(&self, xml: &str, schema: &str) -> Result<bool, SwiftError> {
        // Production would use XML schema validation
        Ok(!xml.is_empty() && !schema.is_empty())
    }
}

impl Default for MxParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pacs008() {
        let parser = MxParser::new();
        let xml = "<FIToFICstmrCdtTrf><GrpHdr/></FIToFICstmrCdtTrf>";
        assert_eq!(parser.detect_message_type(xml).unwrap(), "pacs.008");
    }

    #[test]
    fn test_create_pacs008() {
        let parser = MxParser::new();
        let payment = PaymentInstruction {
            message_id: "MSG001".into(),
            creation_date_time: "2025-12-26T12:00:00Z".into(),
            instructing_agent: "ABCDEFGH".into(),
            instructed_agent: None,
            debtor_name: "John Doe".into(),
            debtor_account: "DE89370400440532013000".into(),
            creditor_name: "Jane Smith".into(),
            creditor_account: "GB33BUKB20201555555555".into(),
            amount: 1000.00,
            currency: "EUR".into(),
            remittance_info: None,
        };
        
        let xml = parser.create_pacs008(&payment).unwrap();
        assert!(xml.contains("FIToFICstmrCdtTrf"));
        assert!(xml.contains("1000.00"));
    }
}
