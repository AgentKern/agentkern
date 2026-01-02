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

mod a2a;
mod adapter;
mod agentkern;
mod mcp;
mod nlip; // ECMA-430 Natural Language Interaction Protocol (Dec 2025)
pub mod translator;

pub use crate::types::Protocol;
pub use adapter::{AdapterRegistry, ProtocolAdapter};
pub use translator::{FieldMapping, ProtocolTranslator, TranslationResult};

// Re-export specific adapters when features enabled
#[cfg(feature = "a2a")]
pub use a2a::A2AAdapter;

#[cfg(feature = "mcp")]
pub use mcp::MCPAdapter;

#[cfg(feature = "nlip")]
pub use nlip::{NLIPAdapter, NLIPContent, NLIPEnvelope};

pub use agentkern::AgentKernAdapter;
