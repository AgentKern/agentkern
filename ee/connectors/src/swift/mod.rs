//! SWIFT Enterprise Connector
//!
//! Full SWIFT integration: MX (ISO 20022), GPI Tracking, Sanctions
//! Per LICENSING.md: Banking tier ($80K+ deals)

mod mx_parser;
mod gpi;
mod sanctions;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::license::{check_feature_license, LicenseError};

pub use mx_parser::MxParser;
pub use gpi::GpiTracker;
pub use sanctions::SanctionsScreener;

/// SWIFT connector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwiftConfig {
    /// BIC of this institution
    pub own_bic: String,
    /// Alliance Access/Lite endpoint
    pub endpoint: String,
    /// Certificate path
    pub cert_path: Option<String>,
    /// Enable GPI tracking
    pub gpi_enabled: bool,
    /// Sanctions list sources
    pub sanctions_sources: Vec<String>,
}

impl Default for SwiftConfig {
    fn default() -> Self {
        Self {
            own_bic: String::new(),
            endpoint: String::new(),
            cert_path: None,
            gpi_enabled: true,
            sanctions_sources: vec!["OFAC".to_string(), "EU".to_string(), "UN".to_string()],
        }
    }
}

/// SWIFT connector for ISO 20022 messaging.
pub struct SwiftConnector {
    config: SwiftConfig,
    mx_parser: MxParser,
    gpi_tracker: Option<GpiTracker>,
    sanctions: SanctionsScreener,
}

impl SwiftConnector {
    /// Create new SWIFT connector (requires license).
    pub fn new(config: SwiftConfig) -> Result<Self, LicenseError> {
        check_feature_license("swift")?;
        
        let gpi_tracker = if config.gpi_enabled {
            Some(GpiTracker::new(&config)?)
        } else {
            None
        };
        
        Ok(Self {
            sanctions: SanctionsScreener::new(&config.sanctions_sources),
            mx_parser: MxParser::new(),
            gpi_tracker,
            config,
        })
    }
    
    /// Parse MX (ISO 20022) message.
    pub fn parse_mx(&self, xml: &str) -> Result<MxMessage, SwiftError> {
        self.mx_parser.parse(xml)
    }
    
    /// Create payment initiation (pacs.008).
    pub fn create_payment(&self, payment: PaymentInstruction) -> Result<String, SwiftError> {
        // Check sanctions first
        self.screen_payment(&payment)?;
        
        // Create ISO 20022 pacs.008 message
        self.mx_parser.create_pacs008(&payment)
    }
    
    /// Screen payment against sanctions lists.
    pub fn screen_payment(&self, payment: &PaymentInstruction) -> Result<SanctionsResult, SwiftError> {
        self.sanctions.screen(&payment.debtor_name, &payment.creditor_name)
    }
    
    /// Track GPI payment.
    pub fn track_payment(&self, uetr: &str) -> Result<GpiStatus, SwiftError> {
        let tracker = self.gpi_tracker.as_ref()
            .ok_or(SwiftError::GpiNotEnabled)?;
        tracker.track(uetr)
    }
    
    /// Get GPI confirmations.
    pub fn get_confirmations(&self, uetr: &str) -> Result<Vec<GpiConfirmation>, SwiftError> {
        let tracker = self.gpi_tracker.as_ref()
            .ok_or(SwiftError::GpiNotEnabled)?;
        tracker.get_confirmations(uetr)
    }
    
    /// Health check.
    pub fn health_check(&self) -> SwiftHealth {
        SwiftHealth {
            own_bic: self.config.own_bic.clone(),
            gpi_enabled: self.gpi_tracker.is_some(),
            sanctions_loaded: self.sanctions.list_count() > 0,
        }
    }
}

/// Payment instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentInstruction {
    pub message_id: String,
    pub creation_date_time: String,
    pub instructing_agent: String,
    pub instructed_agent: Option<String>,
    pub debtor_name: String,
    pub debtor_account: String,
    pub creditor_name: String,
    pub creditor_account: String,
    pub amount: f64,
    pub currency: String,
    pub remittance_info: Option<String>,
}

/// Parsed MX message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MxMessage {
    pub message_type: String,
    pub document_id: String,
    pub creation_date: String,
    pub content: serde_json::Value,
}

/// GPI tracking status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpiStatus {
    pub uetr: String,
    pub status: String,
    pub last_update: String,
    pub confirmations: Vec<GpiConfirmation>,
}

/// GPI confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpiConfirmation {
    pub confirming_agent: String,
    pub status: String,
    pub timestamp: String,
    pub reason_code: Option<String>,
}

/// Sanctions screening result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsResult {
    pub clear: bool,
    pub matches: Vec<SanctionsMatch>,
}

/// Sanctions match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsMatch {
    pub list: String,
    pub name: String,
    pub score: f64,
}

/// SWIFT health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwiftHealth {
    pub own_bic: String,
    pub gpi_enabled: bool,
    pub sanctions_loaded: bool,
}

/// SWIFT error types.
#[derive(Debug, thiserror::Error)]
pub enum SwiftError {
    #[error("GPI tracking not enabled")]
    GpiNotEnabled,
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Sanctions hit: {0}")]
    SanctionsHit(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("License error: {0}")]
    LicenseError(#[from] LicenseError),
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_config_default() {
        let config = SwiftConfig::default();
        assert!(config.gpi_enabled);
        assert!(!config.sanctions_sources.is_empty());
    }

    #[test]
    fn test_payment_instruction() {
        let payment = PaymentInstruction {
            message_id: "MSG001".into(),
            creation_date_time: "2025-12-26T12:00:00Z".into(),
            instructing_agent: "ABCDEFGH".into(),
            instructed_agent: Some("IJKLMNOP".into()),
            debtor_name: "John Doe".into(),
            debtor_account: "DE89370400440532013000".into(),
            creditor_name: "Jane Smith".into(),
            creditor_account: "GB33BUKB20201555555555".into(),
            amount: 1000.00,
            currency: "EUR".into(),
            remittance_info: Some("Invoice 123".into()),
        };
        
        assert_eq!(payment.currency, "EUR");
    }
}
