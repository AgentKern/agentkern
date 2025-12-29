//! AgentKern-Compliance: Re-exports from agentkern-governance
//!
//! This crate is a thin wrapper that re-exports compliance modules from
//! `agentkern-governance::industry` for backward compatibility.
//!
//! **For new development, use `agentkern-governance` directly.**

// Re-export modules from governance
pub use agentkern_governance::industry::healthcare::hipaa;
pub use agentkern_governance::industry::finance::pci;
pub use agentkern_governance::industry::finance::shariah as shariah_compliance;

// Re-export commonly used types (whatever exists)
pub use agentkern_governance::industry::healthcare::*;
pub use agentkern_governance::industry::finance::*;
