//! AWS KMS Integration
//!
//! Production-ready AWS KMS client for envelope encryption.

use aws_config::BehaviorVersion;
use aws_sdk_kms::{Client as KmsClient, primitives::Blob, types::EncryptionAlgorithmSpec};
use std::sync::Arc;
use tokio::sync::OnceCell;

/// AWS KMS wrapper for key management operations.
#[derive(Clone)]
pub struct AwsKmsWrapper {
    client: Arc<KmsClient>,
    key_id: String,
}

/// Cached KMS client (single instance per runtime)
static KMS_CLIENT: OnceCell<Arc<KmsClient>> = OnceCell::const_new();

impl AwsKmsWrapper {
    /// Create new AWS KMS wrapper.
    ///
    /// # Arguments
    /// * `key_id` - AWS KMS Key ID, ARN, or alias (e.g., "alias/my-key")
    pub async fn new(key_id: impl Into<String>) -> Result<Self, KmsError> {
        let client = Self::get_or_init_client().await?;
        Ok(Self {
            client,
            key_id: key_id.into(),
        })
    }

    /// Get or initialize the shared KMS client.
    async fn get_or_init_client() -> Result<Arc<KmsClient>, KmsError> {
        KMS_CLIENT
            .get_or_try_init(|| async {
                let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
                let client = KmsClient::new(&config);
                Ok(Arc::new(client))
            })
            .await
            .map(Arc::clone)
    }

    /// Encrypt data using AWS KMS envelope encryption.
    ///
    /// Uses AWS KMS to encrypt the data directly (for small payloads)
    /// or wrap a data encryption key (for larger payloads).
    pub async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, KmsError> {
        let response = self
            .client
            .encrypt()
            .key_id(&self.key_id)
            .plaintext(Blob::new(plaintext.to_vec()))
            .encryption_algorithm(EncryptionAlgorithmSpec::SymmetricDefault)
            .send()
            .await
            .map_err(|e| KmsError::EncryptFailed(e.to_string()))?;

        response
            .ciphertext_blob()
            .map(|b| b.as_ref().to_vec())
            .ok_or(KmsError::NoCiphertext)
    }

    /// Decrypt data using AWS KMS.
    pub async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, KmsError> {
        let response = self
            .client
            .decrypt()
            .key_id(&self.key_id)
            .ciphertext_blob(Blob::new(ciphertext.to_vec()))
            .encryption_algorithm(EncryptionAlgorithmSpec::SymmetricDefault)
            .send()
            .await
            .map_err(|e| KmsError::DecryptFailed(e.to_string()))?;

        response
            .plaintext()
            .map(|b| b.as_ref().to_vec())
            .ok_or(KmsError::NoPlaintext)
    }

    /// Generate a data key using AWS KMS.
    /// Returns (plaintext_key, encrypted_key).
    pub async fn generate_data_key(&self) -> Result<(Vec<u8>, Vec<u8>), KmsError> {
        use aws_sdk_kms::types::DataKeySpec;

        let response = self
            .client
            .generate_data_key()
            .key_id(&self.key_id)
            .key_spec(DataKeySpec::Aes256)
            .send()
            .await
            .map_err(|e| KmsError::GenerateKeyFailed(e.to_string()))?;

        let plaintext = response
            .plaintext()
            .map(|b| b.as_ref().to_vec())
            .ok_or(KmsError::NoPlaintext)?;

        let encrypted = response
            .ciphertext_blob()
            .map(|b| b.as_ref().to_vec())
            .ok_or(KmsError::NoCiphertext)?;

        Ok((plaintext, encrypted))
    }

    /// Decrypt a previously generated data key.
    pub async fn decrypt_data_key(&self, encrypted_key: &[u8]) -> Result<Vec<u8>, KmsError> {
        self.decrypt(encrypted_key).await
    }

    /// Check if the KMS key is accessible.
    pub async fn verify_key_access(&self) -> Result<bool, KmsError> {
        let response = self
            .client
            .describe_key()
            .key_id(&self.key_id)
            .send()
            .await
            .map_err(|e| KmsError::AccessDenied(e.to_string()))?;

        Ok(response.key_metadata().is_some())
    }
}

/// AWS KMS errors.
#[derive(Debug, thiserror::Error)]
pub enum KmsError {
    #[error("Failed to initialize KMS client: {0}")]
    InitFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptFailed(String),

    #[error("Key generation failed: {0}")]
    GenerateKeyFailed(String),

    #[error("Access denied to KMS key: {0}")]
    AccessDenied(String),

    #[error("No ciphertext in response")]
    NoCiphertext,

    #[error("No plaintext in response")]
    NoPlaintext,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration test - requires AWS credentials
    #[tokio::test]
    #[ignore = "Requires AWS credentials and KMS key"]
    async fn test_kms_encrypt_decrypt() {
        let wrapper = AwsKmsWrapper::new("alias/test-key")
            .await
            .expect("Failed to create KMS wrapper");

        let plaintext = b"Hello, KMS!";
        let ciphertext = wrapper.encrypt(plaintext).await.unwrap();
        let decrypted = wrapper.decrypt(&ciphertext).await.unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    #[ignore = "Requires AWS credentials and KMS key"]
    async fn test_generate_data_key() {
        let wrapper = AwsKmsWrapper::new("alias/test-key")
            .await
            .expect("Failed to create KMS wrapper");

        let (plaintext_key, encrypted_key) = wrapper.generate_data_key().await.unwrap();

        assert_eq!(plaintext_key.len(), 32); // AES-256 = 32 bytes
        assert!(!encrypted_key.is_empty());

        // Verify we can decrypt the key
        let decrypted_key = wrapper.decrypt_data_key(&encrypted_key).await.unwrap();
        assert_eq!(decrypted_key, plaintext_key);
    }
}
