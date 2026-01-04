//! External Identity Provider Federation
//!
//! NOTE: This is for EXTERNAL identity providers (Entra, Okta, Auth0)
//! AgentKern's core identity is in:
//!   - apps/identity/ (NestJS service)
//!   - apps/gateway/src/services/identity.service.ts
//!
//! This module federates external IDP agent IDs with AgentKern DIDs
//! Trust score provider for Zero Trust Conditional Access
//!
//! Graceful Degradation: Works with credentials, demo mode without

pub mod bridge;
pub mod trust;
pub mod demo;

pub use bridge::{IdentityBridge, IdentityConfig, AgentRegistration};
pub use trust::{TrustScoreProvider, TrustScore, TrustFactors};
pub use demo::{DemoIdentity, IdentityFactory};

