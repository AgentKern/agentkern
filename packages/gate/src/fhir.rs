//! AgentKern-Gate: FHIR Healthcare Integration
//!
//! Per EXECUTION_MANDATE.md §2: "Healthcare: HIPAA, HITECH, EU MDR, HL7/FHIR"
//!
//! Features:
//! - FHIR R4 resource handling
//! - Patient, Practitioner, Observation resources
//! - Consent management
//! - Audit event generation
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::fhir::{FhirClient, Patient};
//!
//! let client = FhirClient::new("https://fhir.example.com");
//! let patient = client.get_patient("12345")?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FHIR resource types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Patient,
    Practitioner,
    Observation,
    Condition,
    Medication,
    MedicationRequest,
    AllergyIntolerance,
    Immunization,
    Procedure,
    DiagnosticReport,
    DocumentReference,
    Consent,
    AuditEvent,
    Bundle,
}

impl ResourceType {
    /// Get the FHIR URL path for this resource.
    pub fn path(&self) -> &'static str {
        match self {
            Self::Patient => "Patient",
            Self::Practitioner => "Practitioner",
            Self::Observation => "Observation",
            Self::Condition => "Condition",
            Self::Medication => "Medication",
            Self::MedicationRequest => "MedicationRequest",
            Self::AllergyIntolerance => "AllergyIntolerance",
            Self::Immunization => "Immunization",
            Self::Procedure => "Procedure",
            Self::DiagnosticReport => "DiagnosticReport",
            Self::DocumentReference => "DocumentReference",
            Self::Consent => "Consent",
            Self::AuditEvent => "AuditEvent",
            Self::Bundle => "Bundle",
        }
    }
}

/// FHIR Patient resource (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    /// Resource type (always "Patient")
    pub resource_type: String,
    /// Patient ID
    pub id: String,
    /// Active status
    #[serde(default)]
    pub active: bool,
    /// Patient name
    pub name: Vec<HumanName>,
    /// Gender
    pub gender: Option<String>,
    /// Birth date
    pub birth_date: Option<String>,
    /// Contact info
    #[serde(default)]
    pub telecom: Vec<ContactPoint>,
    /// Address
    #[serde(default)]
    pub address: Vec<Address>,
}

/// FHIR HumanName type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanName {
    pub family: Option<String>,
    #[serde(default)]
    pub given: Vec<String>,
    pub prefix: Option<Vec<String>>,
    pub suffix: Option<Vec<String>>,
}

impl HumanName {
    /// Get full display name.
    pub fn display(&self) -> String {
        let given = self.given.join(" ");
        let family = self.family.as_deref().unwrap_or("");
        format!("{} {}", given, family).trim().to_string()
    }
}

/// FHIR ContactPoint type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactPoint {
    pub system: Option<String>, // phone, email, etc.
    pub value: Option<String>,
    #[serde(rename = "use")]
    pub use_: Option<String>, // home, work, mobile
}

/// FHIR Address type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    #[serde(rename = "use")]
    pub use_: Option<String>,
    pub line: Option<Vec<String>>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

/// FHIR Observation resource (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    pub resource_type: String,
    pub id: String,
    pub status: String,
    pub code: CodeableConcept,
    pub subject: Reference,
    pub effective_date_time: Option<String>,
    pub value_quantity: Option<Quantity>,
    pub value_string: Option<String>,
}

/// FHIR CodeableConcept type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeableConcept {
    #[serde(default)]
    pub coding: Vec<Coding>,
    pub text: Option<String>,
}

/// FHIR Coding type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coding {
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

/// FHIR Reference type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub reference: Option<String>,
    pub display: Option<String>,
}

/// FHIR Quantity type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub system: Option<String>,
    pub code: Option<String>,
}

/// FHIR Consent resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Consent {
    pub resource_type: String,
    pub id: String,
    pub status: ConsentStatus,
    pub scope: CodeableConcept,
    pub patient: Reference,
    pub date_time: Option<String>,
    #[serde(default)]
    pub performer: Vec<Reference>,
}

/// Consent status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsentStatus {
    Draft,
    Proposed,
    Active,
    Rejected,
    Inactive,
    EnteredInError,
}

/// FHIR Audit Event for compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub resource_type: String,
    pub id: String,
    #[serde(rename = "type")]
    pub type_: Coding,
    pub action: AuditAction,
    pub recorded: String,
    #[serde(default)]
    pub agent: Vec<AuditAgent>,
    pub source: AuditSource,
    #[serde(default)]
    pub entity: Vec<AuditEntity>,
}

/// Audit action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    C, // Create
    R, // Read
    U, // Update
    D, // Delete
    E, // Execute
}

/// Audit agent (who did it).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAgent {
    pub who: Reference,
    pub requestor: bool,
}

/// Audit source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSource {
    pub observer: Reference,
}

/// Audit entity (what was accessed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntity {
    pub what: Reference,
}

/// FHIR client for healthcare integrations.
#[derive(Debug)]
pub struct FhirClient {
    base_url: String,
    auth_token: Option<String>,
}

impl FhirClient {
    /// Create a new FHIR client.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            auth_token: None,
        }
    }

    /// Set authentication token.
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Build URL for a resource.
    pub fn resource_url(&self, resource: ResourceType, id: Option<&str>) -> String {
        match id {
            Some(id) => format!("{}/{}/{}", self.base_url, resource.path(), id),
            None => format!("{}/{}", self.base_url, resource.path()),
        }
    }

    /// Create an audit event for HIPAA compliance.
    pub fn create_audit_event(
        &self,
        action: AuditAction,
        agent_id: &str,
        resource_type: ResourceType,
        resource_id: &str,
    ) -> AuditEvent {
        let now = chrono::Utc::now().to_rfc3339();
        
        AuditEvent {
            resource_type: "AuditEvent".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            type_: Coding {
                system: Some("http://terminology.hl7.org/CodeSystem/audit-event-type".to_string()),
                code: Some("rest".to_string()),
                display: Some("RESTful Operation".to_string()),
            },
            action,
            recorded: now,
            agent: vec![AuditAgent {
                who: Reference {
                    reference: Some(format!("Practitioner/{}", agent_id)),
                    display: None,
                },
                requestor: true,
            }],
            source: AuditSource {
                observer: Reference {
                    reference: Some("Device/agentkern-gate".to_string()),
                    display: Some("AgentKern Gate".to_string()),
                },
            },
            entity: vec![AuditEntity {
                what: Reference {
                    reference: Some(format!("{}/{}", resource_type.path(), resource_id)),
                    display: None,
                },
            }],
        }
    }
}

/// FHIR Bundle for batch operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub resource_type: String,
    #[serde(rename = "type")]
    pub type_: BundleType,
    #[serde(default)]
    pub entry: Vec<BundleEntry>,
    pub total: Option<u32>,
}

/// Bundle type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BundleType {
    Document,
    Message,
    Transaction,
    TransactionResponse,
    Batch,
    BatchResponse,
    History,
    Searchset,
    Collection,
}

/// Bundle entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleEntry {
    pub full_url: Option<String>,
    pub resource: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patient_name_display() {
        let name = HumanName {
            family: Some("Smith".to_string()),
            given: vec!["John".to_string(), "Michael".to_string()],
            prefix: None,
            suffix: None,
        };
        
        assert_eq!(name.display(), "John Michael Smith");
    }

    #[test]
    fn test_resource_url() {
        let client = FhirClient::new("https://fhir.example.com");
        
        assert_eq!(
            client.resource_url(ResourceType::Patient, Some("123")),
            "https://fhir.example.com/Patient/123"
        );
        
        assert_eq!(
            client.resource_url(ResourceType::Observation, None),
            "https://fhir.example.com/Observation"
        );
    }

    #[test]
    fn test_audit_event_creation() {
        let client = FhirClient::new("https://fhir.example.com");
        let event = client.create_audit_event(
            AuditAction::R,
            "dr-smith",
            ResourceType::Patient,
            "patient-123",
        );
        
        assert_eq!(event.resource_type, "AuditEvent");
        assert_eq!(event.action, AuditAction::R);
        assert!(!event.agent.is_empty());
    }

    #[test]
    fn test_consent_status() {
        let consent = Consent {
            resource_type: "Consent".to_string(),
            id: "consent-1".to_string(),
            status: ConsentStatus::Active,
            scope: CodeableConcept { coding: vec![], text: Some("patient-privacy".to_string()) },
            patient: Reference { reference: Some("Patient/123".to_string()), display: None },
            date_time: Some("2025-01-01".to_string()),
            performer: vec![],
        };
        
        assert_eq!(consent.status, ConsentStatus::Active);
    }
}
