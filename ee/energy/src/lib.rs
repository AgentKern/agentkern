//! Enterprise Energy Module
//!
//! Per LICENSING.md: Real-time grid API, Intersect integration
//! Per licensing_split.md: Enterprise tier (Google acquisition target)
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod grid;
pub mod intersect;
pub mod demo;

// Re-exports
pub use grid::{GridApi, CarbonIntensityFeed, RegionData};
pub use intersect::{IntersectClient, IntersectConfig};
pub use demo::{DemoGridApi, GridFactory};

