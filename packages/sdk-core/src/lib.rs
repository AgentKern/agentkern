//! AgentKern SDK Core
//!
//! Production-grade Rust library for Agent identity, cryptographic signing,
//! and Liability Proof creation. This is THE TRUTH - all language bindings
//! (Node.js, Python, C#, Swift) are generated from this core.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     sdk-core (Rust)                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Agent          │ Identity + Keypair Management            │
//! │  Signing        │ Ed25519 Sign/Verify                      │
//! │  Proof          │ Liability Proof Creation & Validation    │
//! │  Protocol       │ A2A Message Encoding                     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_sdk_core::{Agent, LiabilityProof};
//!
//! // Generate a new agent with Ed25519 keypair
//! let agent = Agent::generate("my-agent")?;
//!
//! // Create a signed liability proof
//! let proof = agent.create_proof("payment:transfer:100")?;
//!
//! // Verify a proof
//! let is_valid = Agent::verify_proof(&proof)?;
//! ```

#![warn(missing_docs)]
#![warn(rust_2024_compatibility)]

pub mod agent;
pub mod error;
pub mod proof;
pub mod protocol;
pub mod signing;

// Re-exports for ergonomic API
pub use agent::{Agent, AgentConfig, AgentId};
pub use error::{SdkError, SdkResult};
pub use proof::{LiabilityProof, ProofClaims, ProofHeader};
pub use protocol::{A2AMessage, MessageType};
pub use signing::{KeyPair, PublicKey, Signature};

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default proof expiration in seconds (5 minutes)
pub const DEFAULT_PROOF_EXPIRY_SECONDS: u64 = 300;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_full_flow() {
        // Generate agent
        let agent = Agent::generate("test-agent").expect("Failed to generate agent");

        // Create proof
        let proof = agent
            .create_proof("test:action")
            .expect("Failed to create proof");

        // Verify proof
        assert!(Agent::verify_proof(&proof).expect("Verification failed"));
    }
}
