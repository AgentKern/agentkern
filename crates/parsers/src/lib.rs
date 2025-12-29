//! AgentKern-Parsers: Legacy protocol message formats
//!
//! Extracted from agentkern-gate core to improve modularity and enable
//! domain-specific logic to run in WASM actors.

pub mod copybook;
pub mod idoc;
pub mod swift_mt;

// Re-exports
pub use copybook::{CopybookField, CopybookParser, CopybookRecord};
pub use idoc::{IDocMessage, IDocParser, IDocSegment};
pub use swift_mt::{SwiftField, SwiftMtMessage, SwiftMtParser};
