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

pub mod license;
pub mod mainframe;
pub mod sap;
pub mod swift;

// Re-exports
pub use license::{LicenseError, check_license};
pub use mainframe::{CicsClient, ImsClient, MainframeConnector, MqClient};
pub use sap::{BapiCaller, RfcConnection, SapConfig, SapConnector};
pub use swift::{GpiTracker, MxParser, SwiftConfig, SwiftConnector};
