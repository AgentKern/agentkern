//! Legacy System Connectors
//!
//! Per MANDATE.md Section 5: Zero-Trust Security
//! Per Strategic Roadmap: Legacy Bridge for Enterprise ERP/SQL
//!
//! This module provides WASM-isolated connectors for legacy systems:
//! - Generic SQL (JDBC bridge)
//!
//! All connectors run in WASM sandboxes with policy enforcement through Gate.

pub mod mock;
pub mod registry;

pub mod sdk;
pub mod sql;


// Re-exports
pub use mock::MockConnector;
pub use registry::{ConnectorRegistry, RegisteredConnector};

pub use sdk::{
    ConnectorConfig, ConnectorError, ConnectorHealth, ConnectorProtocol, ConnectorResult,
    LegacyConnector,
};
pub use sql::SqlConnector;
