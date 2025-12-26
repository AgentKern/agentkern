//! Protocol Adapters
//!
//! Pluggable protocol translation layer supporting current and future protocols.
//!
//! # Adding a New Protocol
//!
//! 1. Create a new file in `protocols/` (e.g., `my_protocol.rs`)
//! 2. Implement `ProtocolAdapter` trait
//! 3. Add to feature flags in Cargo.toml
//! 4. Register with `AdapterRegistry`

mod adapter;
mod a2a;
mod mcp;
mod verimantle;

pub use adapter::{ProtocolAdapter, AdapterRegistry};
pub use crate::types::Protocol;

// Re-export specific adapters when features enabled
#[cfg(feature = "a2a")]
pub use a2a::A2AAdapter;

#[cfg(feature = "mcp")]
pub use mcp::MCPAdapter;

pub use verimantle::VeriMantleAdapter;
