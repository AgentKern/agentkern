//! Legacy System Connectors
//!
//! Per MANDATE.md Section 5: Zero-Trust Security
//! Per Strategic Roadmap: Legacy Bridge for Enterprise ERP/SQL
//!
//! This module provides WASM-isolated connectors for legacy enterprise systems:
//! - SAP (RFC, BAPI, OData, IDOC)
//! - SWIFT (MT/MX ISO 20022)
//! - COBOL/Mainframe (CICS, IMS)
//! - Generic SQL (JDBC bridge)
//!
//! All connectors run in WASM sandboxes with policy enforcement through Gate.

pub mod mock;
pub mod parsers;
pub mod registry;
pub mod sap;    // Production SAP RFC connector
pub mod sdk;
pub mod sql;
pub mod swift;  // Production SWIFT GPI connector

// Re-exports
pub use mock::MockConnector;
pub use registry::{ConnectorRegistry, RegisteredConnector};
pub use sap::SapRfcConnector;
pub use sdk::{
    ConnectorConfig, ConnectorError, ConnectorHealth, ConnectorProtocol, ConnectorResult,
    LegacyConnector,
};
pub use sql::SqlConnector;
pub use swift::SwiftGpiConnector;
