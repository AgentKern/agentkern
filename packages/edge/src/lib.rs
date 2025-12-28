#![allow(dead_code)]
#![allow(unused)]
//! AgentKern Edge - Minimal Kernel for IoT/Edge Devices
//!
//! A lightweight version of AgentKern for constrained environments:
//! - ARM embedded devices
//! - Warehouse robots
//! - Drones
//! - IoT sensors
//!
//! Designed for:
//! - Low memory footprint (<1MB RAM)
//! - Offline operation
//! - Real-time constraints
//! - Battery-powered devices

#![cfg_attr(feature = "embedded", no_std)]

#[cfg(feature = "embedded")]
extern crate alloc;

pub mod minimal;
pub mod policy;
pub mod offline;

pub use minimal::{EdgeRuntime, EdgeConfig, EdgeError};
pub use policy::{EdgePolicy, PolicyRule, PolicyAction};
pub use offline::{OfflineAgent, OfflineState, SyncStrategy};

/// Edge runtime version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum memory for edge runtime (bytes).
pub const MAX_MEMORY: usize = 1024 * 1024; // 1MB default

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_max_memory() {
        assert_eq!(MAX_MEMORY, 1024 * 1024);
    }
}
