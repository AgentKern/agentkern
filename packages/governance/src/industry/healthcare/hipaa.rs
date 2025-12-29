//! AgentKern-Gate: HIPAA Compliance Module
//!
//! Per EXECUTION_MANDATE.md §2: "Healthcare: HIPAA, HITECH, EU MDR, HL7/FHIR"
//!
//! Features:
//! - PHI (Protected Health Information) detection
//! - Business Associate Agreement (BAA) validation
//! - Minimum necessary access enforcement
//! - Audit logging requirements
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::hipaa::{HipaaValidator, DataType};
//!
//! let validator = HipaaValidator::new();
//! validator.check_phi(&data)?;
//! validator.validate_access(&agent, &resource)?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// HIPAA compliance error.
#[derive(Debug, Error)]
pub enum HipaaError {
    #[error("PHI detected in data: {identifiers:?}")]
    PhiDetected { identifiers: Vec<String> },
    #[error("No BAA in place with entity: {entity}")]
    NoBaaInPlace { entity: String },
    #[error("Access violates minimum necessary principle")]
    MinimumNecessaryViolation,
    #[error("Unauthorized PHI disclosure")]
    UnauthorizedDisclosure,
    #[error("Audit logging required for this operation")]
    AuditRequired,
    #[error("Encryption required for PHI in transit")]
    EncryptionRequired,
}

/// Types of PHI identifiers per HIPAA Safe Harbor (18 identifiers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhiIdentifier {
    /// Names
    Name,
    /// Geographic data (smaller than state)
    GeographicData,
    /// Dates (birth, admission, discharge, death)
    Dates,
    /// Phone numbers
    PhoneNumber,
    /// Fax numbers
    FaxNumber,
    /// Email addresses
    Email,
    /// Social Security numbers
    Ssn,
    /// Medical record numbers
    MedicalRecordNumber,
    /// Health plan beneficiary numbers
    HealthPlanNumber,
    /// Account numbers
    AccountNumber,
    /// Certificate/license numbers
    LicenseNumber,
    /// Vehicle identifiers
    VehicleId,
    /// Device identifiers
    DeviceId,
    /// Web URLs
    WebUrl,
    /// IP addresses
    IpAddress,
    /// Biometric identifiers
    BiometricId,
    /// Full face photos
    PhotoImage,
    /// Any other unique identifier
    OtherUniqueId,
}

impl PhiIdentifier {
    /// Get pattern hints for detection.
    pub fn pattern_hints(&self) -> &'static [&'static str] {
        match self {
            Self::Ssn => &["SSN", "social security", "XXX-XX-XXXX"],
            Self::PhoneNumber => &["phone", "tel", "mobile", "cell"],
            Self::Email => &["email", "@", "e-mail"],
            Self::MedicalRecordNumber => &["MRN", "medical record", "patient ID"],
            Self::HealthPlanNumber => &["member ID", "plan ID", "insurance ID"],
            Self::IpAddress => &["IP", "address", "IPv4", "IPv6"],
            Self::Dates => &["DOB", "birth", "date of birth", "admission date"],
            Self::Name => &["name", "patient name", "first name", "last name"],
            _ => &[],
        }
    }
}

/// PHI detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiScanResult {
    /// Was PHI detected?
    pub contains_phi: bool,
    /// Detected identifier types
    pub identifiers_found: Vec<PhiIdentifier>,
    /// Confidence score (0-100)
    pub confidence: u8,
    /// Recommended action
    pub recommendation: String,
}

/// Access request for HIPAA validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    /// Requesting agent/user ID
    pub requester_id: String,
    /// Role of requester
    pub role: HipaaRole,
    /// Resource being accessed
    pub resource_id: String,
    /// Purpose of access
    pub purpose: AccessPurpose,
    /// Is this emergency access?
    pub is_emergency: bool,
}

/// HIPAA roles for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HipaaRole {
    /// Healthcare provider (doctor, nurse)
    Provider,
    /// Administrative staff
    Admin,
    /// Billing/claims processor
    Billing,
    /// IT/Technical support
    Technical,
    /// Business associate
    BusinessAssociate,
    /// Patient/member
    Patient,
    /// Researcher (IRB approved)
    Researcher,
    /// AI/Automated system
    AiAgent,
}

/// Purpose of accessing PHI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessPurpose {
    /// Treatment of patient
    Treatment,
    /// Payment/billing operations
    Payment,
    /// Healthcare operations
    Operations,
    /// Research (with authorization)
    Research,
    /// Public health reporting
    PublicHealth,
    /// Law enforcement (with warrant)
    LawEnforcement,
    /// Emergency treatment
    Emergency,
    /// Patient request
    PatientRequest,
}

impl AccessPurpose {
    /// Check if this purpose is TPO (Treatment, Payment, Operations).
    pub fn is_tpo(&self) -> bool {
        matches!(self, Self::Treatment | Self::Payment | Self::Operations)
    }
}

/// Business Associate Agreement status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaaStatus {
    /// Entity name
    pub entity: String,
    /// Is BAA signed?
    pub signed: bool,
    /// Signature date
    pub signed_date: Option<String>,
    /// Expiry date
    pub expiry_date: Option<String>,
    /// BAA document ID
    pub document_id: Option<String>,
}

/// HIPAA compliance validator.
#[derive(Debug)]
pub struct HipaaValidator {
    /// Entities with valid BAAs
    valid_baas: HashSet<String>,
    /// Strict mode (reject any potential violation)
    strict_mode: bool,
}

impl Default for HipaaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl HipaaValidator {
    /// Create a new HIPAA validator.
    pub fn new() -> Self {
        Self {
            valid_baas: HashSet::new(),
            strict_mode: false,
        }
    }

    /// Create a strict validator.
    pub fn strict() -> Self {
        Self {
            valid_baas: HashSet::new(),
            strict_mode: true,
        }
    }

    /// Register a valid BAA.
    pub fn register_baa(&mut self, entity: impl Into<String>) {
        self.valid_baas.insert(entity.into());
    }

    /// Check if an entity has a valid BAA.
    pub fn has_baa(&self, entity: &str) -> bool {
        self.valid_baas.contains(entity)
    }

    /// Scan text for potential PHI.
    pub fn scan_for_phi(&self, text: &str) -> PhiScanResult {
        let mut identifiers = Vec::new();
        let text_lower = text.to_lowercase();

        // Check for SSN pattern (XXX-XX-XXXX)
        if text.contains('-') && text.chars().filter(|c| c.is_numeric()).count() == 9 {
            identifiers.push(PhiIdentifier::Ssn);
        }

        // Check for email pattern
        if text.contains('@') && text.contains('.') {
            identifiers.push(PhiIdentifier::Email);
        }

        // Check for phone pattern
        if text_lower.contains("phone") || text.chars().filter(|c| c.is_numeric()).count() >= 10 {
            if text.contains('-') || text.contains('(') {
                identifiers.push(PhiIdentifier::PhoneNumber);
            }
        }

        // Check for date of birth
        if text_lower.contains("dob") || text_lower.contains("date of birth") {
            identifiers.push(PhiIdentifier::Dates);
        }

        // Check for MRN
        if text_lower.contains("mrn") || text_lower.contains("medical record") {
            identifiers.push(PhiIdentifier::MedicalRecordNumber);
        }

        // Check for IP address
        if text.split('.').count() == 4 && text.chars().filter(|c| c.is_numeric()).count() >= 4 {
            identifiers.push(PhiIdentifier::IpAddress);
        }

        let contains_phi = !identifiers.is_empty();
        let confidence = if contains_phi {
            (identifiers.len() * 25).min(100) as u8
        } else {
            0
        };

        PhiScanResult {
            contains_phi,
            identifiers_found: identifiers,
            confidence,
            recommendation: if contains_phi {
                "Apply encryption and access controls before storage/transmission".to_string()
            } else {
                "No PHI detected, standard handling allowed".to_string()
            },
        }
    }

    /// Validate an access request against minimum necessary principle.
    pub fn validate_access(&self, request: &AccessRequest) -> Result<(), HipaaError> {
        // Emergency access bypasses normal checks (but must be audited)
        if request.is_emergency {
            tracing::warn!(
                requester = %request.requester_id,
                resource = %request.resource_id,
                "EMERGENCY PHI access - requires audit review"
            );
            return Ok(());
        }

        // Check if purpose is valid TPO or authorized
        if !request.purpose.is_tpo() {
            // Non-TPO access requires additional authorization
            match request.purpose {
                AccessPurpose::Research => {
                    // Research requires IRB approval
                    tracing::info!("Research access - verify IRB approval");
                }
                AccessPurpose::PatientRequest => {
                    // Patient has right to their own data
                    if request.role != HipaaRole::Patient {
                        return Err(HipaaError::MinimumNecessaryViolation);
                    }
                }
                _ => {
                    if self.strict_mode {
                        return Err(HipaaError::MinimumNecessaryViolation);
                    }
                }
            }
        }

        // Validate role-based access
        match (&request.role, &request.purpose) {
            (HipaaRole::Provider, AccessPurpose::Treatment) => Ok(()),
            (HipaaRole::Admin, AccessPurpose::Operations) => Ok(()),
            (HipaaRole::Billing, AccessPurpose::Payment) => Ok(()),
            (HipaaRole::Patient, AccessPurpose::PatientRequest) => Ok(()),
            (HipaaRole::AiAgent, _) => {
                // AI agents need explicit authorization
                tracing::warn!("AI agent PHI access - verify authorization");
                Ok(())
            }
            _ => {
                if self.strict_mode {
                    Err(HipaaError::MinimumNecessaryViolation)
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Validate that BAA is in place for a business associate.
    pub fn require_baa(&self, entity: &str) -> Result<(), HipaaError> {
        if self.has_baa(entity) {
            Ok(())
        } else {
            Err(HipaaError::NoBaaInPlace {
                entity: entity.to_string(),
            })
        }
    }
}

/// HIPAA-compliant audit record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HipaaAuditRecord {
    /// Timestamp
    pub timestamp: u64,
    /// User/Agent ID
    pub user_id: String,
    /// Action performed
    pub action: String,
    /// Resource accessed
    pub resource_id: String,
    /// Was PHI involved?
    pub phi_involved: bool,
    /// Access purpose
    pub purpose: AccessPurpose,
    /// Outcome (success/failure)
    pub outcome: String,
    /// Client IP
    pub client_ip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phi_detection_ssn() {
        let validator = HipaaValidator::new();
        let result = validator.scan_for_phi("Patient SSN: 123-45-6789");

        assert!(result.contains_phi);
        assert!(result.identifiers_found.contains(&PhiIdentifier::Ssn));
    }

    #[test]
    fn test_phi_detection_email() {
        let validator = HipaaValidator::new();
        let result = validator.scan_for_phi("Contact: patient@hospital.com");

        assert!(result.contains_phi);
        assert!(result.identifiers_found.contains(&PhiIdentifier::Email));
    }

    #[test]
    fn test_no_phi() {
        let validator = HipaaValidator::new();
        let result = validator.scan_for_phi("General health information");

        assert!(!result.contains_phi);
    }

    #[test]
    fn test_baa_validation() {
        let mut validator = HipaaValidator::new();

        assert!(validator.require_baa("new-vendor").is_err());

        validator.register_baa("trusted-vendor");
        assert!(validator.require_baa("trusted-vendor").is_ok());
    }

    #[test]
    fn test_access_validation() {
        let validator = HipaaValidator::new();

        // Provider treatment access should be allowed
        let request = AccessRequest {
            requester_id: "dr-smith".to_string(),
            role: HipaaRole::Provider,
            resource_id: "patient-123".to_string(),
            purpose: AccessPurpose::Treatment,
            is_emergency: false,
        };

        assert!(validator.validate_access(&request).is_ok());
    }

    #[test]
    fn test_emergency_access() {
        let validator = HipaaValidator::new();

        let request = AccessRequest {
            requester_id: "emt-456".to_string(),
            role: HipaaRole::Provider,
            resource_id: "patient-123".to_string(),
            purpose: AccessPurpose::Emergency,
            is_emergency: true,
        };

        assert!(validator.validate_access(&request).is_ok());
    }
}
