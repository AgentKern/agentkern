//! SWIFT GPI Connector - Production Bridge
//!
//! Production-grade connector for SWIFT GPI protocol.
//! Uses agentkern-parsers for MT message parsing.
//!
//! # Production Activation
//!
//! To enable production API calls, set these environment variables:
//! - `SWIFT_API_KEY`: Your SWIFT API Gateway key
//! - `SWIFT_BIC`: Your institution's BIC code
//! - `SWIFT_CERT_PATH`: Path to mTLS certificate (optional for sandbox)
//! - `SWIFT_SANDBOX`: Set to "false" for production (default: true)
//!
//! # Prerequisites
//!
//! 1. Register at https://developer.swift.com
//! 2. Subscribe to GPI Tracker API
//! 3. Generate API credentials
//! 4. Download mTLS certificates for production

use super::sdk::{
    A2ATaskPayload, ConnectorConfig, ConnectorError, ConnectorHealth, ConnectorProtocol,
    ConnectorResult, LegacyConnector, LegacyMessage,
};
use std::collections::HashMap;

/// SWIFT API endpoints
mod endpoints {
    pub const SANDBOX: &str = "https://sandbox.swift.com/swift-apitracker/v5";
    pub const PRODUCTION: &str = "https://api.swiftnet.sipn.swift.com/swift-apitracker/v5";
}

/// SWIFT GPI Connector for global payment tracking.
///
/// # Features
/// - MT message parsing (103, 202, etc.)
/// - GPI payment tracking via Tracker API
/// - UETR (Unique End-to-End Transaction Reference) management
/// - Automatic sandbox/production mode switching
///
/// # Production Mode
///
/// Production mode is enabled when:
/// 1. `SWIFT_API_KEY` is set
/// 2. `SWIFT_SANDBOX` is "false"
///
/// Otherwise, the connector operates in simulation mode.
pub struct SwiftGpiConnector {
    config: ConnectorConfig,
    /// BIC code of the institution
    bic: String,
    /// GPI participant flag
    gpi_enabled: bool,
    /// API key for SWIFT API Gateway
    api_key: Option<String>,
    /// Whether to use sandbox (default: true)
    use_sandbox: bool,
    /// HTTP client for API calls
    #[cfg(feature = "http")]
    http_client: reqwest::Client,
}

impl SwiftGpiConnector {
    /// Create a new SWIFT GPI connector.
    pub fn new(config: ConnectorConfig, bic: String) -> Self {
        Self {
            config,
            bic,
            gpi_enabled: true,
            api_key: None,
            use_sandbox: true,
            #[cfg(feature = "http")]
            http_client: reqwest::Client::new(),
        }
    }

    /// Create from environment variables.
    ///
    /// # Required Environment Variables
    /// - `SWIFT_BIC`: Institution BIC code
    ///
    /// # Optional Environment Variables
    /// - `SWIFT_API_KEY`: API key for production calls
    /// - `SWIFT_SANDBOX`: "true" (default) or "false"
    pub fn from_env() -> ConnectorResult<Self> {
        let bic = std::env::var("SWIFT_BIC")
            .map_err(|_| ConnectorError::ConnectionFailed("SWIFT_BIC not set".into()))?;

        let api_key = std::env::var("SWIFT_API_KEY").ok();
        let use_sandbox = std::env::var("SWIFT_SANDBOX")
            .map(|v| v.to_lowercase() != "false")
            .unwrap_or(true);

        let endpoint = if use_sandbox {
            endpoints::SANDBOX
        } else {
            endpoints::PRODUCTION
        };

        let config = ConnectorConfig {
            id: uuid::Uuid::new_v4().to_string(),
            name: if use_sandbox {
                "SWIFT GPI Sandbox".to_string()
            } else {
                "SWIFT GPI Production".to_string()
            },
            protocol: ConnectorProtocol::SwiftGpi,
            endpoint: endpoint.to_string(),
            timeout_ms: 60_000, // SWIFT can be slow
            max_retries: 3,
            settings: HashMap::new(),
        };

        let mut connector = Self::new(config, bic);
        connector.api_key = api_key;
        connector.use_sandbox = use_sandbox;
        Ok(connector)
    }

    /// Check if production API calls are enabled.
    ///
    /// Returns true if:
    /// 1. API key is set
    /// 2. Sandbox mode is disabled
    pub fn is_production_enabled(&self) -> bool {
        self.api_key.is_some() && !self.use_sandbox
    }

    /// Get the current API endpoint.
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    /// Generate UETR (Unique End-to-End Transaction Reference).
    pub fn generate_uetr() -> String {
        // UETR is UUID v4 format per SWIFT standards
        uuid::Uuid::new_v4().to_string()
    }

    /// Parse MT103 payment message.
    pub fn parse_mt103(&self, raw: &str) -> ConnectorResult<MT103Payment> {
        let parser = agentkern_parsers::SwiftMtParser::new();
        let parsed = parser
            .parse(raw)
            .map_err(|e| ConnectorError::ParseError(e.to_string()))?;

        // Extract fields from parsed message
        let amount_info = parsed.get_amount().unwrap_or(("USD".to_string(), 0.0));

        Ok(MT103Payment {
            sender_bic: parsed.sender_bic.clone().unwrap_or_default(),
            receiver_bic: parsed.receiver_bic.clone().unwrap_or_default(),
            amount: amount_info.1.to_string(),
            currency: amount_info.0,
            value_date: parsed
                .get_field("32A")
                .map(|f| f.value.clone())
                .unwrap_or_default(),
            reference: parsed
                .get_field("20")
                .map(|f| f.value.clone())
                .unwrap_or_default(),
            uetr: None, // UETR may be in field 121
        })
    }

    /// Track payment via GPI Tracker API.
    ///
    /// # Production Mode
    /// When enabled, calls the real SWIFT GPI Tracker API:
    /// `GET /payments/{uetr}/transactions`
    ///
    /// # Simulation Mode
    /// Returns simulated tracking status for testing.
    pub async fn track_payment(&self, uetr: &str) -> ConnectorResult<GpiTrackingStatus> {
        tracing::info!(
            uetr = uetr,
            bic = %self.bic,
            production = self.is_production_enabled(),
            "Tracking GPI payment"
        );

        if self.is_production_enabled() {
            self.track_payment_production(uetr).await
        } else {
            self.track_payment_simulated(uetr)
        }
    }

    /// Production API call to SWIFT GPI Tracker.
    #[cfg(feature = "http")]
    async fn track_payment_production(&self, uetr: &str) -> ConnectorResult<GpiTrackingStatus> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            ConnectorError::AuthenticationFailed("SWIFT_API_KEY not configured".into())
        })?;

        let url = format!("{}/payments/{}/transactions", self.config.endpoint, uetr);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_millis(self.config.timeout_ms))
            .send()
            .await
            .map_err(|e| ConnectorError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ConnectorError::ConnectionFailed(format!(
                "SWIFT API error {}: {}",
                status, body
            )));
        }

        let tracker_response: SwiftTrackerResponse = response
            .json()
            .await
            .map_err(|e| ConnectorError::ParseError(e.to_string()))?;

        Ok(self.convert_tracker_response(uetr, tracker_response))
    }

    /// Fallback when http feature is disabled.
    #[cfg(not(feature = "http"))]
    async fn track_payment_production(&self, uetr: &str) -> ConnectorResult<GpiTrackingStatus> {
        tracing::warn!("HTTP feature disabled, falling back to simulation");
        self.track_payment_simulated(uetr)
    }

    /// Simulated tracking for development/testing.
    fn track_payment_simulated(&self, uetr: &str) -> ConnectorResult<GpiTrackingStatus> {
        tracing::debug!(uetr = uetr, "Using simulated GPI tracking");

        Ok(GpiTrackingStatus {
            uetr: uetr.to_string(),
            transaction_status: TransactionStatus::Accepted,
            initiating_agent: self.bic.clone(),
            last_update: chrono::Utc::now().to_rfc3339(),
            settlements: vec![],
            simulated: true,
        })
    }

    /// Convert SWIFT API response to internal format.
    fn convert_tracker_response(
        &self,
        uetr: &str,
        response: SwiftTrackerResponse,
    ) -> GpiTrackingStatus {
        let settlements: Vec<Settlement> = response
            .payment_event
            .iter()
            .filter_map(|event| {
                if event.transaction_status == "ACSC" {
                    Some(Settlement {
                        settling_agent: event.from.clone(),
                        settled_amount: event.confirmed_amount.value.to_string(),
                        currency: event.confirmed_amount.currency.clone(),
                        timestamp: event.date_time.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        GpiTrackingStatus {
            uetr: uetr.to_string(),
            transaction_status: TransactionStatus::from_swift_code(
                response
                    .payment_event
                    .last()
                    .map(|e| e.transaction_status.as_str())
                    .unwrap_or("PDNG"),
            ),
            initiating_agent: response.initiation_time.clone().unwrap_or_default(),
            last_update: response
                .last_update_time
                .clone()
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            settlements,
            simulated: false,
        }
    }

    /// Initiate GPI payment.
    ///
    /// # Production Mode
    /// When enabled, submits payment via SWIFT Alliance Lite2 API.
    ///
    /// # Simulation Mode
    /// Returns simulated confirmation for testing.
    pub async fn initiate_payment(
        &self,
        payment: &MT103Payment,
    ) -> ConnectorResult<PaymentConfirmation> {
        let uetr = payment.uetr.clone().unwrap_or_else(Self::generate_uetr);

        tracing::info!(
            uetr = %uetr,
            amount = %payment.amount,
            currency = %payment.currency,
            production = self.is_production_enabled(),
            "Initiating SWIFT GPI payment"
        );

        if self.is_production_enabled() {
            self.initiate_payment_production(&uetr, payment).await
        } else {
            Ok(PaymentConfirmation {
                uetr,
                status: TransactionStatus::Pending,
                timestamp: chrono::Utc::now().to_rfc3339(),
                simulated: true,
            })
        }
    }

    /// Production payment initiation.
    #[cfg(feature = "http")]
    async fn initiate_payment_production(
        &self,
        uetr: &str,
        payment: &MT103Payment,
    ) -> ConnectorResult<PaymentConfirmation> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            ConnectorError::AuthenticationFailed("SWIFT_API_KEY not configured".into())
        })?;

        let url = format!("{}/payments", self.config.endpoint);

        let request_body = serde_json::json!({
            "uetr": uetr,
            "instructed_amount": {
                "currency": payment.currency,
                "amount": payment.amount
            },
            "debtor_agent": payment.sender_bic,
            "creditor_agent": payment.receiver_bic,
            "payment_reference": payment.reference
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(std::time::Duration::from_millis(self.config.timeout_ms))
            .send()
            .await
            .map_err(|e| ConnectorError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ConnectorError::ConnectionFailed(format!(
                "SWIFT payment initiation failed {}: {}",
                status, body
            )));
        }

        Ok(PaymentConfirmation {
            uetr: uetr.to_string(),
            status: TransactionStatus::Pending,
            timestamp: chrono::Utc::now().to_rfc3339(),
            simulated: false,
        })
    }

    #[cfg(not(feature = "http"))]
    async fn initiate_payment_production(
        &self,
        uetr: &str,
        _payment: &MT103Payment,
    ) -> ConnectorResult<PaymentConfirmation> {
        tracing::warn!("HTTP feature disabled, payment initiation simulated");
        Ok(PaymentConfirmation {
            uetr: uetr.to_string(),
            status: TransactionStatus::Pending,
            timestamp: chrono::Utc::now().to_rfc3339(),
            simulated: true,
        })
    }
}

// =============================================================================
// SWIFT API Response Types
// =============================================================================

/// SWIFT Tracker API response structure.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct SwiftTrackerResponse {
    #[serde(default)]
    payment_event: Vec<PaymentEvent>,
    initiation_time: Option<String>,
    last_update_time: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PaymentEvent {
    from: String,
    transaction_status: String,
    confirmed_amount: ConfirmedAmount,
    date_time: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ConfirmedAmount {
    currency: String,
    value: f64,
}

// =============================================================================
// Public Types
// =============================================================================

/// MT103 Single Customer Credit Transfer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MT103Payment {
    pub sender_bic: String,
    pub receiver_bic: String,
    pub amount: String,
    pub currency: String,
    pub value_date: String,
    pub reference: String,
    pub uetr: Option<String>,
}

/// GPI Tracking Status.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpiTrackingStatus {
    pub uetr: String,
    pub transaction_status: TransactionStatus,
    pub initiating_agent: String,
    pub last_update: String,
    pub settlements: Vec<Settlement>,
    /// True if this is simulated data (production mode disabled)
    #[serde(default)]
    pub simulated: bool,
}

/// Transaction status codes (ISO 20022).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Accepted,
    Settled,
    Rejected,
    Cancelled,
}

impl TransactionStatus {
    /// Convert from SWIFT status code.
    pub fn from_swift_code(code: &str) -> Self {
        match code {
            "ACSC" | "ACSP" => Self::Settled,
            "ACTC" | "ACCP" => Self::Accepted,
            "RJCT" => Self::Rejected,
            "CANC" => Self::Cancelled,
            _ => Self::Pending,
        }
    }
}

/// Settlement info.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settlement {
    pub settling_agent: String,
    pub settled_amount: String,
    pub currency: String,
    pub timestamp: String,
}

/// Payment confirmation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaymentConfirmation {
    pub uetr: String,
    pub status: TransactionStatus,
    pub timestamp: String,
    /// True if this is simulated data (production mode disabled)
    #[serde(default)]
    pub simulated: bool,
}

// =============================================================================
// LegacyConnector Implementation
// =============================================================================

#[async_trait::async_trait]
impl LegacyConnector for SwiftGpiConnector {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn protocol(&self) -> ConnectorProtocol {
        ConnectorProtocol::SwiftGpi
    }

    fn config(&self) -> &ConnectorConfig {
        &self.config
    }

    async fn health_check(&self) -> ConnectorResult<ConnectorHealth> {
        if self.is_production_enabled() {
            // TODO: Call SWIFT health endpoint when available
            Ok(ConnectorHealth::healthy())
        } else {
            Ok(ConnectorHealth::degraded("Running in simulation mode"))
        }
    }

    fn translate_to_legacy(&self, task: &A2ATaskPayload) -> ConnectorResult<LegacyMessage> {
        let mt_type = match task.method.as_str() {
            "transfer" | "payment" => "MT103",
            "cover" => "MT202COV",
            "status" => "MT199",
            _ => "MT999",
        };

        let data = serde_json::to_vec(&task.params)
            .map_err(|e| ConnectorError::ParseError(e.to_string()))?;

        Ok(LegacyMessage {
            data,
            message_type: mt_type.to_string(),
            metadata: HashMap::new(),
        })
    }

    fn translate_from_legacy(&self, msg: &LegacyMessage) -> ConnectorResult<A2ATaskPayload> {
        let params: serde_json::Value = serde_json::from_slice(&msg.data)
            .map_err(|e| ConnectorError::ParseError(e.to_string()))?;

        let method = match msg.message_type.as_str() {
            "MT103" => "payment_received",
            "MT202COV" => "cover_received",
            _ => "message_received",
        };

        Ok(A2ATaskPayload {
            id: uuid::Uuid::new_v4().to_string(),
            method: method.to_string(),
            params,
            source_agent: None,
            target_agent: None,
        })
    }

    async fn execute(&self, msg: &LegacyMessage) -> ConnectorResult<LegacyMessage> {
        match msg.message_type.as_str() {
            "MT103" => {
                let raw = String::from_utf8_lossy(&msg.data);
                let payment = self.parse_mt103(&raw)?;
                let confirmation = self.initiate_payment(&payment).await?;

                let data = serde_json::to_vec(&confirmation)
                    .map_err(|e| ConnectorError::Internal(e.to_string()))?;

                Ok(LegacyMessage {
                    data,
                    message_type: "MT103_ACK".to_string(),
                    metadata: HashMap::new(),
                })
            }
            _ => Err(ConnectorError::NotSupported(format!(
                "Message type {} not supported",
                msg.message_type
            ))),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uetr_generation() {
        let uetr = SwiftGpiConnector::generate_uetr();
        assert_eq!(uetr.len(), 36); // UUID format
    }

    #[test]
    fn test_swift_connector_config() {
        let config = ConnectorConfig {
            id: "swift-1".to_string(),
            name: "SWIFT GPI".to_string(),
            protocol: ConnectorProtocol::SwiftGpi,
            endpoint: "swift.example.com".to_string(),
            timeout_ms: 60_000,
            max_retries: 3,
            settings: HashMap::new(),
        };

        let connector = SwiftGpiConnector::new(config, "BANKUS33XXX".to_string());
        assert_eq!(connector.protocol(), ConnectorProtocol::SwiftGpi);
        assert_eq!(connector.bic, "BANKUS33XXX");
        assert!(!connector.is_production_enabled());
    }

    #[test]
    fn test_production_mode_detection() {
        let config = ConnectorConfig {
            id: "swift-prod".to_string(),
            name: "SWIFT GPI Production".to_string(),
            protocol: ConnectorProtocol::SwiftGpi,
            endpoint: endpoints::PRODUCTION.to_string(),
            timeout_ms: 60_000,
            max_retries: 3,
            settings: HashMap::new(),
        };

        let mut connector = SwiftGpiConnector::new(config, "BANKUS33XXX".to_string());
        assert!(!connector.is_production_enabled());

        connector.api_key = Some("test-key".to_string());
        assert!(!connector.is_production_enabled()); // Still sandbox

        connector.use_sandbox = false;
        assert!(connector.is_production_enabled()); // Now production
    }

    #[test]
    fn test_transaction_status_conversion() {
        assert_eq!(
            TransactionStatus::from_swift_code("ACSC"),
            TransactionStatus::Settled
        );
        assert_eq!(
            TransactionStatus::from_swift_code("RJCT"),
            TransactionStatus::Rejected
        );
        assert_eq!(
            TransactionStatus::from_swift_code("PDNG"),
            TransactionStatus::Pending
        );
        assert_eq!(
            TransactionStatus::from_swift_code("UNKNOWN"),
            TransactionStatus::Pending
        );
    }

    #[tokio::test]
    async fn test_simulated_tracking() {
        let config = ConnectorConfig {
            id: "swift-test".to_string(),
            name: "SWIFT Test".to_string(),
            protocol: ConnectorProtocol::SwiftGpi,
            endpoint: endpoints::SANDBOX.to_string(),
            timeout_ms: 5_000,
            max_retries: 1,
            settings: HashMap::new(),
        };

        let connector = SwiftGpiConnector::new(config, "TESTBIC".to_string());
        let result = connector.track_payment("test-uetr").await;

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.simulated);
        assert_eq!(status.uetr, "test-uetr");
    }
}
