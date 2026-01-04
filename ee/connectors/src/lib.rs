//! Enterprise Connectors Module
#![allow(unused)]
//!
//! Per LICENSING.md: These connectors require enterprise license.
//! Per ee/LICENSE-ENTERPRISE.md: Commercial use requires subscription.
//!
//! Features:
//! - SAP RFC/BAPI/OData/Event Mesh
//! - SWIFT MX (ISO 20022), GPI, Sanctions
//! - Mainframe CICS, IMS, MQ

pub mod sap;
pub mod swift;
pub mod mainframe;
pub mod license;

// Re-exports
pub use sap::{SapConnector, SapConfig, RfcConnection, BapiCaller};
pub use swift::{SwiftConnector, SwiftConfig, MxParser, GpiTracker};
pub use mainframe::{MainframeConnector, CicsClient, ImsClient, MqClient};
pub use license::{check_license, LicenseError};
