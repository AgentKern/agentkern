//! Protocol Parsers - Parse legacy protocol message formats
//!
//! These parsers are Open Source (Community tier) to enable testing
//! and development. Production connectors require Enterprise license.

pub mod copybook;
pub mod idoc;
pub mod swift_mt;

// Re-exports
pub use copybook::{CopybookField, CopybookParser, CopybookRecord};
pub use idoc::{IDocMessage, IDocParser, IDocSegment};
pub use swift_mt::{SwiftField, SwiftMtMessage, SwiftMtParser};
