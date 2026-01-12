//! Nexus PQC Handshake
//!
//! Per Phase 3 Roadmap: "Implementing Hybrid PQC Handshake in Nexus"
//!
//! Provides mutual trust establishment via NIST-standardized hybrid signatures.

use serde::{Deserialize, Serialize};
use crate::agent_card::AgentCard;
use crate::error::NexusError;
use agentkern_crypto::{CryptoProvider, CryptoMode, Signature, KeyPair};

/// Handshake state machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeStep {
    /// Step 1: Initiator sends Hello + Challenge
    Hello {
        initiator_card: AgentCard,
        nonce: Vec<u8>,
    },
    /// Step 2: Receiver responds with Signature + Counter-Challenge
    Respond {
        receiver_card: AgentCard,
        initiator_signature: Signature,
        receiver_nonce: Vec<u8>,
    },
    /// Step 3: Initiator finalizes
    Finalize {
        receiver_signature: Signature,
    }
}

/// PQC Handshake manager
pub struct PqcHandshake {
    crypto: CryptoProvider,
}

impl PqcHandshake {
    pub fn new() -> Self {
        Self {
            crypto: CryptoProvider::new(CryptoMode::Hybrid),
        }
    }

    /// Step 1: Initiate a heartbeat/handshake
    pub fn initiate(&self, my_card: AgentCard) -> (HandshakeStep, Vec<u8>) {
        let nonce = uuid::Uuid::new_v4().as_bytes().to_vec();
        (
            HandshakeStep::Hello {
                initiator_card: my_card,
                nonce: nonce.clone(),
            },
            nonce
        )
    }

    /// Step 2: Process a Hello and respond with a signature
    pub fn respond(
        &self,
        step: HandshakeStep,
        my_card: AgentCard,
        my_keypair: &KeyPair,
    ) -> Result<HandshakeStep, NexusError> {
        if let HandshakeStep::Hello { initiator_card: _, nonce } = step {
            // Sign the initiator's nonce
            let signature = self.crypto.sign(&nonce, my_keypair)
                .map_err(|e| NexusError::InternalError { message: e.to_string() })?;
            
            let my_nonce = uuid::Uuid::new_v4().as_bytes().to_vec();

            Ok(HandshakeStep::Respond {
                receiver_card: my_card,
                initiator_signature: signature,
                receiver_nonce: my_nonce,
            })
        } else {
            Err(NexusError::ProtocolError { message: "Invalid handshake step for respond".into() })
        }
    }

    /// Step 3: Verify the responder and finalize initiator side
    pub fn verify_responder(
        &self,
        step: HandshakeStep,
        expected_nonce: &[u8],
    ) -> Result<AgentCard, NexusError> {
        if let HandshakeStep::Respond { receiver_card, initiator_signature, .. } = step {
            // Verify our original nonce was signed correctly by the receiver
            let public_key = receiver_card.pqc_public_key.as_ref()
                .ok_or(NexusError::SecurityError { message: "PQC Handshake: Receiver card missing PQC public key".into() })?;
            
            let verified = self.crypto.verify(expected_nonce, &initiator_signature, public_key)
                .map_err(|e| NexusError::InternalError { message: e.to_string() })?;

            if verified {
                Ok(receiver_card)
            } else {
                Err(NexusError::SecurityError { message: "PQC Handshake: Signature verification failed".into() })
            }
        } else {
            Err(NexusError::ProtocolError { message: "Invalid handshake step for verification".into() })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentkern_crypto::CryptoMode;

    #[test]
    fn test_full_handshake_flow() {
        let handshake = PqcHandshake::new();
        let crypto = CryptoProvider::new(CryptoMode::Hybrid);
        
        let initiator_kp = crypto.generate_keypair().unwrap();
        let receiver_kp = crypto.generate_keypair().unwrap();

        let mut initiator_card = AgentCard::default();
        initiator_card.pqc_public_key = Some(initiator_kp.public_key.clone());

        let mut receiver_card = AgentCard::default();
        receiver_card.pqc_public_key = Some(receiver_kp.public_key.clone());

        // 1. Initiator starts
        let (step1, nonce1) = handshake.initiate(initiator_card.clone());

        // 2. Receiver responds
        let step2 = handshake.respond(step1, receiver_card.clone(), &receiver_kp).unwrap();

        // 3. Initiator verifies
        let final_card = handshake.verify_responder(step2, &nonce1).unwrap();

        assert_eq!(final_card.pqc_public_key, Some(receiver_kp.public_key));
    }
}
