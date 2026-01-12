//! Model Provenance & Supply Chain Security
//!
//! Per Antifragility Roadmap: "Neural Supply Chain Integrity"
//! Verifies that loaded ONNX models are signed by a trusted authority.
//!
//! # Example
//!
//! ```rust,ignore
//! use agentkern_gate::provenance::{ModelProvenance, ProvenanceError};
//!
//! let provenance = ModelProvenance::new("trusted_pubkey_base64");
//! provenance.verify_file("models/sentiment.onnx", "signature_base64")?;
//! ```

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Debug, thiserror::Error)]
pub enum ProvenanceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Decoding failed: {0}")]
    Decoding(String),
    #[error("Signature verification failed")]
    InvalidSignature,
    #[error("Invalid public key")]
    InvalidPublicKey,
}

/// Verifier for Neural Models.
pub struct ModelProvenance {
    trusted_key: VerifyingKey,
}

impl ModelProvenance {
    /// Create a new verifier with a trusted public key (Base64).
    pub fn new(public_key_b64: &str) -> Result<Self, ProvenanceError> {
        let bytes = BASE64
            .decode(public_key_b64)
            .map_err(|e| ProvenanceError::Decoding(e.to_string()))?;

        let array: [u8; 32] = bytes
            .try_into()
            .map_err(|_| ProvenanceError::InvalidPublicKey)?;

        let trusted_key = VerifyingKey::from_bytes(&array)
            .map_err(|_| ProvenanceError::InvalidPublicKey)?;

        Ok(Self { trusted_key })
    }

    /// Verify a model file against a detached signature.
    pub fn verify_file<P: AsRef<Path>>(
        &self,
        model_path: P,
        signature_b64: &str,
    ) -> Result<(), ProvenanceError> {
        // 1. Calculate SHA-256 hash of the model file
        let mut file = File::open(model_path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        let file_hash = hasher.finalize();

        // 2. Decode signature
        let sig_bytes = BASE64
            .decode(signature_b64)
            .map_err(|e| ProvenanceError::Decoding(e.to_string()))?;

        let signature = Signature::from_bytes(&sig_bytes.try_into().map_err(|_| ProvenanceError::InvalidSignature)?)
            .into(); // Convert from dalek Signature to internal if needed, usually direct

        // 3. Verify signature of the HASH
        self.trusted_key
            .verify(&file_hash, &signature)
            .map_err(|_| ProvenanceError::InvalidSignature)?;

        tracing::info!("Model provenance verified successfully.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;
    use std::io::Write;

    #[test]
    fn test_provenance_flow() {
        // 1. Generate Keypair
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let pubkey_b64 = BASE64.encode(verifying_key.to_bytes());

        // 2. Create Dummy Model
        let model_path = "test_model.onnx";
        let mut file = File::create(model_path).unwrap();
        file.write_all(b"fake-onnx-model-content").unwrap();

        // 3. Sign the Model (simulate build time signing)
        let mut hasher = Sha256::new();
        hasher.update(b"fake-onnx-model-content");
        let hash = hasher.finalize();
        let signature = signing_key.sign(&hash);
        let sig_b64 = BASE64.encode(signature.to_bytes());

        // 4. Verify
        let provenance = ModelProvenance::new(&pubkey_b64).unwrap();
        let result = provenance.verify_file(model_path, &sig_b64);

        // Cleanup
        std::fs::remove_file(model_path).unwrap();

        assert!(result.is_ok());
    }

    #[test]
    fn test_tampered_model() {
        // 1. Generate Keypair
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let pubkey_b64 = BASE64.encode(verifying_key.to_bytes());

        // 2. Create Dummy Model
        let model_path = "tampered_model.onnx";
        let mut file = File::create(model_path).unwrap();
        file.write_all(b"original-content").unwrap();

        // 3. Sign Original
        let mut hasher = Sha256::new();
        hasher.update(b"original-content");
        let hash = hasher.finalize();
        let signature = signing_key.sign(&hash);
        let sig_b64 = BASE64.encode(signature.to_bytes());

        // 4. Tamper with File
        let mut file = File::create(model_path).unwrap();
        file.write_all(b"malicious-content").unwrap();

        // 5. Verify
        let provenance = ModelProvenance::new(&pubkey_b64).unwrap();
        let result = provenance.verify_file(model_path, &sig_b64);

        // Cleanup
        std::fs::remove_file(model_path).unwrap();

        assert!(matches!(result, Err(ProvenanceError::InvalidSignature)));
    }
}
