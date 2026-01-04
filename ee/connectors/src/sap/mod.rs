//! SAP Enterprise Connector
//!
//! Full SAP integration: RFC, BAPI, OData, Event Mesh
//! Per LICENSING.md: Enterprise tier ($100K+ SAP deals)

mod rfc;
mod bapi;
mod odata;
mod event_mesh;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::license::{check_feature_license, LicenseError};

pub use rfc::RfcConnection;
pub use bapi::BapiCaller;
pub use odata::ODataClient;
pub use event_mesh::EventMeshClient;

/// SAP connector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SapConfig {
    /// SAP system ID
    pub system_id: String,
    /// SAP client number
    pub client: String,
    /// Application server host
    pub ashost: String,
    /// System number
    pub sysnr: String,
    /// SAP user
    pub user: String,
    /// Language
    pub language: String,
    /// Connection pool size
    pub pool_size: usize,
}

impl Default for SapConfig {
    fn default() -> Self {
        Self {
            system_id: String::new(),
            client: "100".to_string(),
            ashost: "localhost".to_string(),
            sysnr: "00".to_string(),
            user: String::new(),
            language: "EN".to_string(),
            pool_size: 5,
        }
    }
}

/// SAP connector with all integration modes.
pub struct SapConnector {
    config: SapConfig,
    rfc: Option<RfcConnection>,
    odata: Option<ODataClient>,
    event_mesh: Option<EventMeshClient>,
}

impl SapConnector {
    /// Create a new SAP connector (requires license).
    pub fn new(config: SapConfig) -> Result<Self, LicenseError> {
        check_feature_license("sap")?;
        
        Ok(Self {
            config,
            rfc: None,
            odata: None,
            event_mesh: None,
        })
    }
    
    /// Connect via RFC.
    pub fn connect_rfc(&mut self, password: &str) -> Result<(), SapError> {
        self.rfc = Some(RfcConnection::new(&self.config, password)?);
        Ok(())
    }
    
    /// Connect via OData (S/4HANA).
    pub fn connect_odata(&mut self, base_url: &str, auth: ODataAuth) -> Result<(), SapError> {
        self.odata = Some(ODataClient::new(base_url, auth)?);
        Ok(())
    }
    
    /// Call a BAPI function.
    pub fn call_bapi(&self, bapi_name: &str, params: HashMap<String, serde_json::Value>) -> Result<BapiResult, SapError> {
        let rfc = self.rfc.as_ref().ok_or(SapError::NotConnected)?;
        let caller = BapiCaller::new(rfc);
        caller.call(bapi_name, params)
    }
    
    /// Read OData entity.
    pub fn read_entity(&self, entity_set: &str, key: &str) -> Result<serde_json::Value, SapError> {
        let odata = self.odata.as_ref().ok_or(SapError::ODataNotConfigured)?;
        odata.get(entity_set, key)
    }
    
    /// Create OData entity.
    pub fn create_entity(&self, entity_set: &str, data: serde_json::Value) -> Result<serde_json::Value, SapError> {
        let odata = self.odata.as_ref().ok_or(SapError::ODataNotConfigured)?;
        odata.post(entity_set, data)
    }
    
    /// Subscribe to Event Mesh.
    pub fn subscribe_events(&mut self, queue: &str) -> Result<(), SapError> {
        let mut mesh = EventMeshClient::new(&self.config)?;
        mesh.subscribe(queue)?;
        self.event_mesh = Some(mesh);
        Ok(())
    }
    
    /// Health check.
    pub fn health_check(&self) -> SapHealth {
        SapHealth {
            rfc_connected: self.rfc.is_some(),
            odata_connected: self.odata.is_some(),
            event_mesh_connected: self.event_mesh.is_some(),
        }
    }
}

/// OData authentication.
#[derive(Debug, Clone)]
pub enum ODataAuth {
    Basic { username: String, password: String },
    OAuth2 { client_id: String, client_secret: String, token_url: String },
    Certificate { cert_path: String, key_path: String },
}

/// BAPI call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BapiResult {
    pub success: bool,
    pub return_table: Vec<BapiReturn>,
    pub export_params: HashMap<String, serde_json::Value>,
    pub tables: HashMap<String, Vec<serde_json::Value>>,
}

/// BAPI return message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BapiReturn {
    pub message_type: String,
    pub message_id: String,
    pub message_number: String,
    pub message: String,
}

/// SAP health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SapHealth {
    pub rfc_connected: bool,
    pub odata_connected: bool,
    pub event_mesh_connected: bool,
}

/// SAP error types.
#[derive(Debug, thiserror::Error)]
pub enum SapError {
    #[error("Not connected to SAP")]
    NotConnected,
    
    #[error("OData not configured")]
    ODataNotConfigured,
    
    #[error("RFC connection failed: {0}")]
    RfcError(String),
    
    #[error("BAPI error: {0}")]
    BapiError(String),
    
    #[error("OData error: {0}")]
    ODataError(String),
    
    #[error("Event Mesh error: {0}")]
    EventMeshError(String),
    
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
    fn test_sap_config_default() {
        let config = SapConfig::default();
        assert_eq!(config.client, "100");
        assert_eq!(config.language, "EN");
    }

    #[test]
    fn test_sap_health() {
        let health = SapHealth {
            rfc_connected: true,
            odata_connected: false,
            event_mesh_connected: false,
        };
        assert!(health.rfc_connected);
    }

    // Note: Full connector tests require license key
}
