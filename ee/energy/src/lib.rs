//! Enterprise Energy Module
//!
//! Per LICENSING.md: Real-time grid API, Intersect integration
//! Per licensing_split.md: Enterprise tier (Google acquisition target)
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod demo;
pub mod grid;
pub mod intersect;

// Re-exports
pub use demo::{DemoGridApi, GridFactory};
pub use grid::{CarbonIntensityFeed, GridApi, RegionData};
pub use intersect::{IntersectClient, IntersectConfig};
