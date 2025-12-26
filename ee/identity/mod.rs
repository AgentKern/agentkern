//! Identity Provider Integration
//!
//! Bridge between VeriMantle DIDs and enterprise identity providers
//! Supports: Entra, Okta, Auth0, Ping Identity, etc.
//! Trust score provider for Zero Trust Conditional Access
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod bridge;
pub mod trust;
pub mod demo;

pub use bridge::{IdentityBridge, IdentityConfig, AgentRegistration};
pub use trust::{TrustScoreProvider, TrustScore, TrustFactors};
pub use demo::{DemoIdentity, IdentityFactory};

