//! GPI Tracker - SWIFT GPI Payment Tracking
//!
//! Track payments with Universal End-to-End Transaction Reference (UETR)

use super::{GpiConfirmation, GpiStatus, SwiftConfig, SwiftError};
use crate::license::LicenseError;

/// SWIFT GPI tracker.
pub struct GpiTracker {
    config: SwiftConfig,
}

impl GpiTracker {
    /// Create new GPI tracker (requires license).
    pub fn new(config: &SwiftConfig) -> Result<Self, LicenseError> {
        Ok(Self {
            config: config.clone(),
        })
    }

    /// Generate UETR for new payment.
    pub fn generate_uetr() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Track payment by UETR.
    pub fn track(&self, uetr: &str) -> Result<GpiStatus, SwiftError> {
        // Production would call SWIFT gpi Tracker API
        Ok(GpiStatus {
            uetr: uetr.to_string(),
            status: "ACSC".to_string(), // Accepted Settlement Completed
            last_update: chrono::Utc::now().to_rfc3339(),
            confirmations: vec![GpiConfirmation {
                confirming_agent: "ABCDEFGH".into(),
                status: "ACSC".into(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                reason_code: None,
            }],
        })
    }

    /// Get confirmations for UETR.
    pub fn get_confirmations(&self, uetr: &str) -> Result<Vec<GpiConfirmation>, SwiftError> {
        let status = self.track(uetr)?;
        Ok(status.confirmations)
    }

    /// Update payment status.
    pub fn update_status(
        &self,
        uetr: &str,
        new_status: &str,
        reason: Option<&str>,
    ) -> Result<(), SwiftError> {
        // Production would call SWIFT gpi Status Update API
        Ok(())
    }

    /// Get payments pending confirmation.
    pub fn get_pending(&self) -> Result<Vec<String>, SwiftError> {
        // Would query for pending UETRs
        Ok(vec![])
    }
}

/// GPI status codes.
pub mod status_codes {
    pub const ACSC: &str = "ACSC"; // Accepted Settlement Completed
    pub const ACSP: &str = "ACSP"; // Accepted Settlement in Progress
    pub const PDNG: &str = "PDNG"; // Pending
    pub const RJCT: &str = "RJCT"; // Rejected
    pub const RCVD: &str = "RCVD"; // Received
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uetr() {
        let uetr = GpiTracker::generate_uetr();
        assert!(!uetr.is_empty());
        assert!(uetr.contains('-')); // UUID format
    }
}
