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

pub mod sdk;
pub mod registry;
pub mod mock;
pub mod sql;
pub mod parsers;

// Re-exports
pub use sdk::{
    LegacyConnector, ConnectorProtocol, ConnectorConfig, 
    ConnectorHealth, ConnectorError, ConnectorResult,
};
pub use registry::{ConnectorRegistry, RegisteredConnector};
pub use mock::MockConnector;
pub use sql::SqlConnector;
