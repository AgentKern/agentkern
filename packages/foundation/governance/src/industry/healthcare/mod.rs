//! Healthcare Compliance Module
//!
//! - HIPAA (Health Insurance Portability and Accountability Act)
//! - HITECH (Health Information Technology for Economic and Clinical Health)
//! - HL7/FHIR interoperability

pub mod fhir;
pub mod hipaa;

pub use fhir::*;
pub use hipaa::*;
