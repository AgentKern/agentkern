//! Protocol Parsers - Parse legacy protocol message formats
//!
//! These parsers are Open Source (Community tier) to enable testing
//! and development. Production connectors require Enterprise license.

pub mod swift_mt;
pub mod idoc;
pub mod copybook;

// Re-exports
pub use swift_mt::{SwiftMtParser, SwiftMtMessage, SwiftField};
pub use idoc::{IDocParser, IDocMessage, IDocSegment};
pub use copybook::{CopybookParser, CopybookField, CopybookRecord};
