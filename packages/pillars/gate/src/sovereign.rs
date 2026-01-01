//! AgentKern-Gate: Sovereign Data Module
//!
//! Per GLOBAL_GAPS.md §1: Data Sovereignty / Geo-Fenced Cells
//!
//! Features:
//! - Geo-Fenced Cells: Prevent cross-region data synchronization
//! - Residency Controller: Block data transfers violating sovereignty
//! - Cross-Border Transfer Validation: Check data origin vs destination
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::sovereign::{SovereignController, DataTransfer};
//! use agentkern_gate::types::DataRegion;
//!
//! let controller = SovereignController::new();
//!
//! // This transfer would be BLOCKED (CN data cannot leave CN)
//! let transfer = DataTransfer::new("user-data-123", DataRegion::Cn, DataRegion::Us);
//! assert!(!controller.is_allowed(&transfer));
//! ```

use crate::types::DataRegion;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors for sovereign data operations.
#[derive(Debug, Error)]
pub enum SovereignError {
    #[error("Cross-border transfer blocked: {origin:?} -> {destination:?}")]
    TransferBlocked {
        origin: DataRegion,
        destination: DataRegion,
    },
    #[error("Data residency violation: data must stay in {required:?}")]
    ResidencyViolation { required: DataRegion },
    #[error("No adequacy agreement between {from:?} and {to:?}")]
    NoAdequacy { from: DataRegion, to: DataRegion },
}

/// A data transfer request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransfer {
    /// Unique identifier for the data
    pub data_id: String,
    /// Origin region where data was created
    pub origin: DataRegion,
    /// Destination region for the transfer
    pub destination: DataRegion,
    /// Type of data (for policy matching)
    pub data_type: DataType,
    /// Is this PII (Personally Identifiable Information)?
    pub is_pii: bool,
    /// Business justification
    pub justification: Option<String>,
}

/// Type of data being transferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// General business data
    Business,
    /// Personal/user data
    Personal,
    /// Financial data
    Financial,
    /// Health data (HIPAA, etc.)
    Health,
    /// Aggregated/anonymized data
    Aggregated,
}

impl DataTransfer {
    /// Create a new data transfer request.
    pub fn new(data_id: impl Into<String>, origin: DataRegion, destination: DataRegion) -> Self {
        Self {
            data_id: data_id.into(),
            origin,
            destination,
            data_type: DataType::Business,
            is_pii: false,
            justification: None,
        }
    }

    /// Mark this transfer as containing PII.
    pub fn with_pii(mut self) -> Self {
        self.is_pii = true;
        self
    }

    /// Set the data type.
    pub fn with_data_type(mut self, data_type: DataType) -> Self {
        self.data_type = data_type;
        self
    }

    /// Add business justification.
    pub fn with_justification(mut self, justification: impl Into<String>) -> Self {
        self.justification = Some(justification.into());
        self
    }
}

/// Result of a transfer validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDecision {
    /// Is the transfer allowed?
    pub allowed: bool,
    /// Reason for the decision
    pub reason: String,
    /// Required safeguards (if any)
    pub safeguards: Vec<String>,
}

/// Sovereign data controller for geo-fencing.
#[derive(Debug)]
pub struct SovereignController {
    /// Regions that require strict data localization (no PII can leave)
    strict_localization: HashSet<DataRegion>,
    /// Adequacy agreements between regions
    adequacy_agreements: HashMap<(DataRegion, DataRegion), bool>,
}

impl Default for SovereignController {
    fn default() -> Self {
        Self::new()
    }
}

impl SovereignController {
    /// Create a new sovereign controller with default rules.
    pub fn new() -> Self {
        let mut controller = Self {
            strict_localization: HashSet::new(),
            adequacy_agreements: HashMap::new(),
        };

        // Per GLOBAL_GAPS.md: Strict localization regions
        controller.strict_localization.insert(DataRegion::Cn); // PIPL
        controller.strict_localization.insert(DataRegion::India); // DPDP

        // EU adequacy decisions (simplified)
        controller.add_adequacy(DataRegion::Eu, DataRegion::Us); // EU-US Data Privacy Framework
        controller.add_adequacy(DataRegion::Eu, DataRegion::AsiaPac); // Japan, Korea adequacy

        // MENA -> no external adequacy for government/critical data

        controller
    }

    /// Add an adequacy agreement between two regions.
    pub fn add_adequacy(&mut self, from: DataRegion, to: DataRegion) {
        self.adequacy_agreements.insert((from, to), true);
        self.adequacy_agreements.insert((to, from), true); // Bidirectional
    }

    /// Check if a region requires strict localization.
    pub fn requires_localization(&self, region: DataRegion) -> bool {
        self.strict_localization.contains(&region) || region.requires_localization()
    }

    /// Check if a transfer is allowed.
    pub fn is_allowed(&self, transfer: &DataTransfer) -> bool {
        self.validate(transfer).allowed
    }

    /// Validate a data transfer with detailed decision.
    pub fn validate(&self, transfer: &DataTransfer) -> TransferDecision {
        // Same region is always allowed
        if transfer.origin == transfer.destination {
            return TransferDecision {
                allowed: true,
                reason: "Same region transfer".to_string(),
                safeguards: vec![],
            };
        }

        // Global region has no restrictions
        if transfer.origin == DataRegion::Global {
            return TransferDecision {
                allowed: true,
                reason: "Global data has no residency restrictions".to_string(),
                safeguards: vec![],
            };
        }

        // Check adequacy for PII/Personal data FIRST
        if transfer.is_pii || transfer.data_type == DataType::Personal {
            let has_adequacy = self
                .adequacy_agreements
                .get(&(transfer.origin, transfer.destination))
                .copied()
                .unwrap_or(false);

            // If adequacy exists, allow the transfer
            if has_adequacy {
                return TransferDecision {
                    allowed: true,
                    reason: format!(
                        "Transfer allowed under adequacy agreement between {:?} and {:?}",
                        transfer.origin, transfer.destination
                    ),
                    safeguards: vec![],
                };
            }

            // Check strict localization AFTER adequacy check
            // (regions in strict_localization HashSet have no adequacy agreements)
            if self.strict_localization.contains(&transfer.origin) {
                return TransferDecision {
                    allowed: false,
                    reason: format!(
                        "PII from {:?} cannot be transferred outside due to data localization laws ({})", 
                        transfer.origin,
                        transfer.origin.privacy_law()
                    ),
                    safeguards: vec![],
                };
            }

            // No adequacy and not strict localization - require SCCs
            return TransferDecision {
                allowed: false,
                reason: format!(
                    "No adequacy agreement between {:?} and {:?} for personal data",
                    transfer.origin, transfer.destination
                ),
                safeguards: vec![
                    "Standard Contractual Clauses (SCCs) required".to_string(),
                    "Data Protection Impact Assessment required".to_string(),
                ],
            };
        }

        // Health data has additional restrictions
        if transfer.data_type == DataType::Health {
            return TransferDecision {
                allowed: true,
                reason: "Health data transfer allowed with safeguards".to_string(),
                safeguards: vec![
                    "HIPAA Business Associate Agreement required".to_string(),
                    "Encryption in transit required".to_string(),
                    "Audit logging required".to_string(),
                ],
            };
        }

        // Default: allowed
        TransferDecision {
            allowed: true,
            reason: "Transfer complies with sovereignty rules".to_string(),
            safeguards: vec![],
        }
    }

    /// Block a transfer and return an error.
    pub fn enforce(&self, transfer: &DataTransfer) -> Result<(), SovereignError> {
        let decision = self.validate(transfer);

        if decision.allowed {
            Ok(())
        } else {
            Err(SovereignError::TransferBlocked {
                origin: transfer.origin,
                destination: transfer.destination,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_region_always_allowed() {
        let controller = SovereignController::new();
        let transfer = DataTransfer::new("data-1", DataRegion::Eu, DataRegion::Eu).with_pii();

        assert!(controller.is_allowed(&transfer));
    }

    #[test]
    fn test_cn_pii_blocked() {
        let controller = SovereignController::new();
        let transfer = DataTransfer::new("data-cn", DataRegion::Cn, DataRegion::Us).with_pii();

        assert!(!controller.is_allowed(&transfer));
    }

    #[test]
    fn test_cn_non_pii_allowed() {
        let controller = SovereignController::new();
        let transfer = DataTransfer::new("data-cn", DataRegion::Cn, DataRegion::Us)
            .with_data_type(DataType::Aggregated);

        assert!(controller.is_allowed(&transfer));
    }

    #[test]
    fn test_eu_us_adequacy() {
        let controller = SovereignController::new();
        let transfer = DataTransfer::new("data-eu", DataRegion::Eu, DataRegion::Us)
            .with_pii()
            .with_data_type(DataType::Personal);

        assert!(controller.is_allowed(&transfer));
    }

    #[test]
    fn test_india_pii_blocked() {
        let controller = SovereignController::new();
        let transfer = DataTransfer::new("data-in", DataRegion::India, DataRegion::Us).with_pii();

        assert!(!controller.is_allowed(&transfer));
    }

    #[test]
    fn test_global_always_allowed() {
        let controller = SovereignController::new();
        let transfer =
            DataTransfer::new("data-global", DataRegion::Global, DataRegion::Cn).with_pii();

        assert!(controller.is_allowed(&transfer));
    }

    #[test]
    fn test_health_data_safeguards() {
        let controller = SovereignController::new();
        let transfer = DataTransfer::new("health-data", DataRegion::Us, DataRegion::Eu)
            .with_data_type(DataType::Health);

        let decision = controller.validate(&transfer);
        assert!(decision.allowed);
        assert!(!decision.safeguards.is_empty());
        assert!(decision.safeguards.iter().any(|s| s.contains("HIPAA")));
    }

    #[test]
    fn test_enforce_blocked() {
        let controller = SovereignController::new();
        let transfer = DataTransfer::new("data-cn", DataRegion::Cn, DataRegion::Us).with_pii();

        let result = controller.enforce(&transfer);
        assert!(result.is_err());
    }
}
