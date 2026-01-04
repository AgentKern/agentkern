//! Mainframe Enterprise Connector
//!
//! IBM mainframe integration: CICS, IMS, MQ
//! Per LICENSING.md: Enterprise tier (F500 mainframe deals)

mod cics;
mod ims;
mod mq;

use serde::{Deserialize, Serialize};
use super::license::{check_feature_license, LicenseError};

pub use cics::CicsClient;
pub use ims::ImsClient;
pub use mq::MqClient;

/// Mainframe connector configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainframeConfig {
    /// Mainframe hostname
    pub host: String,
    /// CICS port
    pub cics_port: u16,
    /// IMS Connect port
    pub ims_port: Option<u16>,
    /// MQ Queue Manager
    pub queue_manager: Option<String>,
    /// MQ Channel
    pub mq_channel: Option<String>,
    /// Code page (EBCDIC)
    pub code_page: String,
}

impl Default for MainframeConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            cics_port: 1490,
            ims_port: Some(9999),
            queue_manager: None,
            mq_channel: None,
            code_page: "IBM037".to_string(),
        }
    }
}

/// Mainframe connector.
pub struct MainframeConnector {
    config: MainframeConfig,
    cics: Option<CicsClient>,
    ims: Option<ImsClient>,
    mq: Option<MqClient>,
}

impl MainframeConnector {
    /// Create new Mainframe connector (requires license).
    pub fn new(config: MainframeConfig) -> Result<Self, LicenseError> {
        check_feature_license("mainframe")?;
        
        Ok(Self {
            config,
            cics: None,
            ims: None,
            mq: None,
        })
    }
    
    /// Connect to CICS.
    pub fn connect_cics(&mut self, user: &str, password: &str) -> Result<(), MainframeError> {
        self.cics = Some(CicsClient::connect(&self.config, user, password)?);
        Ok(())
    }
    
    /// Connect to IMS.
    pub fn connect_ims(&mut self, datastores: Vec<String>) -> Result<(), MainframeError> {
        if self.config.ims_port.is_none() {
            return Err(MainframeError::ImsNotConfigured);
        }
        self.ims = Some(ImsClient::connect(&self.config, datastores)?);
        Ok(())
    }
    
    /// Connect to MQ.
    pub fn connect_mq(&mut self) -> Result<(), MainframeError> {
        let qm = self.config.queue_manager.as_ref()
            .ok_or(MainframeError::MqNotConfigured)?;
        self.mq = Some(MqClient::connect(&self.config, qm)?);
        Ok(())
    }
    
    /// Execute CICS transaction.
    pub fn exec_transaction(&self, tranid: &str, commarea: &[u8]) -> Result<Vec<u8>, MainframeError> {
        let cics = self.cics.as_ref().ok_or(MainframeError::NotConnected)?;
        cics.exec_transaction(tranid, commarea)
    }
    
    /// Execute CICS program.
    pub fn link_program(&self, program: &str, commarea: &[u8]) -> Result<Vec<u8>, MainframeError> {
        let cics = self.cics.as_ref().ok_or(MainframeError::NotConnected)?;
        cics.link_program(program, commarea)
    }
    
    /// Run IMS transaction.
    pub fn ims_transaction(&self, trancode: &str, segments: Vec<&[u8]>) -> Result<Vec<Vec<u8>>, MainframeError> {
        let ims = self.ims.as_ref().ok_or(MainframeError::ImsNotConfigured)?;
        ims.exec_transaction(trancode, segments)
    }
    
    /// Put message to MQ queue.
    pub fn mq_put(&self, queue: &str, message: &[u8]) -> Result<String, MainframeError> {
        let mq = self.mq.as_ref().ok_or(MainframeError::MqNotConfigured)?;
        mq.put(queue, message)
    }
    
    /// Get message from MQ queue.
    pub fn mq_get(&self, queue: &str) -> Result<Option<Vec<u8>>, MainframeError> {
        let mq = self.mq.as_ref().ok_or(MainframeError::MqNotConfigured)?;
        mq.get(queue)
    }
    
    /// Health check.
    pub fn health_check(&self) -> MainframeHealth {
        MainframeHealth {
            cics_connected: self.cics.is_some(),
            ims_connected: self.ims.is_some(),
            mq_connected: self.mq.is_some(),
        }
    }
}

/// Mainframe health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainframeHealth {
    pub cics_connected: bool,
    pub ims_connected: bool,
    pub mq_connected: bool,
}

/// Mainframe error types.
#[derive(Debug, thiserror::Error)]
pub enum MainframeError {
    #[error("Not connected")]
    NotConnected,
    
    #[error("IMS not configured")]
    ImsNotConfigured,
    
    #[error("MQ not configured")]
    MqNotConfigured,
    
    #[error("CICS error: {0}")]
    CicsError(String),
    
    #[error("IMS error: {0}")]
    ImsError(String),
    
    #[error("MQ error: {0}")]
    MqError(String),
    
    #[error("Encoding error: {0}")]
    EncodingError(String),
    
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
    fn test_mainframe_config_default() {
        let config = MainframeConfig::default();
        assert_eq!(config.cics_port, 1490);
        assert_eq!(config.code_page, "IBM037");
    }

    #[test]
    fn test_mainframe_health() {
        let health = MainframeHealth {
            cics_connected: true,
            ims_connected: false,
            mq_connected: true,
        };
        assert!(health.cics_connected);
        assert!(!health.ims_connected);
    }
}
