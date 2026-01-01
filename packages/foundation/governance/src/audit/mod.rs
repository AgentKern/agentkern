//! Audit & Evidence Collection Module
//!
//! Cross-cutting compliance infrastructure:
//! - Audit ledger (ISO 42001)
//! - Evidence collection (SOC 2, FedRAMP)
//! - Compliance reporting

mod evidence;
mod ledger;

pub use evidence::*;
pub use ledger::*;
