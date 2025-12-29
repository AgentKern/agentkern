//! Healthcare Compliance Module
//!
//! - HIPAA (Health Insurance Portability and Accountability Act)
//! - HITECH (Health Information Technology for Economic and Clinical Health)
//! - HL7/FHIR interoperability

mod hipaa;
mod fhir;

pub use hipaa::*;
pub use fhir::*;
